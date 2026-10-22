use sea_orm::DatabaseConnection;
use thiserror::Error;
use uuid::Uuid;

use super::{
    domain::{
        GroupPostPage, GymBranchProfile, GymOrganizationProfile, GymSearch, GymSearchPage,
        OfficialGroupProfile, ValidationError,
    },
    infrastructure::postgres::{GymDirectoryRepository, RepositoryError},
};

#[derive(Clone)]
pub struct GymDirectoryService {
    repository: GymDirectoryRepository,
}

impl GymDirectoryService {
    pub fn new(database: DatabaseConnection) -> Self {
        Self {
            repository: GymDirectoryRepository::new(database),
        }
    }

    pub async fn search(&self, search: GymSearch) -> Result<GymSearchPage, ServiceError> {
        let search = search.validate()?;
        let mut items = self.repository.search(&search).await?;
        let has_more = items.len() > search.limit as usize;
        items.truncate(search.limit as usize);
        Ok(GymSearchPage {
            items,
            next_offset: has_more.then_some(search.offset + search.limit),
        })
    }

    pub async fn branch(&self, branch_id: Uuid) -> Result<GymBranchProfile, ServiceError> {
        self.repository
            .find_branch(branch_id)
            .await?
            .ok_or(ServiceError::BranchNotFound)
    }

    pub async fn organization(&self, slug: &str) -> Result<GymOrganizationProfile, ServiceError> {
        let slug = slug.trim();
        if slug.is_empty()
            || slug.len() > 80
            || slug.starts_with('-')
            || slug.ends_with('-')
            || slug.contains("--")
            || !slug
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
        {
            return Err(ValidationError::new("slug", "contains an invalid gym slug").into());
        }
        self.repository
            .find_organization(slug)
            .await?
            .ok_or(ServiceError::OrganizationNotFound)
    }

    pub async fn group(&self, group_id: Uuid) -> Result<OfficialGroupProfile, ServiceError> {
        self.repository
            .find_group(group_id)
            .await?
            .ok_or(ServiceError::GroupNotFound)
    }

    pub async fn group_posts(
        &self,
        group_id: Uuid,
        limit: u64,
        offset: u64,
    ) -> Result<GroupPostPage, ServiceError> {
        if !(1..=50).contains(&limit) {
            return Err(ValidationError::new("limit", "must be between 1 and 50").into());
        }
        if offset > 500 {
            return Err(ValidationError::new("offset", "must not exceed 500").into());
        }
        self.group(group_id).await?;
        let mut items = self
            .repository
            .list_group_posts(group_id, limit, offset)
            .await?;
        let has_more = items.len() > limit as usize;
        items.truncate(limit as usize);
        Ok(GroupPostPage {
            items,
            next_offset: has_more.then_some(offset + limit),
        })
    }
}

#[derive(Debug, Error)]
pub enum ServiceError {
    #[error(transparent)]
    Validation(#[from] ValidationError),
    #[error("gym branch was not found")]
    BranchNotFound,
    #[error("gym organization was not found")]
    OrganizationNotFound,
    #[error("official gym group was not found")]
    GroupNotFound,
    #[error("gym directory persistence failed")]
    Repository(#[from] RepositoryError),
}
