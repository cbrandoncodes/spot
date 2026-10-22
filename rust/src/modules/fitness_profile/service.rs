use sea_orm::DatabaseConnection;
use thiserror::Error;
use uuid::Uuid;

use super::{
    domain::{
        GoalsExperienceAnswers, MovementWellnessAnswers, OnboardingAnswersV1, OnboardingGroup,
        QUESTIONNAIRE_VERSION, ScheduleSetupAnswers, TrainingPreferencesAnswers, ValidationError,
    },
    infrastructure::postgres::{FitnessProfileRecord, FitnessProfileRepository, RepositoryError},
};

#[derive(Clone)]
pub struct FitnessProfileService {
    repository: FitnessProfileRepository,
}

#[derive(Clone, Debug)]
pub struct OnboardingDraft {
    pub questionnaire_version: i32,
    pub current_group: OnboardingGroup,
    pub answers: OnboardingAnswersV1,
    pub revision: i64,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl FitnessProfileService {
    pub fn new(database: DatabaseConnection) -> Self {
        Self {
            repository: FitnessProfileRepository::new(database),
        }
    }

    pub async fn get_onboarding(&self, user_id: Uuid) -> Result<OnboardingDraft, ServiceError> {
        let draft = self.repository.get_or_create_draft(user_id).await?;
        decode_draft(draft)
    }

    pub async fn save_goals_experience(
        &self,
        user_id: Uuid,
        revision: i64,
        answers: GoalsExperienceAnswers,
    ) -> Result<OnboardingDraft, ServiceError> {
        answers.validate()?;
        let mut draft = self.get_onboarding(user_id).await?;
        ensure_revision(revision, draft.revision)?;
        draft.answers.goals_experience = Some(answers);
        self.save(
            user_id,
            revision,
            OnboardingGroup::ScheduleSetup,
            &draft.answers,
        )
        .await
    }

    pub async fn save_schedule_setup(
        &self,
        user_id: Uuid,
        revision: i64,
        answers: ScheduleSetupAnswers,
    ) -> Result<OnboardingDraft, ServiceError> {
        answers.validate()?;
        let mut draft = self.get_onboarding(user_id).await?;
        ensure_revision(revision, draft.revision)?;
        draft.answers.schedule_setup = Some(answers);
        self.save(
            user_id,
            revision,
            OnboardingGroup::TrainingPreferences,
            &draft.answers,
        )
        .await
    }

    pub async fn save_training_preferences(
        &self,
        user_id: Uuid,
        revision: i64,
        mut answers: TrainingPreferencesAnswers,
    ) -> Result<OnboardingDraft, ServiceError> {
        answers.validate()?;
        let mut draft = self.get_onboarding(user_id).await?;
        ensure_revision(revision, draft.revision)?;
        draft.answers.training_preferences = Some(answers);
        self.save(
            user_id,
            revision,
            OnboardingGroup::MovementWellness,
            &draft.answers,
        )
        .await
    }

    pub async fn save_movement_wellness(
        &self,
        user_id: Uuid,
        revision: i64,
        mut answers: MovementWellnessAnswers,
    ) -> Result<OnboardingDraft, ServiceError> {
        answers.validate()?;
        let mut draft = self.get_onboarding(user_id).await?;
        ensure_revision(revision, draft.revision)?;
        draft.answers.movement_wellness = Some(answers);
        self.save(
            user_id,
            revision,
            OnboardingGroup::MovementWellness,
            &draft.answers,
        )
        .await
    }

    pub async fn complete(
        &self,
        user_id: Uuid,
        revision: i64,
    ) -> Result<FitnessProfileRecord, ServiceError> {
        let draft = self.get_onboarding(user_id).await?;
        if draft.completed_at.is_some() && draft.revision == revision + 1 {
            return self
                .get_profile(user_id)
                .await?
                .ok_or(ServiceError::CompletedProfileMissing);
        }
        ensure_revision(revision, draft.revision)?;
        if draft.questionnaire_version != QUESTIONNAIRE_VERSION {
            return Err(ServiceError::QuestionnaireVersionUnsupported);
        }
        let profile = draft.answers.complete()?;
        self.repository
            .complete(user_id, revision, &profile)
            .await
            .map_err(Into::into)
    }

    pub async fn get_profile(
        &self,
        user_id: Uuid,
    ) -> Result<Option<FitnessProfileRecord>, ServiceError> {
        self.repository
            .find_profile(user_id)
            .await
            .map_err(Into::into)
    }

    async fn save(
        &self,
        user_id: Uuid,
        revision: i64,
        current_group: OnboardingGroup,
        answers: &OnboardingAnswersV1,
    ) -> Result<OnboardingDraft, ServiceError> {
        let draft = self
            .repository
            .save_draft(user_id, revision, current_group, answers)
            .await?;
        decode_draft(draft)
    }
}

fn ensure_revision(provided: i64, current: i64) -> Result<(), ServiceError> {
    if provided == current {
        Ok(())
    } else {
        Err(ServiceError::RevisionConflict)
    }
}

fn decode_draft(
    draft: super::infrastructure::postgres::DraftRecord,
) -> Result<OnboardingDraft, ServiceError> {
    Ok(OnboardingDraft {
        questionnaire_version: draft.questionnaire_version,
        current_group: draft.current_group.parse()?,
        answers: serde_json::from_value(draft.answers).map_err(ServiceError::InvalidStoredDraft)?,
        revision: draft.revision,
        completed_at: draft.completed_at,
    })
}

#[derive(Debug, Error)]
pub enum ServiceError {
    #[error(transparent)]
    Validation(#[from] ValidationError),
    #[error("onboarding draft changed; reload before saving")]
    RevisionConflict,
    #[error("the stored onboarding questionnaire version is no longer supported")]
    QuestionnaireVersionUnsupported,
    #[error("stored onboarding answers are invalid")]
    InvalidStoredDraft(serde_json::Error),
    #[error("completed onboarding has no fitness profile")]
    CompletedProfileMissing,
    #[error("fitness profile persistence failed")]
    Repository(RepositoryError),
}

impl From<RepositoryError> for ServiceError {
    fn from(error: RepositoryError) -> Self {
        match error {
            RepositoryError::RevisionConflict => Self::RevisionConflict,
            RepositoryError::Validation(error) => Self::Validation(error),
            error => Self::Repository(error),
        }
    }
}
