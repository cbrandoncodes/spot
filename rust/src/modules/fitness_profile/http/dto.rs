use std::str::FromStr;

use serde::{Deserialize, Serialize};

use super::super::{
    domain::{
        ExerciseAvoidance, ExerciseCode, GoalsExperienceAnswers, MovementConstraint,
        MovementWellnessAnswers, OnboardingAnswersV1, OnboardingGroup, ScheduleSetupAnswers,
        TrainingPreferencesAnswers, ValidationError,
    },
    infrastructure::postgres::FitnessProfileRecord,
    service::OnboardingDraft,
};

#[derive(Debug, Deserialize, garde::Validate)]
#[garde(allow_unvalidated)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct GoalsExperienceRequest {
    #[garde(range(min = 0))]
    pub(super) revision: i64,
    #[garde(length(bytes, max = 64))]
    primary_goal: String,
    #[garde(length(max = 2), inner(length(bytes, max = 64)))]
    secondary_goals: Vec<String>,
    #[garde(length(bytes, max = 64))]
    experience_level: String,
    #[garde(length(bytes, max = 64))]
    current_activity_level: String,
}

impl GoalsExperienceRequest {
    pub(super) fn into_domain(self) -> Result<GoalsExperienceAnswers, ValidationError> {
        Ok(GoalsExperienceAnswers {
            primary_goal: self.primary_goal.parse()?,
            secondary_goals: parse_values(&self.secondary_goals)?,
            experience_level: self.experience_level.parse()?,
            current_activity_level: self.current_activity_level.parse()?,
        })
    }
}

#[derive(Debug, Deserialize, garde::Validate)]
#[garde(allow_unvalidated)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct ScheduleSetupRequest {
    #[garde(range(min = 0))]
    pub(super) revision: i64,
    #[garde(range(min = 1, max = 7))]
    days_per_week: i16,
    #[garde(range(min = 15, max = 180))]
    minimum_duration_minutes: i16,
    #[garde(range(min = 15, max = 180))]
    preferred_duration_minutes: i16,
    #[garde(range(min = 15, max = 180))]
    maximum_duration_minutes: i16,
    #[garde(length(bytes, max = 32))]
    training_environment: String,
    #[garde(length(min = 1, max = 20), inner(length(bytes, max = 64)))]
    available_equipment: Vec<String>,
    #[garde(length(max = 7), inner(length(bytes, max = 16)))]
    preferred_weekdays: Vec<String>,
}

impl ScheduleSetupRequest {
    pub(super) fn into_domain(self) -> Result<ScheduleSetupAnswers, ValidationError> {
        Ok(ScheduleSetupAnswers {
            days_per_week: self.days_per_week,
            minimum_duration_minutes: self.minimum_duration_minutes,
            preferred_duration_minutes: self.preferred_duration_minutes,
            maximum_duration_minutes: self.maximum_duration_minutes,
            training_environment: self.training_environment.parse()?,
            available_equipment: parse_values(&self.available_equipment)?,
            preferred_weekdays: parse_values(&self.preferred_weekdays)?,
        })
    }
}

#[derive(Debug, Deserialize, garde::Validate)]
#[garde(allow_unvalidated)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct TrainingPreferencesRequest {
    #[garde(range(min = 0))]
    pub(super) revision: i64,
    #[garde(length(min = 1, max = 6), inner(length(bytes, max = 64)))]
    preferred_workout_types: Vec<String>,
    #[garde(length(max = 20), dive)]
    disliked_exercises: Vec<ExerciseAvoidanceRequest>,
    #[garde(length(bytes, max = 64))]
    coaching_style: String,
    #[garde(length(bytes, max = 32))]
    starting_intensity: String,
    #[garde(length(chars, max = 512))]
    consistency_barrier: Option<String>,
}

impl TrainingPreferencesRequest {
    pub(super) fn into_domain(self) -> Result<TrainingPreferencesAnswers, ValidationError> {
        Ok(TrainingPreferencesAnswers {
            preferred_workout_types: parse_values(&self.preferred_workout_types)?,
            disliked_exercises: self
                .disliked_exercises
                .into_iter()
                .map(ExerciseAvoidanceRequest::into_domain)
                .collect::<Result<_, _>>()?,
            coaching_style: self.coaching_style.parse()?,
            starting_intensity: self.starting_intensity.parse()?,
            consistency_barrier: self.consistency_barrier,
        })
    }
}

#[derive(Debug, Deserialize, garde::Validate)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ExerciseAvoidanceRequest {
    #[garde(length(bytes, min = 1, max = 64))]
    exercise_code: String,
    #[garde(length(bytes, min = 1, max = 32))]
    preference: String,
}

