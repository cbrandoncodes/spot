use anyhow::Context;
use spot_backend::{
    AppState,
    bootstrap::{config::Settings, database, http, telemetry},
};
use tracing::{error, info};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    telemetry::initialize()?;

    let settings = Settings::from_env().context("invalid application configuration")?;
    let pool = database::connect(&settings.database)
        .await
        .context("failed to connect to PostgreSQL")?;

    if settings.database.run_migrations {
        database::migrate(&pool)
            .await
            .context("failed to run database migrations")?;
    }

    let auth = spot_backend::modules::auth::service::build(&settings.auth, pool.clone())
        .await
        .context("failed to initialize authentication")?;

    let listener = tokio::net::TcpListener::bind(settings.http_addr)
        .await
        .context("failed to bind HTTP listener")?;
    info!(
        address = %settings.http_addr,
        environment = ?settings.environment,
        "Spot API listening"
    );

    let app = http::router(AppState { db: pool, auth }, &settings.auth.trusted_origins)
        .context("SPOT_AUTH_TRUSTED_ORIGINS contains an invalid HTTP origin")?;

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .context("HTTP server failed")
}

async fn shutdown_signal() {
    if let Err(error) = tokio::signal::ctrl_c().await {
        error!(%error, "failed to install shutdown signal handler");
    }
}
