use sea_orm_migration::prelude::*;

mod m20261021_000001_auth_foundation;
mod m20261021_000002_profiles;
mod m20261022_000003_fitness_profile;
mod m20261022_000004_gym_directory;
mod m20261102_000001_available_weekdays;
mod m20261102_000002_session_duration_range;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20261021_000001_auth_foundation::Migration),
            Box::new(m20261021_000002_profiles::Migration),
            Box::new(m20261022_000003_fitness_profile::Migration),
            Box::new(m20261022_000004_gym_directory::Migration),
            Box::new(m20261102_000001_available_weekdays::Migration),
            Box::new(m20261102_000002_session_duration_range::Migration),
        ]
    }
}