impl ExerciseAvoidanceRequest {
    fn into_domain(self) -> Result<ExerciseAvoidance, ValidationError> {
        Ok(ExerciseAvoidance {
            exercise_code: ExerciseCode::parse_supported(&self.exercise_code)?,
            preference: self.preference.parse()?,
        })
    }
}

#[derive(Debug, Deserialize, garde::Validate)]
#[garde(allow_unvalidated)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct MovementWellnessRequest {
    #[garde(range(min = 0))]
    pub(super) revision: i64,
    #[garde(length(max = 12), dive)]
    movement_constraints: Vec<MovementConstraintRequest>,
    #[garde(length(chars, max = 512))]
    accessibility_needs: Option<String>,
    #[garde(range(min = 500, max = 2750))]
    height_mm: Option<i32>,
    #[garde(range(min = 20_000, max = 500_000))]
    weight_grams: Option<i32>,
    #[garde(length(bytes, max = 16))]
    unit_system: String,
    acknowledge_sensitive_data: bool,
}

impl MovementWellnessRequest {
    pub(super) fn into_domain(self) -> Result<MovementWellnessAnswers, ValidationError> {
        Ok(MovementWellnessAnswers {
            movement_constraints: self
                .movement_constraints
                .into_iter()
                .map(MovementConstraintRequest::into_domain)
                .collect::<Result<_, _>>()?,
            accessibility_needs: self.accessibility_needs,
            height_mm: self.height_mm,
            weight_grams: self.weight_grams,
            unit_system: self.unit_system.parse()?,
            acknowledge_sensitive_data: self.acknowledge_sensitive_data,
        })
    }
}

#[derive(Debug, Deserialize, garde::Validate)]
#[garde(allow_unvalidated)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct MovementConstraintRequest {
    #[garde(length(bytes, max = 32))]
    body_region: String,
    #[garde(length(bytes, max = 32))]
    movement_pattern: Option<String>,
    #[garde(length(bytes, max = 16))]
    impact: String,
    #[garde(length(chars, max = 512))]
    notes: Option<String>,
}

impl MovementConstraintRequest {
    fn into_domain(self) -> Result<MovementConstraint, ValidationError> {
        Ok(MovementConstraint {
            body_region: self.body_region.parse()?,
            movement_pattern: self
                .movement_pattern
                .as_deref()
                .map(str::parse)
                .transpose()?,
            impact: self.impact.parse()?,
            notes: self.notes,
        })
    }
}

#[derive(Debug, Deserialize, garde::Validate)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct CompleteOnboardingRequest {
    #[garde(range(min = 0))]
    pub(super) revision: i64,
}

#[derive(Debug, Deserialize, garde::Validate)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct ReopenOnboardingRequest {
    #[garde(range(min = 0))]
    pub(super) revision: i64,
}

fn parse_values<T>(values: &[String]) -> Result<Vec<T>, ValidationError>
where
    T: FromStr<Err = ValidationError>,
{
    values.iter().map(|value| value.parse()).collect()
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct OnboardingResponse {
    questionnaire_version: i32,
    revision: i64,
    current_group: &'static str,
    completed_at: Option<String>,
    groups: Vec<GroupProgressResponse>,
    answers: OnboardingAnswersV1,
}

impl From<OnboardingDraft> for OnboardingResponse {
    fn from(draft: OnboardingDraft) -> Self {
        let completed = draft.answers.completed_groups();
        let groups = [
            OnboardingGroup::GoalsExperience,
            OnboardingGroup::ScheduleSetup,
            OnboardingGroup::TrainingPreferences,
            OnboardingGroup::MovementWellness,
        ]
        .into_iter()
        .map(|group| GroupProgressResponse {
            key: group.as_str(),
            status: if completed.contains(&group) {
                "complete"
            } else if draft.current_group == group {
                "in_progress"
            } else {
                "not_started"
            },
        })
        .collect();

        Self {
            questionnaire_version: draft.questionnaire_version,
            revision: draft.revision,
            current_group: draft.current_group.as_str(),
            completed_at: draft.completed_at.map(|value| value.to_rfc3339()),
            groups,
            answers: draft.answers,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GroupProgressResponse {
    key: &'static str,
    status: &'static str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct FitnessProfileResponse {
    user_id: String,
    revision: i64,
    questionnaire_version: i32,
    completed_at: String,
    profile: super::super::domain::CompleteFitnessProfile,
}

impl From<FitnessProfileRecord> for FitnessProfileResponse {
    fn from(record: FitnessProfileRecord) -> Self {
        Self {
            user_id: record.user_id.to_string(),
            revision: record.revision,
            questionnaire_version: record.questionnaire_version,
            completed_at: record.completed_at.to_rfc3339(),
            profile: record.profile,
        }
    }
}
