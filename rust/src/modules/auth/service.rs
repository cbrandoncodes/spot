use std::sync::Arc;

use better_auth::config::{BearerConfig, OAuthStateStrategy};
use better_auth::middleware::{CsrfConfig, RateLimitConfig, RateLimitStorageKind};
use better_auth::plugins::oauth::OAuthProvider;
use better_auth::plugins::{
    AccountManagementPlugin, EmailOtpPlugin, EmailOtpStorage, EmailPasswordPlugin, OAuthPlugin,
    PasswordManagementPlugin, SessionManagementPlugin, UserManagementPlugin,
};
use better_auth::seaorm::{DatabaseConnection, SeaOrmStore};
use better_auth::{AuthConfig, BetterAuth, config::IdGeneration};
use secrecy::ExposeSecret;

use crate::bootstrap::config::AuthSettings;

use super::infrastructure::resend::ResendOtpSender;
use super::schema::{AppAuthSchema, AppPluginSchema};

pub async fn build(
    settings: &AuthSettings,
    database: DatabaseConnection,
) -> better_auth::AuthResult<Arc<BetterAuth<AppAuthSchema>>> {
    let mut config = AuthConfig::new(settings.secret.expose_secret())
        .app_name("Spot")
        .base_url(settings.base_url.as_str())
        .base_path("/api/auth")
        .trusted_origins(settings.trusted_origins.clone())
        .password_min_length(10)
        .password_max_length(128);

    config.session.bearer = Some(BearerConfig::default());
    config.advanced.database.generate_id = Some(IdGeneration::Uuid);
    config.account.encrypt_oauth_tokens = Some(true);
    config.account.store_state_strategy = Some(OAuthStateStrategy::Database);
    config.account.account_linking.allow_different_emails = false;
    config.account.account_linking.allow_unlinking_all = Some(false);
    config.account.account_linking.require_local_email_verified = true;

    let store = SeaOrmStore::<AppAuthSchema>::new(config.clone(), database)
        .with_plugin_schema::<AppPluginSchema>();
    let otp_sender = ResendOtpSender::new(
        settings.resend.api_key.clone(),
        settings.resend.from_email.clone(),
    );
    let mut builder = BetterAuth::<AppAuthSchema>::new(config)
        .store(store)
        .csrf(CsrfConfig::new().enabled(true))
        .rate_limit(
            RateLimitConfig::new()
                .enabled(true)
                .storage(RateLimitStorageKind::Database),
        )
        .plugin(
            EmailPasswordPlugin::new()
                .enable_signup(true)
                .require_email_verification(true),
        )
        .plugin(
            EmailOtpPlugin::new()
                .sender(Arc::new(otp_sender))
                .storage(EmailOtpStorage::Hashed)
                .send_verification_on_sign_up(true)
                .change_email(true)
                .verify_current_email(true),
        )
        .plugin(SessionManagementPlugin::new())
        .plugin(PasswordManagementPlugin::new().revoke_sessions_on_password_reset(true))
        .plugin(AccountManagementPlugin::new())
        .plugin(UserManagementPlugin::new().delete_user_enabled(true));

    if let Some(google) = &settings.google {
        let oauth = OAuthPlugin::new().add_provider(
            "google",
            OAuthProvider::google(&google.client_id, google.client_secret.expose_secret()),
        );
        builder = builder.plugin(oauth);
    }

    Ok(Arc::new(builder.build().await?))
}
