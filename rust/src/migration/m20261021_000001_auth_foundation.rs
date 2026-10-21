use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let database = manager.get_connection();

        for statement in [
            r#"CREATE EXTENSION IF NOT EXISTS pg_trgm"#,
            r#"CREATE EXTENSION IF NOT EXISTS cube"#,
            r#"CREATE EXTENSION IF NOT EXISTS earthdistance"#,
            r#"
                CREATE TABLE auth_users (
                    id UUID PRIMARY KEY DEFAULT pg_catalog.gen_random_uuid(),
                    name TEXT NOT NULL,
                    email TEXT NOT NULL UNIQUE,
                    email_verified BOOLEAN NOT NULL,
                    image TEXT NULL,
                    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
                )
            "#,
            r#"
                CREATE TABLE auth_sessions (
                    id UUID PRIMARY KEY DEFAULT pg_catalog.gen_random_uuid(),
                    expires_at TIMESTAMPTZ NOT NULL,
                    token TEXT NOT NULL UNIQUE,
                    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                    updated_at TIMESTAMPTZ NOT NULL,
                    ip_address TEXT NULL,
                    user_agent TEXT NULL,
                    user_id UUID NOT NULL REFERENCES auth_users(id) ON DELETE CASCADE
                )
            "#,
            r#"CREATE INDEX auth_sessions_user_id_idx ON auth_sessions(user_id)"#,
            r#"
                CREATE TABLE auth_accounts (
                    id UUID PRIMARY KEY DEFAULT pg_catalog.gen_random_uuid(),
                    account_id TEXT NOT NULL,
                    provider_id TEXT NOT NULL,
                    user_id UUID NOT NULL REFERENCES auth_users(id) ON DELETE CASCADE,
                    access_token TEXT NULL,
                    refresh_token TEXT NULL,
                    id_token TEXT NULL,
                    access_token_expires_at TIMESTAMPTZ NULL,
                    refresh_token_expires_at TIMESTAMPTZ NULL,
                    scope TEXT NULL,
                    password TEXT NULL,
                    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                    updated_at TIMESTAMPTZ NOT NULL
                )
            "#,
            r#"CREATE INDEX auth_accounts_user_id_idx ON auth_accounts(user_id)"#,
            r#"
                CREATE TABLE auth_verifications (
                    id UUID PRIMARY KEY DEFAULT pg_catalog.gen_random_uuid(),
                    identifier TEXT NOT NULL,
                    value TEXT NOT NULL,
                    expires_at TIMESTAMPTZ NOT NULL,
                    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
                )
            "#,
            r#"CREATE INDEX auth_verifications_identifier_idx ON auth_verifications(identifier)"#,
            r#"
                CREATE TABLE auth_rate_limits (
                    id UUID PRIMARY KEY DEFAULT pg_catalog.gen_random_uuid(),
                    key TEXT NOT NULL UNIQUE,
                    count INTEGER NOT NULL,
                    last_request BIGINT NOT NULL
                )
            "#,
        ] {
            database.execute_unprepared(statement).await?;
        }

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let database = manager.get_connection();

        for statement in [
            "DROP TABLE IF EXISTS auth_rate_limits",
            "DROP TABLE IF EXISTS auth_verifications",
            "DROP TABLE IF EXISTS auth_accounts",
            "DROP TABLE IF EXISTS auth_sessions",
            "DROP TABLE IF EXISTS auth_users",
        ] {
            database.execute_unprepared(statement).await?;
        }

        Ok(())
    }
}
