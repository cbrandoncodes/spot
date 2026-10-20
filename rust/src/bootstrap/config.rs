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
pub struct Settings {
    pub environment: Environment,
    pub http_addr: SocketAddr,
    pub database: DatabaseSettings,
}

impl Settings {
    pub fn from_env() -> Result<Self, ConfigError> {
        let environment = optional("SPOT_ENVIRONMENT", "local").parse()?;
        let http_addr = parse(
            "SPOT_HTTP_ADDR",
            &optional("SPOT_HTTP_ADDR", "127.0.0.1:8080"),
        )?;
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

        Ok(Self {
            environment,
            http_addr,
            database: DatabaseSettings {
                url: required("DATABASE_URL")?.into(),
                max_connections,
                acquire_timeout: Duration::from_secs(acquire_timeout_seconds),
                run_migrations,
            },
        })
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
