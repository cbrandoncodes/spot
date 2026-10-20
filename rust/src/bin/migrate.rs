use anyhow::Context;
use spot_backend::bootstrap::{config::Settings, database, telemetry};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    telemetry::initialize()?;

    let settings = Settings::from_env().context("invalid application configuration")?;
    let pool = database::connect(&settings.database)
        .await
        .context("failed to connect to PostgreSQL")?;

    database::migrate(&pool)
        .await
        .context("failed to run database migrations")?;
    tracing::info!("database migrations completed");

    Ok(())
}
