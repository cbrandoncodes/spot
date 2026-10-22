use std::str::FromStr;

use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, DbErr, EntityTrait,
    ExprTrait, IntoActiveModel, QueryFilter, QueryOrder, QuerySelect, TransactionTrait,
    sea_query::Expr,
};
use thiserror::Error;
use uuid::Uuid;

use crate::modules::fitness_profile::domain::{
    CompleteFitnessProfile, ExerciseAvoidance, ExerciseCode, GoalsExperienceAnswers,
    MovementConstraint, MovementWellnessAnswers, OnboardingAnswersV1, OnboardingGroup,
    PRIVACY_NOTICE_VERSION, QUESTIONNAIRE_VERSION, SNAPSHOT_SCHEMA_VERSION, ScheduleSetupAnswers,
    TrainingPreferencesAnswers, ValidationError, Weekday,
};

use super::entities::{
    draft, equipment, exercise_avoidance, fitness_profile, goal, movement_constraint,
    preferred_weekday, snapshot, wellness, workout_preference,
};

#[derive(Clone)]
pub struct FitnessProfileRepository {
    database: DatabaseConnection,
}

#[derive(Clone, Debug)]
pub struct FitnessProfileRecord {
    pub user_id: Uuid,
    pub revision: i64,
    pub questionnaire_version: i32,
    pub completed_at: chrono::DateTime<Utc>,
    pub profile: CompleteFitnessProfile,
}

impl FitnessProfileRepository {
    pub fn new(database: DatabaseConnection) -> Self {
        Self { database }
    }

    pub async fn get_or_create_draft(
        &self,
        user_id: Uuid,
    ) -> Result<draft::Model, RepositoryError> {
        if let Some(draft) = draft::Entity::find_by_id(user_id)
            .one(&self.database)
            .await?
        {
            return Ok(draft);
        }

        let now = Utc::now();
        let active = draft::ActiveModel {
            user_id: Set(user_id),
            questionnaire_version: Set(QUESTIONNAIRE_VERSION),
            current_group: Set(OnboardingGroup::GoalsExperience.as_str().to_owned()),
            answers: Set(serde_json::to_value(OnboardingAnswersV1::default())?),
            revision: Set(0),
            completed_at: Set(None),
            created_at: Set(now),
            updated_at: Set(now),
        };

        match active.insert(&self.database).await {
            Ok(draft) => Ok(draft),
            Err(error) if is_duplicate(&error) => draft::Entity::find_by_id(user_id)
                .one(&self.database)
                .await?
                .ok_or(RepositoryError::ConcurrentCreate),
            Err(error) => Err(error.into()),
        }
    }

    pub async fn save_draft(
        &self,
        user_id: Uuid,
        expected_revision: i64,
        current_group: OnboardingGroup,
        answers: &OnboardingAnswersV1,
    ) -> Result<draft::Model, RepositoryError> {
        let answers = serde_json::to_value(answers)?;
        let result = draft::Entity::update_many()
            .col_expr(draft::Column::Answers, Expr::value(answers))
            .col_expr(
                draft::Column::CurrentGroup,
                Expr::value(current_group.as_str()),
            )
            .col_expr(
                draft::Column::Revision,
                Expr::col(draft::Column::Revision).add(1),
            )
            .col_expr(
                draft::Column::CompletedAt,
                Expr::value(Option::<chrono::DateTime<Utc>>::None),
            )
            .col_expr(draft::Column::UpdatedAt, Expr::value(Utc::now()))
            .filter(draft::Column::UserId.eq(user_id))
            .filter(draft::Column::Revision.eq(expected_revision))
            .exec(&self.database)
            .await?;

        if result.rows_affected != 1 {
            return Err(RepositoryError::RevisionConflict);
        }

        draft::Entity::find_by_id(user_id)
            .one(&self.database)
            .await?
            .ok_or(RepositoryError::MissingDraft)
    }

