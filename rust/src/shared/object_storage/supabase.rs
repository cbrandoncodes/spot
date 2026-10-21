use std::time::Duration;

use reqwest::{
    Client, Method, StatusCode, Url,
    header::{AUTHORIZATION, CACHE_CONTROL, CONTENT_TYPE, HeaderMap, HeaderName, HeaderValue},
};
use secrecy::ExposeSecret;
use thiserror::Error;

use crate::bootstrap::config::SupabaseSettings;

const API_KEY: HeaderName = HeaderName::from_static("apikey");
const X_UPSERT: HeaderName = HeaderName::from_static("x-upsert");
const DEFAULT_CACHE_CONTROL: &str = "max-age=31536000";

#[derive(Clone)]
pub struct SupabaseStorage {
    http: Client,
    objects_url: Url,
    public_objects_url: Url,
    bucket: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StoredObject {
    pub key: String,
    pub public_url: Url,
}

impl SupabaseStorage {
    pub fn new(settings: &SupabaseSettings) -> Result<Self, SupabaseStorageInitError> {
        let project_url =
            Url::parse(&settings.url).map_err(|_| SupabaseStorageInitError::InvalidProjectUrl)?;
        if !matches!(project_url.scheme(), "http" | "https")
            || project_url.cannot_be_a_base()
            || project_url.query().is_some()
            || project_url.fragment().is_some()
        {
            return Err(SupabaseStorageInitError::InvalidProjectUrl);
        }

        validate_bucket_name(&settings.main_bucket_name)?;

        let mut api_key = HeaderValue::from_str(settings.secret_key.expose_secret())
            .map_err(|_| SupabaseStorageInitError::InvalidSecretKey)?;
        api_key.set_sensitive(true);
        let mut authorization =
            HeaderValue::from_str(&format!("Bearer {}", settings.secret_key.expose_secret()))
                .map_err(|_| SupabaseStorageInitError::InvalidSecretKey)?;
        authorization.set_sensitive(true);

        let mut headers = HeaderMap::new();
        headers.insert(API_KEY, api_key);
        headers.insert(AUTHORIZATION, authorization);

        let http = Client::builder()
            .default_headers(headers)
            .timeout(Duration::from_secs(10))
            .user_agent(concat!("spot-backend/", env!("CARGO_PKG_VERSION")))
            .build()
            .map_err(SupabaseStorageInitError::BuildClient)?;
        let public_objects_url = project_url
            .join("/storage/v1/object/public/")
            .map_err(|_| SupabaseStorageInitError::InvalidProjectUrl)?;
        let objects_url = project_url
            .join("/storage/v1/object/")
            .map_err(|_| SupabaseStorageInitError::InvalidProjectUrl)?;

        Ok(Self {
            http,
            objects_url,
            public_objects_url,
            bucket: settings.main_bucket_name.clone(),
        })
    }

    pub fn bucket_name(&self) -> &str {
        &self.bucket
    }

    pub fn public_url(&self, object_key: &str) -> Result<Url, StorageUrlError> {
        self.url_for(&self.public_objects_url, object_key)
    }

    pub async fn upload(
        &self,
        object_key: &str,
        content_type: &str,
        contents: Vec<u8>,
    ) -> Result<StoredObject, StorageOperationError> {
        self.write_object(Method::POST, "upload", object_key, content_type, contents)
            .await
    }

    pub async fn replace(
        &self,
        object_key: &str,
        content_type: &str,
        contents: Vec<u8>,
    ) -> Result<StoredObject, StorageOperationError> {
        self.write_object(Method::PUT, "replace", object_key, content_type, contents)
            .await
    }

    pub async fn exists(&self, object_key: &str) -> Result<bool, StorageOperationError> {
        let url = self.object_url(object_key)?;
        let response =
            self.http
                .head(url)
                .send()
                .await
                .map_err(|source| StorageOperationError::Request {
                    operation: "check existence",
                    source,
                })?;

        match response.status() {
            status if status.is_success() => Ok(true),
            StatusCode::NOT_FOUND => Ok(false),
            status => Err(StorageOperationError::Rejected {
                operation: "check existence",
                status,
            }),
        }
    }

    pub async fn delete(&self, object_key: &str) -> Result<(), StorageOperationError> {
        let url = self.object_url(object_key)?;
        let response = self.http.delete(url).send().await.map_err(|source| {
            StorageOperationError::Request {
                operation: "delete",
                source,
            }
        })?;

        ensure_success("delete", response.status())
    }

    async fn write_object(
        &self,
        method: Method,
        operation: &'static str,
        object_key: &str,
        content_type: &str,
        contents: Vec<u8>,
    ) -> Result<StoredObject, StorageOperationError> {
        let url = self.object_url(object_key)?;
        let public_url = self.public_url(object_key)?;
        let content_type = HeaderValue::from_str(content_type)
            .map_err(|_| StorageOperationError::InvalidContentType)?;
        let is_upload = method == Method::POST;
        let mut request = self
            .http
            .request(method, url)
            .header(CONTENT_TYPE, content_type)
            .header(CACHE_CONTROL, DEFAULT_CACHE_CONTROL)
            .body(contents);
        if is_upload {
            request = request.header(X_UPSERT, "false");
        }
        let response = request
            .send()
            .await
            .map_err(|source| StorageOperationError::Request { operation, source })?;

        ensure_success(operation, response.status())?;

        Ok(StoredObject {
            key: object_key.to_owned(),
            public_url,
        })
    }

