use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let database = manager.get_connection();

        for statement in [
            r#"
                CREATE TABLE onboarding_drafts (
                    user_id UUID PRIMARY KEY REFERENCES auth_users(id) ON DELETE CASCADE,
                    questionnaire_version INTEGER NOT NULL DEFAULT 1,
                    current_group TEXT NOT NULL DEFAULT 'goals_experience',
                    answers JSONB NOT NULL DEFAULT '{}'::jsonb,
                    revision BIGINT NOT NULL DEFAULT 0,
                    completed_at TIMESTAMPTZ NULL,
                    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                    CONSTRAINT onboarding_drafts_questionnaire_version_positive
                        CHECK (questionnaire_version > 0),
                    CONSTRAINT onboarding_drafts_revision_nonnegative CHECK (revision >= 0),
                    CONSTRAINT onboarding_drafts_current_group_values CHECK (
                        current_group IN (
                            'goals_experience',
                            'schedule_setup',
                            'training_preferences',
                            'movement_wellness',
                            'complete'
                        )
                    ),
                    CONSTRAINT onboarding_drafts_answers_object
                        CHECK (jsonb_typeof(answers) = 'object')
                )
            "#,
            r#"
                CREATE TABLE fitness_profiles (
                    user_id UUID PRIMARY KEY REFERENCES auth_users(id) ON DELETE CASCADE,
                    experience_level TEXT NOT NULL,
                    current_activity_level TEXT NOT NULL,
                    training_days_per_week SMALLINT NOT NULL,
                    session_duration_minutes SMALLINT NOT NULL,
                    training_environment TEXT NOT NULL,
                    coaching_style TEXT NOT NULL,
                    starting_intensity TEXT NOT NULL,
                    consistency_barrier TEXT NULL,
                    unit_system TEXT NOT NULL DEFAULT 'metric',
                    questionnaire_version INTEGER NOT NULL,
                    revision BIGINT NOT NULL,
                    health_data_consent_at TIMESTAMPTZ NULL,
                    privacy_notice_version TEXT NULL,
                    completed_at TIMESTAMPTZ NOT NULL,
                    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                    CONSTRAINT fitness_profiles_experience_values CHECK (
                        experience_level IN ('never_trained', 'beginner', 'intermediate', 'advanced')
                    ),
                    CONSTRAINT fitness_profiles_activity_values CHECK (
                        current_activity_level IN ('mostly_inactive', 'lightly_active', 'active', 'very_active')
                    ),
                    CONSTRAINT fitness_profiles_days_range CHECK (training_days_per_week BETWEEN 1 AND 7),
                    CONSTRAINT fitness_profiles_duration_range CHECK (session_duration_minutes BETWEEN 15 AND 180),
                    CONSTRAINT fitness_profiles_environment_values CHECK (
                        training_environment IN ('gym', 'home', 'mixed')
                    ),
                    CONSTRAINT fitness_profiles_coaching_values CHECK (
                        coaching_style IN ('calm_supportive', 'motivational', 'direct', 'technical', 'balanced')
                    ),
                    CONSTRAINT fitness_profiles_intensity_values CHECK (
                        starting_intensity IN ('gentle', 'balanced', 'challenging')
                    ),
                    CONSTRAINT fitness_profiles_barrier_length CHECK (
                        consistency_barrier IS NULL OR char_length(consistency_barrier) <= 160
                    ),
                    CONSTRAINT fitness_profiles_unit_values CHECK (unit_system IN ('metric', 'imperial')),
                    CONSTRAINT fitness_profiles_questionnaire_version_positive CHECK (questionnaire_version > 0),
                    CONSTRAINT fitness_profiles_revision_positive CHECK (revision > 0),
                    CONSTRAINT fitness_profiles_privacy_notice_length CHECK (
                        privacy_notice_version IS NULL OR char_length(privacy_notice_version) <= 40
                    )
                )
            "#,
            r#"
                CREATE TABLE fitness_profile_goals (
                    user_id UUID NOT NULL REFERENCES fitness_profiles(user_id) ON DELETE CASCADE,
                    goal_code TEXT NOT NULL,
                    priority SMALLINT NOT NULL,
                    PRIMARY KEY (user_id, goal_code),
                    CONSTRAINT fitness_profile_goals_priority_unique UNIQUE (user_id, priority),
                    CONSTRAINT fitness_profile_goals_priority_range CHECK (priority BETWEEN 1 AND 3),
                    CONSTRAINT fitness_profile_goals_code_values CHECK (
                        goal_code IN (
                            'general_fitness', 'build_muscle', 'get_stronger',
                            'improve_conditioning', 'improve_mobility', 'athletic_performance'
                        )
                    )
                )
            "#,
            r#"
                CREATE TABLE fitness_profile_equipment (
                    user_id UUID NOT NULL REFERENCES fitness_profiles(user_id) ON DELETE CASCADE,
                    equipment_code TEXT NOT NULL,
                    PRIMARY KEY (user_id, equipment_code),
                    CONSTRAINT fitness_profile_equipment_code_values CHECK (
                        equipment_code IN (
                            'bodyweight', 'dumbbells', 'adjustable_bench', 'barbell', 'weight_plates',
                            'squat_rack', 'cable_station', 'lat_pulldown', 'leg_press', 'smith_machine',
                            'resistance_bands', 'kettlebells', 'pull_up_bar', 'cardio_machines'
                        )
                    )
                )
            "#,
            r#"
                CREATE TABLE fitness_profile_preferred_weekdays (
                    user_id UUID NOT NULL REFERENCES fitness_profiles(user_id) ON DELETE CASCADE,
                    weekday SMALLINT NOT NULL,
                    PRIMARY KEY (user_id, weekday),
                    CONSTRAINT fitness_profile_weekday_range CHECK (weekday BETWEEN 1 AND 7)
                )
            "#,
            r#"
                CREATE TABLE fitness_profile_workout_preferences (
                    user_id UUID NOT NULL REFERENCES fitness_profiles(user_id) ON DELETE CASCADE,
                    workout_type TEXT NOT NULL,
                    PRIMARY KEY (user_id, workout_type),
                    CONSTRAINT fitness_profile_workout_type_values CHECK (
                        workout_type IN ('strength', 'hypertrophy', 'conditioning', 'bodyweight', 'mobility', 'mixed')
                    )
                )
            "#,
            r#"
                CREATE TABLE fitness_profile_exercise_avoidances (
                    user_id UUID NOT NULL REFERENCES fitness_profiles(user_id) ON DELETE CASCADE,
                    exercise_code TEXT NOT NULL,
                    preference TEXT NOT NULL,
                    PRIMARY KEY (user_id, exercise_code),
                    CONSTRAINT fitness_profile_exercise_code_format CHECK (
                        exercise_code ~ '^[a-z0-9]+(_[a-z0-9]+)*$' AND char_length(exercise_code) <= 64
                    ),
                    CONSTRAINT fitness_profile_exercise_preference_values CHECK (
                        preference IN ('prefer_alternative', 'never_include')
                    )
                )
            "#,
            r#"
                CREATE TABLE fitness_profile_movement_constraints (
                    id UUID PRIMARY KEY,
                    user_id UUID NOT NULL REFERENCES fitness_profiles(user_id) ON DELETE CASCADE,
                    body_region TEXT NOT NULL,
                    movement_pattern TEXT NULL,
                    impact TEXT NOT NULL,
                    notes TEXT NULL,
                    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                    CONSTRAINT fitness_profile_constraint_region_values CHECK (
                        body_region IN (
                            'neck', 'shoulder', 'elbow', 'wrist_hand', 'upper_back', 'lower_back',
                            'hip', 'knee', 'ankle_foot', 'other'
                        )
                    ),
                    CONSTRAINT fitness_profile_constraint_movement_values CHECK (
                        movement_pattern IS NULL OR movement_pattern IN (
                            'squat', 'hinge', 'lunge', 'horizontal_push', 'vertical_push',
                            'horizontal_pull', 'vertical_pull', 'carry', 'rotation', 'running', 'jumping'
                        )
                    ),
                    CONSTRAINT fitness_profile_constraint_impact_values CHECK (
                        impact IN ('monitor', 'modify', 'avoid')
                    ),
                    CONSTRAINT fitness_profile_constraint_notes_length CHECK (
                        notes IS NULL OR char_length(notes) <= 240
                    )
                )
            "#,
            r#"
                CREATE TABLE fitness_profile_wellness (
                    user_id UUID PRIMARY KEY REFERENCES fitness_profiles(user_id) ON DELETE CASCADE,
                    accessibility_needs TEXT NULL,
                    height_mm INTEGER NULL,
                    weight_grams INTEGER NULL,
                    CONSTRAINT fitness_profile_accessibility_length CHECK (
                        accessibility_needs IS NULL OR char_length(accessibility_needs) <= 240
                    ),
                    CONSTRAINT fitness_profile_height_range CHECK (
                        height_mm IS NULL OR height_mm BETWEEN 500 AND 2750
                    ),
                    CONSTRAINT fitness_profile_weight_range CHECK (
                        weight_grams IS NULL OR weight_grams BETWEEN 20000 AND 500000
                    )
                )
            "#,
            r#"
                CREATE TABLE fitness_profile_snapshots (
                    id UUID PRIMARY KEY,
                    user_id UUID NOT NULL REFERENCES fitness_profiles(user_id) ON DELETE CASCADE,
                    profile_revision BIGINT NOT NULL,
                    schema_version INTEGER NOT NULL,
                    normalized_input JSONB NOT NULL,
                    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                    CONSTRAINT fitness_profile_snapshots_revision_unique UNIQUE (user_id, profile_revision),
                    CONSTRAINT fitness_profile_snapshots_revision_positive CHECK (profile_revision > 0),
                    CONSTRAINT fitness_profile_snapshots_schema_positive CHECK (schema_version > 0),
                    CONSTRAINT fitness_profile_snapshots_input_object CHECK (
                        jsonb_typeof(normalized_input) = 'object'
                    )
                )
            "#,
            r#"
                CREATE INDEX fitness_profile_snapshots_user_created_idx
                ON fitness_profile_snapshots (user_id, created_at DESC)
            "#,
        ] {
            database.execute_unprepared(statement).await?;
        }

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let database = manager.get_connection();
        for statement in [
            "DROP TABLE IF EXISTS fitness_profile_snapshots",
            "DROP TABLE IF EXISTS fitness_profile_wellness",
            "DROP TABLE IF EXISTS fitness_profile_movement_constraints",
            "DROP TABLE IF EXISTS fitness_profile_exercise_avoidances",
            "DROP TABLE IF EXISTS fitness_profile_workout_preferences",
            "DROP TABLE IF EXISTS fitness_profile_preferred_weekdays",
            "DROP TABLE IF EXISTS fitness_profile_equipment",
            "DROP TABLE IF EXISTS fitness_profile_goals",
            "DROP TABLE IF EXISTS fitness_profiles",
            "DROP TABLE IF EXISTS onboarding_drafts",
        ] {
            database.execute_unprepared(statement).await?;
        }
        Ok(())
    }
}