    pub async fn complete(
        &self,
        user_id: Uuid,
        expected_draft_revision: i64,
        profile: &CompleteFitnessProfile,
    ) -> Result<FitnessProfileRecord, RepositoryError> {
        let transaction = self.database.begin().await?;
        let draft = draft::Entity::find_by_id(user_id)
            .filter(draft::Column::Revision.eq(expected_draft_revision))
            .lock_exclusive()
            .one(&transaction)
            .await?
            .ok_or(RepositoryError::RevisionConflict)?;

        let now = Utc::now();
        let existing = fitness_profile::Entity::find_by_id(user_id)
            .lock_exclusive()
            .one(&transaction)
            .await?;
        let profile_revision = existing.as_ref().map_or(1, |profile| profile.revision + 1);
        let consent_at = profile
            .movement_wellness
            .contains_sensitive_data()
            .then_some(now);
        let privacy_notice = consent_at.map(|_| PRIVACY_NOTICE_VERSION.to_owned());

        if let Some(existing) = existing {
            let created_at = existing.created_at;
            let mut active = existing.into_active_model();
            populate_profile_model(
                &mut active,
                profile,
                profile_revision,
                consent_at,
                privacy_notice,
                now,
            );
            active.created_at = Set(created_at);
            active.update(&transaction).await?;
        } else {
            let mut active = fitness_profile::ActiveModel {
                user_id: Set(user_id),
                created_at: Set(now),
                ..Default::default()
            };
            populate_profile_model(
                &mut active,
                profile,
                profile_revision,
                consent_at,
                privacy_notice,
                now,
            );
            active.insert(&transaction).await?;
        }

        delete_owned_rows(&transaction, user_id).await?;
        insert_owned_rows(&transaction, user_id, profile, now).await?;

        snapshot::ActiveModel {
            id: Set(Uuid::now_v7()),
            user_id: Set(user_id),
            profile_revision: Set(profile_revision),
            schema_version: Set(SNAPSHOT_SCHEMA_VERSION),
            normalized_input: Set(serde_json::to_value(profile)?),
            created_at: Set(now),
        }
        .insert(&transaction)
        .await?;

        let mut active_draft = draft.into_active_model();
        active_draft.current_group = Set(OnboardingGroup::Complete.as_str().to_owned());
        active_draft.revision = Set(expected_draft_revision + 1);
        active_draft.completed_at = Set(Some(now));
        active_draft.updated_at = Set(now);
        active_draft.update(&transaction).await?;

        transaction.commit().await?;
        self.find_profile(user_id)
            .await?
            .ok_or(RepositoryError::MissingProfile)
    }

    pub async fn find_profile(
        &self,
        user_id: Uuid,
    ) -> Result<Option<FitnessProfileRecord>, RepositoryError> {
        let Some(base) = fitness_profile::Entity::find_by_id(user_id)
            .one(&self.database)
            .await?
        else {
            return Ok(None);
        };

        let goals = goal::Entity::find()
            .filter(goal::Column::UserId.eq(user_id))
            .order_by_asc(goal::Column::Priority)
            .all(&self.database)
            .await?;
        let equipment = equipment::Entity::find()
            .filter(equipment::Column::UserId.eq(user_id))
            .all(&self.database)
            .await?;
        let weekdays = preferred_weekday::Entity::find()
            .filter(preferred_weekday::Column::UserId.eq(user_id))
            .order_by_asc(preferred_weekday::Column::Weekday)
            .all(&self.database)
            .await?;
        let workout_types = workout_preference::Entity::find()
            .filter(workout_preference::Column::UserId.eq(user_id))
            .all(&self.database)
            .await?;
        let avoidances = exercise_avoidance::Entity::find()
            .filter(exercise_avoidance::Column::UserId.eq(user_id))
            .all(&self.database)
            .await?;
        let constraints = movement_constraint::Entity::find()
            .filter(movement_constraint::Column::UserId.eq(user_id))
            .order_by_asc(movement_constraint::Column::CreatedAt)
            .all(&self.database)
            .await?;
        let wellness = wellness::Entity::find_by_id(user_id)
            .one(&self.database)
            .await?
            .ok_or(RepositoryError::CorruptData(
                "fitness profile wellness row is missing",
            ))?;

        let primary = goals.first().ok_or(RepositoryError::CorruptData(
            "fitness profile goals are missing",
        ))?;
        if primary.priority != 1 {
            return Err(RepositoryError::CorruptData(
                "fitness profile primary goal is missing",
            ));
        }

        let profile = CompleteFitnessProfile {
            goals_experience: GoalsExperienceAnswers {
                primary_goal: primary.goal_code.parse()?,
                secondary_goals: goals
                    .iter()
                    .skip(1)
                    .map(|row| row.goal_code.parse())
                    .collect::<Result<_, _>>()?,
                experience_level: base.experience_level.parse()?,
                current_activity_level: base.current_activity_level.parse()?,
            },
            schedule_setup: ScheduleSetupAnswers {
                days_per_week: base.training_days_per_week,
                session_duration_minutes: base.session_duration_minutes,
                training_environment: base.training_environment.parse()?,
                available_equipment: equipment
                    .iter()
                    .map(|row| row.equipment_code.parse())
                    .collect::<Result<_, _>>()?,
                preferred_weekdays: weekdays
                    .iter()
                    .map(|row| Weekday::from_number(row.weekday))
                    .collect::<Result<_, _>>()?,
            },
            training_preferences: TrainingPreferencesAnswers {
                preferred_workout_types: workout_types
                    .iter()
                    .map(|row| row.workout_type.parse())
                    .collect::<Result<_, _>>()?,
                disliked_exercises: avoidances
                    .into_iter()
                    .map(|row| {
                        Ok(ExerciseAvoidance {
                            exercise_code: ExerciseCode::from_str(&row.exercise_code)?,
                            preference: row.preference.parse()?,
                        })
                    })
                    .collect::<Result<_, ValidationError>>()?,
                coaching_style: base.coaching_style.parse()?,
                starting_intensity: base.starting_intensity.parse()?,
                consistency_barrier: base.consistency_barrier,
            },
            movement_wellness: MovementWellnessAnswers {
                movement_constraints: constraints
                    .into_iter()
                    .map(|row| {
                        Ok(MovementConstraint {
                            body_region: row.body_region.parse()?,
                            movement_pattern: row
                                .movement_pattern
                                .as_deref()
                                .map(str::parse)
                                .transpose()?,
                            impact: row.impact.parse()?,
                            notes: row.notes,
                        })
                    })
                    .collect::<Result<_, ValidationError>>()?,
                accessibility_needs: wellness.accessibility_needs,
                height_mm: wellness.height_mm,
                weight_grams: wellness.weight_grams,
                unit_system: base.unit_system.parse()?,
                acknowledge_sensitive_data: base.health_data_consent_at.is_some(),
            },
        };

        Ok(Some(FitnessProfileRecord {
            user_id,
            revision: base.revision,
            questionnaire_version: base.questionnaire_version,
            completed_at: base.completed_at,
            profile,
        }))
    }
}