    fn object_url(&self, object_key: &str) -> Result<Url, StorageUrlError> {
        self.url_for(&self.objects_url, object_key)
    }

    fn url_for(&self, base_url: &Url, object_key: &str) -> Result<Url, StorageUrlError> {
        let segments = validate_object_key(object_key)?;
        let mut url = base_url.clone();
        let mut path = url
            .path_segments_mut()
            .map_err(|_| StorageUrlError::InvalidBaseUrl)?;
        path.pop_if_empty().push(&self.bucket);
        for segment in segments {
            path.push(segment);
        }
        drop(path);

        Ok(url)
    }
}

fn ensure_success(
    operation: &'static str,
    status: StatusCode,
) -> Result<(), StorageOperationError> {
    if status.is_success() {
        Ok(())
    } else {
        Err(StorageOperationError::Rejected { operation, status })
    }
}

fn validate_bucket_name(bucket: &str) -> Result<(), SupabaseStorageInitError> {
    if bucket.trim().is_empty() || bucket.contains('/') || bucket.contains('\\') {
        return Err(SupabaseStorageInitError::InvalidBucketName);
    }
    Ok(())
}

fn validate_object_key(object_key: &str) -> Result<Vec<&str>, StorageUrlError> {
    let segments = object_key.split('/').collect::<Vec<_>>();
    if segments.iter().any(|segment| {
        segment.is_empty()
            || matches!(*segment, "." | "..")
            || segment.contains('\\')
            || segment.chars().any(char::is_control)
    }) {
        return Err(StorageUrlError::InvalidObjectKey);
    }
    Ok(segments)
}

#[derive(Debug, Error)]
pub enum SupabaseStorageInitError {
    #[error("SUPABASE_URL must be a valid HTTP(S) project URL without a query or fragment")]
    InvalidProjectUrl,
    #[error("SUPABASE_SECRET_KEY is not a valid HTTP credential")]
    InvalidSecretKey,
    #[error("SUPABASE_MAIN_BUCKET_NAME must be a non-empty bucket name, not a path")]
    InvalidBucketName,
    #[error("failed to construct the Supabase HTTP client")]
    BuildClient(#[source] reqwest::Error),
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum StorageUrlError {
    #[error("object key must contain only non-empty, safe path segments")]
    InvalidObjectKey,
    #[error("the configured storage URL cannot contain path segments")]
    InvalidBaseUrl,
}

#[derive(Debug, Error)]
pub enum StorageOperationError {
    #[error(transparent)]
    InvalidObjectKey(#[from] StorageUrlError),
    #[error("content type is not a valid HTTP header value")]
    InvalidContentType,
    #[error("Supabase Storage {operation} request failed")]
    Request {
        operation: &'static str,
        #[source]
        source: reqwest::Error,
    },
    #[error("Supabase Storage {operation} request was rejected with HTTP {status}")]
    Rejected {
        operation: &'static str,
        status: StatusCode,
    },
}

#[cfg(test)]
mod tests {
    use secrecy::SecretString;

    use super::{StorageUrlError, SupabaseSettings, SupabaseStorage};

    fn settings() -> SupabaseSettings {
        SupabaseSettings {
            url: "https://example.supabase.co".to_owned(),
            secret_key: SecretString::from("sb_secret_test"),
            main_bucket_name: "main".to_owned(),
        }
    }

    #[test]
    fn builds_public_url_and_encodes_segments() {
        let storage = SupabaseStorage::new(&settings()).expect("valid settings");
        let url = storage
            .public_url("avatars/user id/avatar.png")
            .expect("valid object key");

        assert_eq!(storage.bucket_name(), "main");
        assert_eq!(
            url.as_str(),
            "https://example.supabase.co/storage/v1/object/public/main/avatars/user%20id/avatar.png"
        );
    }

    #[test]
    fn rejects_unsafe_object_keys() {
        let storage = SupabaseStorage::new(&settings()).expect("valid settings");

        assert_eq!(
            storage.public_url("avatars/../secret.png"),
            Err(StorageUrlError::InvalidObjectKey)
        );
        assert_eq!(
            storage.public_url("/avatars/me.png"),
            Err(StorageUrlError::InvalidObjectKey)
        );
    }

    #[tokio::test]
    async fn rejects_invalid_upload_input_before_request() {
        let storage = SupabaseStorage::new(&settings()).expect("valid settings");

        let invalid_key = storage
            .upload("avatars/../secret.png", "image/png", Vec::new())
            .await;
        assert!(matches!(
            invalid_key,
            Err(super::StorageOperationError::InvalidObjectKey(
                StorageUrlError::InvalidObjectKey
            ))
        ));

        let invalid_content_type = storage
            .upload("avatars/me.png", "image/png\r\nx-unsafe: true", Vec::new())
            .await;
        assert!(matches!(
            invalid_content_type,
            Err(super::StorageOperationError::InvalidContentType)
        ));
    }
}
