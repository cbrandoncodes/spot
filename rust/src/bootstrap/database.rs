use better_auth::seaorm::{Database, DatabaseConnection, sea_orm::ConnectOptions};
use sea_orm_migration::MigratorTrait;
use secrecy::ExposeSecret;

use super::config::DatabaseSettings;
use crate::migration::Migrator;

pub async fn connect(
    settings: &DatabaseSettings,
) -> Result<DatabaseConnection, sea_orm_migration::DbErr> {
    let mut options = ConnectOptions::new(settings.url.expose_secret());
    options
        .max_connections(settings.max_connections)
        .connect_timeout(settings.acquire_timeout)
        .sqlx_logging(false);

    Database::connect(options).await
}

pub async fn migrate(database: &DatabaseConnection) -> Result<(), sea_orm_migration::DbErr> {
    Migrator::up(database, None).await
}