fn populate_profile_model(
    active: &mut fitness_profile::ActiveModel,
    profile: &CompleteFitnessProfile,
    revision: i64,
    consent_at: Option<chrono::DateTime<Utc>>,
    privacy_notice: Option<String>,
    now: chrono::DateTime<Utc>,
) {
    active.experience_level = Set(profile
        .goals_experience
        .experience_level
        .as_str()
        .to_owned());
    active.current_activity_level = Set(profile
        .goals_experience
        .current_activity_level
        .as_str()
        .to_owned());
    active.training_days_per_week = Set(profile.schedule_setup.days_per_week);
    active.session_duration_minutes = Set(profile.schedule_setup.session_duration_minutes);
    active.training_environment = Set(profile
        .schedule_setup
        .training_environment
        .as_str()
        .to_owned());
    active.coaching_style = Set(profile
        .training_preferences
        .coaching_style
        .as_str()
        .to_owned());
    active.starting_intensity = Set(profile
        .training_preferences
        .starting_intensity
        .as_str()
        .to_owned());
    active.consistency_barrier = Set(profile.training_preferences.consistency_barrier.clone());
    active.unit_system = Set(profile.movement_wellness.unit_system.as_str().to_owned());
    active.questionnaire_version = Set(QUESTIONNAIRE_VERSION);
    active.revision = Set(revision);
    active.health_data_consent_at = Set(consent_at);
    active.privacy_notice_version = Set(privacy_notice);
    active.completed_at = Set(now);
    active.updated_at = Set(now);
}

async fn delete_owned_rows(
    transaction: &sea_orm::DatabaseTransaction,
    user_id: Uuid,
) -> Result<(), DbErr> {
    goal::Entity::delete_many()
        .filter(goal::Column::UserId.eq(user_id))
        .exec(transaction)
        .await?;
    equipment::Entity::delete_many()
        .filter(equipment::Column::UserId.eq(user_id))
        .exec(transaction)
        .await?;
    preferred_weekday::Entity::delete_many()
        .filter(preferred_weekday::Column::UserId.eq(user_id))
        .exec(transaction)
        .await?;
    workout_preference::Entity::delete_many()
        .filter(workout_preference::Column::UserId.eq(user_id))
        .exec(transaction)
        .await?;
    exercise_avoidance::Entity::delete_many()
        .filter(exercise_avoidance::Column::UserId.eq(user_id))
        .exec(transaction)
        .await?;
    movement_constraint::Entity::delete_many()
        .filter(movement_constraint::Column::UserId.eq(user_id))
        .exec(transaction)
        .await?;
    wellness::Entity::delete_many()
        .filter(wellness::Column::UserId.eq(user_id))
        .exec(transaction)
        .await?;
    Ok(())
}

