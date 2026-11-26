use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let database = manager.get_connection();

        for statement in [
            "ALTER TABLE fitness_profile_preferred_weekdays RENAME TO fitness_profile_available_weekdays",
            "ALTER TABLE fitness_profile_available_weekdays RENAME CONSTRAINT fitness_profile_weekday_range TO fitness_profile_available_weekday_range",
        ] {
            database.execute_unprepared(statement).await?;
        }

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let database = manager.get_connection();
        for statement in [
            "ALTER TABLE fitness_profile_available_weekdays RENAME CONSTRAINT fitness_profile_available_weekday_range TO fitness_profile_weekday_range",
            "ALTER TABLE fitness_profile_available_weekdays RENAME TO fitness_profile_preferred_weekdays",
        ] {
            database.execute_unprepared(statement).await?;
        }
        Ok(())
    }
}
