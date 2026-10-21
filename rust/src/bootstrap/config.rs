use std::{env, net::SocketAddr, str::FromStr, time::Duration};

use secrecy::SecretString;
use thiserror::Error;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Environment {
    Local,
    Test,
    Staging,
    Production,
}

impl FromStr for Environment {
    type Err = ConfigError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.to_ascii_lowercase().as_str() {
            "local" => Ok(Self::Local),
            "test" => Ok(Self::Test),
            "staging" => Ok(Self::Staging),
            "production" => Ok(Self::Production),
            _ => Err(ConfigError::InvalidValue {
                name: "SPOT_ENVIRONMENT",
                value: value.to_owned(),
            }),
        }
    }
}

#[derive(Clone, Debug)]
pub struct DatabaseSettings {
    pub url: SecretString,
    pub max_connections: u32,
    pub acquire_timeout: Duration,
    pub run_migrations: bool,
}

#[derive(Clone, Debug)]
pub struct GoogleOAuthSettings {
    pub client_id: String,
    pub client_secret: SecretString,
}

#[derive(Clone, Debug)]
pub struct ResendSettings {
    pub api_key: SecretString,
    pub from_email: String,
}

#[derive(Clone, Debug)]
pub struct AuthSettings {
    pub secret: SecretString,
    pub base_url: String,
    pub trusted_origins: Vec<String>,
    pub google: Option<GoogleOAuthSettings>,
    pub resend: ResendSettings,
}

#[derive(Clone, Debug)]
pub struct Settings {
    pub environment: Environment,
    pub http_addr: SocketAddr,
    pub database: DatabaseSettings,
    pub auth: AuthSettings,
}

impl Settings {
    pub fn from_env() -> Result<Self, ConfigError> {
        let environment = optional("SPOT_ENVIRONMENT", "local").parse()?;
        let http_addr = parse(
            "SPOT_HTTP_ADDR",
            &optional("SPOT_HTTP_ADDR", "127.0.0.1:8080"),
        )?;
        let database = database_from_env(environment)?;
        let auth_secret = required("SPOT_AUTH_SECRET")?;
        if auth_secret.len() < 32 {
            return Err(ConfigError::TooShort {
                name: "SPOT_AUTH_SECRET",
                minimum: 32,
            });
        }
        let base_url = optional("SPOT_AUTH_BASE_URL", "http://127.0.0.1:8080");
        let trusted_origins = comma_separated(&optional(
            "SPOT_AUTH_TRUSTED_ORIGINS",
            "http://localhost:3000",
        ));
        let google = paired_optional_secrets("SPOT_GOOGLE_CLIENT_ID", "SPOT_GOOGLE_CLIENT_SECRET")?
            .map(|(client_id, client_secret)| GoogleOAuthSettings {
                client_id,
                client_secret: client_secret.into(),
            });
        let resend = ResendSettings {
            api_key: required("SPOT_RESEND_API_KEY")?.into(),
            from_email: required("SPOT_EMAIL_FROM")?,
        };

        Ok(Self {
            environment,
            http_addr,
            database,
            auth: AuthSettings {
                secret: auth_secret.into(),
                base_url,
                trusted_origins,
                google,
                resend,
            },
        })
    }

    pub fn database_from_env() -> Result<DatabaseSettings, ConfigError> {
        let environment = optional("SPOT_ENVIRONMENT", "local").parse()?;
        database_from_env(environment)
    }
}

fn database_from_env(environment: Environment) -> Result<DatabaseSettings, ConfigError> {
    let max_connections = parse(
        "SPOT_DATABASE_MAX_CONNECTIONS",
        &optional("SPOT_DATABASE_MAX_CONNECTIONS", "10"),
    )?;
    let acquire_timeout_seconds = parse(
        "SPOT_DATABASE_ACQUIRE_TIMEOUT_SECONDS",
        &optional("SPOT_DATABASE_ACQUIRE_TIMEOUT_SECONDS", "5"),
    )?;
    let run_migrations_default = matches!(environment, Environment::Local | Environment::Test);
    let run_migrations = match env::var("SPOT_DATABASE_RUN_MIGRATIONS") {
        Ok(value) => parse("SPOT_DATABASE_RUN_MIGRATIONS", &value)?,
        Err(env::VarError::NotPresent) => run_migrations_default,
        Err(env::VarError::NotUnicode(_)) => {
            return Err(ConfigError::NotUnicode("SPOT_DATABASE_RUN_MIGRATIONS"));
        }
    };

    Ok(DatabaseSettings {
        url: required("DATABASE_URL")?.into(),
        max_connections,
        acquire_timeout: Duration::from_secs(acquire_timeout_seconds),
        run_migrations,
    })
}

fn comma_separated(value: &str) -> Vec<String> {
    value
        .split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .collect()
}

fn paired_optional_secrets(
    first_name: &'static str,
    second_name: &'static str,
) -> Result<Option<(String, String)>, ConfigError> {
    let first = env::var(first_name)
        .ok()
        .filter(|value| !value.trim().is_empty());
    let second = env::var(second_name)
        .ok()
        .filter(|value| !value.trim().is_empty());

    match (first, second) {
        (None, None) => Ok(None),
        (Some(first), Some(second)) => Ok(Some((first, second))),
        _ => Err(ConfigError::IncompletePair {
            first: first_name,
            second: second_name,
        }),
    }
}

fn required(name: &'static str) -> Result<String, ConfigError> {
    match env::var(name) {
        Ok(value) if !value.trim().is_empty() => Ok(value),
        Ok(_) | Err(env::VarError::NotPresent) => Err(ConfigError::Missing(name)),
        Err(env::VarError::NotUnicode(_)) => Err(ConfigError::NotUnicode(name)),
    }
}

fn optional(name: &'static str, default: &'static str) -> String {
    env::var(name).unwrap_or_else(|_| default.to_owned())
}

fn parse<T>(name: &'static str, value: &str) -> Result<T, ConfigError>
where
    T: FromStr,
{
    value.parse().map_err(|_| ConfigError::InvalidValue {
        name,
        value: value.to_owned(),
    })
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("required environment variable `{0}` is missing")]
    Missing(&'static str),
    #[error("environment variable `{0}` is not valid UTF-8")]
    NotUnicode(&'static str),
    #[error("environment variable `{name}` has invalid value `{value}`")]
    InvalidValue { name: &'static str, value: String },
    #[error("environment variable `{name}` must contain at least {minimum} characters")]
    TooShort { name: &'static str, minimum: usize },
    #[error("environment variables `{first}` and `{second}` must be set together")]
    IncompletePair {
        first: &'static str,
        second: &'static str,
    },
}

#[cfg(test)]
mod tests {
    use super::{Environment, FromStr};

    #[test]
    fn environment_is_case_insensitive() {
        assert_eq!(
            Environment::from_str("PRODUCTION").expect("valid environment"),
            Environment::Production
        );
    }
}