async fn insert_owned_rows(
    transaction: &sea_orm::DatabaseTransaction,
    user_id: Uuid,
    profile: &CompleteFitnessProfile,
    now: chrono::DateTime<Utc>,
) -> Result<(), DbErr> {
    goal::Entity::insert_many(
        profile
            .goals_experience
            .ranked_goals()
            .map(|(goal, priority)| goal::ActiveModel {
                user_id: Set(user_id),
                goal_code: Set(goal.as_str().to_owned()),
                priority: Set(priority),
            }),
    )
    .exec(transaction)
    .await?;

    equipment::Entity::insert_many(profile.schedule_setup.available_equipment.iter().map(
        |equipment| equipment::ActiveModel {
            user_id: Set(user_id),
            equipment_code: Set(equipment.as_str().to_owned()),
        },
    ))
    .exec(transaction)
    .await?;

    if !profile.schedule_setup.preferred_weekdays.is_empty() {
        preferred_weekday::Entity::insert_many(
            profile
                .schedule_setup
                .preferred_weekdays
                .iter()
                .map(|weekday| preferred_weekday::ActiveModel {
                    user_id: Set(user_id),
                    weekday: Set(weekday.number()),
                }),
        )
        .exec(transaction)
        .await?;
    }

    workout_preference::Entity::insert_many(
        profile
            .training_preferences
            .preferred_workout_types
            .iter()
            .map(|workout_type| workout_preference::ActiveModel {
                user_id: Set(user_id),
                workout_type: Set(workout_type.as_str().to_owned()),
            }),
    )
    .exec(transaction)
    .await?;

    if !profile.training_preferences.disliked_exercises.is_empty() {
        exercise_avoidance::Entity::insert_many(
            profile
                .training_preferences
                .disliked_exercises
                .iter()
                .map(|avoidance| exercise_avoidance::ActiveModel {
                    user_id: Set(user_id),
                    exercise_code: Set(avoidance.exercise_code.as_str().to_owned()),
                    preference: Set(avoidance.preference.as_str().to_owned()),
                }),
        )
        .exec(transaction)
        .await?;
    }

    if !profile.movement_wellness.movement_constraints.is_empty() {
        movement_constraint::Entity::insert_many(
            profile
                .movement_wellness
                .movement_constraints
                .iter()
                .map(|constraint| movement_constraint::ActiveModel {
                    id: Set(Uuid::now_v7()),
                    user_id: Set(user_id),
                    body_region: Set(constraint.body_region.as_str().to_owned()),
                    movement_pattern: Set(constraint
                        .movement_pattern
                        .map(|value| value.as_str().to_owned())),
                    impact: Set(constraint.impact.as_str().to_owned()),
                    notes: Set(constraint.notes.clone()),
                    created_at: Set(now),
                }),
        )
        .exec(transaction)
        .await?;
    }

    wellness::ActiveModel {
        user_id: Set(user_id),
        accessibility_needs: Set(profile.movement_wellness.accessibility_needs.clone()),
        height_mm: Set(profile.movement_wellness.height_mm),
        weight_grams: Set(profile.movement_wellness.weight_grams),
    }
    .insert(transaction)
    .await?;
    Ok(())
}

fn is_duplicate(error: &DbErr) -> bool {
    let message = error.to_string();
    message.contains("onboarding_drafts_pkey") || message.contains("duplicate key")
}

#[derive(Debug, Error)]
pub enum RepositoryError {
    #[error("onboarding draft revision does not match the latest saved revision")]
    RevisionConflict,
    #[error("onboarding draft does not exist")]
    MissingDraft,
    #[error("fitness profile does not exist after completion")]
    MissingProfile,
    #[error("onboarding draft creation raced but the resulting row was not found")]
    ConcurrentCreate,
    #[error("stored fitness profile is invalid: {0}")]
    CorruptData(&'static str),
    #[error(transparent)]
    Validation(#[from] ValidationError),
    #[error("fitness profile serialization failed")]
    Serialization(#[from] serde_json::Error),
    #[error("fitness profile database operation failed")]
    Database(#[from] DbErr),
}
