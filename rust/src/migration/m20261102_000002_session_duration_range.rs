use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let database = manager.get_connection();

        for statement in [
            r#"
                ALTER TABLE fitness_profiles
                    ADD COLUMN minimum_duration_minutes SMALLINT,
                    ADD COLUMN preferred_duration_minutes SMALLINT,
                    ADD COLUMN maximum_duration_minutes SMALLINT
            "#,
            r#"
                UPDATE fitness_profiles
                SET minimum_duration_minutes = session_duration_minutes,
                    preferred_duration_minutes = session_duration_minutes,
                    maximum_duration_minutes = session_duration_minutes
            "#,
            r#"
                ALTER TABLE fitness_profiles
                    DROP CONSTRAINT fitness_profiles_duration_range,
                    ALTER COLUMN minimum_duration_minutes SET NOT NULL,
                    ALTER COLUMN preferred_duration_minutes SET NOT NULL,
                    ALTER COLUMN maximum_duration_minutes SET NOT NULL,
                    ADD CONSTRAINT fitness_profiles_duration_range CHECK (
                        minimum_duration_minutes BETWEEN 15 AND 180
                        AND preferred_duration_minutes BETWEEN 15 AND 180
                        AND maximum_duration_minutes BETWEEN 15 AND 180
                        AND minimum_duration_minutes <= preferred_duration_minutes
                        AND preferred_duration_minutes <= maximum_duration_minutes
                    ),
                    DROP COLUMN session_duration_minutes
            "#,
            r#"
                UPDATE onboarding_drafts
                SET answers = jsonb_set(
                    jsonb_set(
                        jsonb_set(
                            answers #- '{scheduleSetup,sessionDurationMinutes}',
                            '{scheduleSetup,minimumDurationMinutes}',
                            answers #> '{scheduleSetup,sessionDurationMinutes}',
                            true
                        ),
                        '{scheduleSetup,preferredDurationMinutes}',
                        answers #> '{scheduleSetup,sessionDurationMinutes}',
                        true
                    ),
                    '{scheduleSetup,maximumDurationMinutes}',
                    answers #> '{scheduleSetup,sessionDurationMinutes}',
                    true
                )
                WHERE answers #> '{scheduleSetup,sessionDurationMinutes}' IS NOT NULL
            "#,
        ] {
            database.execute_unprepared(statement).await?;
        }

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let database = manager.get_connection();

        for statement in [
            r#"
                ALTER TABLE fitness_profiles
                    ADD COLUMN session_duration_minutes SMALLINT
            "#,
            r#"
                UPDATE fitness_profiles
                SET session_duration_minutes = preferred_duration_minutes
            "#,
            r#"
                ALTER TABLE fitness_profiles
                    DROP CONSTRAINT fitness_profiles_duration_range,
                    ALTER COLUMN session_duration_minutes SET NOT NULL,
                    ADD CONSTRAINT fitness_profiles_duration_range CHECK (
                        session_duration_minutes BETWEEN 15 AND 180
                    ),
                    DROP COLUMN minimum_duration_minutes,
                    DROP COLUMN preferred_duration_minutes,
                    DROP COLUMN maximum_duration_minutes
            "#,
            r#"
                UPDATE onboarding_drafts
                SET answers = jsonb_set(
                    (
                        (
                            answers #- '{scheduleSetup,minimumDurationMinutes}'
                        ) #- '{scheduleSetup,preferredDurationMinutes}'
                    ) #- '{scheduleSetup,maximumDurationMinutes}',
                    '{scheduleSetup,sessionDurationMinutes}',
                    answers #> '{scheduleSetup,preferredDurationMinutes}',
                    true
                )
                WHERE answers #> '{scheduleSetup,preferredDurationMinutes}' IS NOT NULL
            "#,
        ] {
            database.execute_unprepared(statement).await?;
        }

        Ok(())
    }
}
