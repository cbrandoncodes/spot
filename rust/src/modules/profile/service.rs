use sea_orm::DatabaseConnection;
use thiserror::Error;
use uuid::Uuid;

use crate::shared::object_storage::{StorageOperationError, SupabaseStorage};

use super::{
    domain::{ProfileChanges, Username, ValidationError},
    infrastructure::postgres::{Model, PrivacyChanges, ProfileRepository, RepositoryError},
};

const MAX_AVATAR_BYTES: usize = 5 * 1024 * 1024;

#[derive(Clone)]
pub struct ProfileService {
    repository: ProfileRepository,
    storage: SupabaseStorage,
}

impl ProfileService {
    pub fn new(database: DatabaseConnection, storage: SupabaseStorage) -> Self {
        Self {
            repository: ProfileRepository::new(database),
            storage,
        }
    }

    pub async fn get_or_create(
        &self,
        user_id: Uuid,
        initial_name: &str,
    ) -> Result<Model, ProfileError> {
        self.repository
            .get_or_create(user_id, initial_name)
            .await
            .map_err(Into::into)
    }

    pub async fn update(
        &self,
        user_id: Uuid,
        initial_name: &str,
        changes: ProfileChanges,
    ) -> Result<Model, ProfileError> {
        let profile = self.repository.get_or_create(user_id, initial_name).await?;
        if let Some(username) = &changes.username
            && !self
                .repository
                .username_available(username.normalized(), user_id)
                .await?
        {
            return Err(ProfileError::UsernameTaken);
        }
        self.repository
            .update_profile(profile, changes)
            .await
            .map_err(Into::into)
    }

    pub async fn username_available(
        &self,
        user_id: Uuid,
        candidate: &str,
    ) -> Result<bool, ProfileError> {
        let username: Username = candidate.parse()?;
        self.repository
            .username_available(username.normalized(), user_id)
            .await
            .map_err(Into::into)
    }

    pub async fn find_discoverable(&self, username: &str) -> Result<Option<Model>, ProfileError> {
        let username: Username = username.parse()?;
        self.repository
            .find_discoverable_by_username(username.normalized())
            .await
            .map_err(Into::into)
    }

    pub async fn update_privacy(
        &self,
        user_id: Uuid,
        initial_name: &str,
        privacy: PrivacyChanges,
    ) -> Result<Model, ProfileError> {
        let profile = self.repository.get_or_create(user_id, initial_name).await?;
        self.repository
            .save_privacy(profile, privacy)
            .await
            .map_err(Into::into)
    }

    pub async fn upload_avatar(
        &self,
        user_id: Uuid,
        initial_name: &str,
        content_type: &str,
        contents: Vec<u8>,
    ) -> Result<Model, ProfileError> {
        if contents.is_empty() || contents.len() > MAX_AVATAR_BYTES {
            return Err(ProfileError::InvalidAvatar(
                "avatar must contain between 1 byte and 5 MB",
            ));
        }
        let extension = match content_type {
            "image/jpeg" if is_jpeg(&contents) => "jpg",
            "image/png" if is_png(&contents) => "png",
            "image/webp" if is_webp(&contents) => "webp",
            _ => {
                return Err(ProfileError::InvalidAvatar(
                    "avatar content must match its JPEG, PNG, or WebP Content-Type",
                ));
            }
        };
        let profile = self.repository.get_or_create(user_id, initial_name).await?;
        let object_key = format!("avatars/{user_id}/{}.{}", Uuid::now_v7(), extension);
        let stored = self
            .storage
            .upload(&object_key, content_type, contents)
            .await?;

        let result = self
            .repository
            .set_avatar_key(profile, Some(stored.key.clone()))
            .await;
        let (profile, previous) = match result {
            Ok(result) => result,
            Err(error) => {
                if let Err(cleanup_error) = self.storage.delete(&stored.key).await {
                    tracing::error!(error = %cleanup_error, "failed to clean up avatar after database error");
                }
                return Err(error.into());
            }
        };

        if let Some(previous) = previous
            && let Err(error) = self.storage.delete(&previous).await
        {
            tracing::warn!(error = %error, "previous avatar cleanup failed");
        }

        Ok(profile)
    }

    pub async fn delete_avatar(
        &self,
        user_id: Uuid,
        initial_name: &str,
    ) -> Result<Model, ProfileError> {
        let profile = self.repository.get_or_create(user_id, initial_name).await?;
        let (profile, previous) = self.repository.set_avatar_key(profile, None).await?;
        if let Some(previous) = previous
            && let Err(error) = self.storage.delete(&previous).await
        {
            tracing::warn!(error = %error, "deleted avatar cleanup failed");
        }
        Ok(profile)
    }

    pub fn avatar_url(&self, profile: &Model) -> Option<String> {
        profile.avatar_key.as_deref().and_then(|key| {
            self.storage
                .public_url(key)
                .map(|url| url.to_string())
                .map_err(|error| {
                    tracing::error!(error = %error, "stored avatar key is invalid");
                })
                .ok()
        })
    }
}

fn is_jpeg(contents: &[u8]) -> bool {
    contents.starts_with(&[0xff, 0xd8, 0xff])
}

fn is_png(contents: &[u8]) -> bool {
    contents.starts_with(b"\x89PNG\r\n\x1a\n")
}

fn is_webp(contents: &[u8]) -> bool {
    contents.len() >= 12 && contents.starts_with(b"RIFF") && &contents[8..12] == b"WEBP"
}

#[derive(Debug, Error)]
pub enum ProfileError {
    #[error(transparent)]
    Validation(#[from] ValidationError),
    #[error("username is already taken")]
    UsernameTaken,
    #[error("avatar is invalid: {0}")]
    InvalidAvatar(&'static str),
    #[error("profile persistence failed")]
    Repository(RepositoryError),
    #[error("avatar storage failed")]
    Storage(#[from] StorageOperationError),
}

impl From<RepositoryError> for ProfileError {
    fn from(error: RepositoryError) -> Self {
        match error {
            RepositoryError::UsernameTaken => Self::UsernameTaken,
            error => Self::Repository(error),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{is_jpeg, is_png, is_webp};

    #[test]
    fn recognizes_supported_image_signatures() {
        assert!(is_jpeg(&[0xff, 0xd8, 0xff, 0xe0]));
        assert!(is_png(b"\x89PNG\r\n\x1a\nrest"));
        assert!(is_webp(b"RIFF\x04\x00\x00\x00WEBP"));
        assert!(!is_png(b"not an image"));
        assert!(!is_webp(b"RIFFtoo short"));
    }
}
