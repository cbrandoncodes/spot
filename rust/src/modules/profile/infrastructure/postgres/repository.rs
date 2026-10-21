use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, DbErr, EntityTrait,
    IntoActiveModel, QueryFilter,
};
use thiserror::Error;
use uuid::Uuid;

use crate::modules::profile::domain::ProfileChanges;

use super::entity::{ActiveModel, Column, Entity, Model};

#[derive(Clone)]
pub struct ProfileRepository {
    database: DatabaseConnection,
}

impl ProfileRepository {
    pub fn new(database: DatabaseConnection) -> Self {
        Self { database }
    }

    pub async fn get_or_create(
        &self,
        user_id: Uuid,
        initial_name: &str,
    ) -> Result<Model, RepositoryError> {
        if let Some(profile) = Entity::find_by_id(user_id).one(&self.database).await? {
            return Ok(profile);
        }

        let now = Utc::now();
        let initial_name = initial_name.trim();
        let initial_name = if initial_name.is_empty() {
            "Spot member".to_owned()
        } else {
            initial_name.chars().take(80).collect()
        };
        let profile = ActiveModel {
            user_id: Set(user_id),
            name: Set(initial_name),
            username: Set(None),
            username_normalized: Set(None),
            avatar_key: Set(None),
            bio: Set(None),
            country_code: Set(None),
            city: Set(None),
            district: Set(None),
            visibility: Set("private".to_owned()),
            discoverable_by_username: Set(true),
            show_primary_gym: Set(false),
            discoverable_in_gym: Set(false),
            show_district: Set(false),
            show_activity: Set(false),
            show_training_presence: Set(false),
            allow_gym_friend_requests: Set(false),
            show_body_metrics: Set(false),
            friend_request_policy: Set("anyone".to_owned()),
            created_at: Set(now),
            updated_at: Set(now),
        };

        match profile.insert(&self.database).await {
            Ok(profile) => Ok(profile),
            Err(error) if is_primary_key_conflict(&error) => Entity::find_by_id(user_id)
                .one(&self.database)
                .await?
                .ok_or(RepositoryError::ConcurrentCreate),
            Err(error) => Err(error.into()),
        }
    }

    pub async fn username_available(
        &self,
        normalized: &str,
        excluding_user: Uuid,
    ) -> Result<bool, RepositoryError> {
        let existing = Entity::find()
            .filter(Column::UsernameNormalized.eq(normalized))
            .filter(Column::UserId.ne(excluding_user))
            .one(&self.database)
            .await?;
        Ok(existing.is_none())
    }

    pub async fn find_discoverable_by_username(
        &self,
        normalized: &str,
    ) -> Result<Option<Model>, RepositoryError> {
        Entity::find()
            .filter(Column::UsernameNormalized.eq(normalized))
            .filter(Column::DiscoverableByUsername.eq(true))
            .one(&self.database)
            .await
            .map_err(Into::into)
    }

    pub async fn update_profile(
        &self,
        profile: Model,
        changes: ProfileChanges,
    ) -> Result<Model, RepositoryError> {
        let mut active = profile.into_active_model();
        if let Some(name) = changes.name {
            active.name = Set(name.into_inner());
        }
        if let Some(username) = changes.username {
            let (display, normalized) = username.into_parts();
            active.username = Set(Some(display));
            active.username_normalized = Set(Some(normalized));
        }
        if let Some(bio) = changes.bio {
            active.bio = Set(bio.map(|value| value.into_inner()));
        }
        if let Some(country_code) = changes.country_code {
            active.country_code = Set(country_code.map(|value| value.into_inner()));
        }
        if let Some(city) = changes.city {
            active.city = Set(city.map(|value| value.into_inner()));
        }
        if let Some(district) = changes.district {
            active.district = Set(district.map(|value| value.into_inner()));
        }
        active.updated_at = Set(Utc::now());

        active.update(&self.database).await.map_err(|error| {
            if is_username_conflict(&error) {
                RepositoryError::UsernameTaken
            } else {
                RepositoryError::Database(error)
            }
        })
    }

    pub async fn save_privacy(
        &self,
        profile: Model,
        privacy: PrivacyChanges,
    ) -> Result<Model, RepositoryError> {
        let mut active = profile.into_active_model();
        if let Some(value) = privacy.visibility {
            active.visibility = Set(value);
        }
        assign(
            &mut active.discoverable_by_username,
            privacy.discoverable_by_username,
        );
        assign(&mut active.show_primary_gym, privacy.show_primary_gym);
        assign(&mut active.discoverable_in_gym, privacy.discoverable_in_gym);
        assign(&mut active.show_district, privacy.show_district);
        assign(&mut active.show_activity, privacy.show_activity);
        assign(
            &mut active.show_training_presence,
            privacy.show_training_presence,
        );
        assign(
            &mut active.allow_gym_friend_requests,
            privacy.allow_gym_friend_requests,
        );
        assign(&mut active.show_body_metrics, privacy.show_body_metrics);
        if let Some(value) = privacy.friend_request_policy {
            active.friend_request_policy = Set(value);
        }
        active.updated_at = Set(Utc::now());
        active.update(&self.database).await.map_err(Into::into)
    }

    pub async fn set_avatar_key(
        &self,
        profile: Model,
        avatar_key: Option<String>,
    ) -> Result<(Model, Option<String>), RepositoryError> {
        let previous = profile.avatar_key.clone();
        let mut active = profile.into_active_model();
        active.avatar_key = Set(avatar_key);
        active.updated_at = Set(Utc::now());
        let profile = active.update(&self.database).await?;
        Ok((profile, previous))
    }
}

#[derive(Clone, Debug, Default)]
pub struct PrivacyChanges {
    pub visibility: Option<String>,
    pub discoverable_by_username: Option<bool>,
    pub show_primary_gym: Option<bool>,
    pub discoverable_in_gym: Option<bool>,
    pub show_district: Option<bool>,
    pub show_activity: Option<bool>,
    pub show_training_presence: Option<bool>,
    pub allow_gym_friend_requests: Option<bool>,
    pub show_body_metrics: Option<bool>,
    pub friend_request_policy: Option<String>,
}

fn assign(target: &mut sea_orm::ActiveValue<bool>, value: Option<bool>) {
    if let Some(value) = value {
        *target = Set(value);
    }
}

fn is_username_conflict(error: &DbErr) -> bool {
    error
        .to_string()
        .contains("profiles_username_normalized_uq")
}

fn is_primary_key_conflict(error: &DbErr) -> bool {
    let message = error.to_string();
    message.contains("profiles_pkey") || message.contains("duplicate key")
}

#[derive(Debug, Error)]
pub enum RepositoryError {
    #[error("username is already taken")]
    UsernameTaken,
    #[error("profile creation raced but the resulting row was not found")]
    ConcurrentCreate,
    #[error("profile database operation failed")]
    Database(#[from] DbErr),
}
