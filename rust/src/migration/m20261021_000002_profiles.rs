use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let database = manager.get_connection();

        for statement in [
            r#"
                CREATE TABLE profiles (
                    user_id UUID PRIMARY KEY REFERENCES auth_users(id) ON DELETE CASCADE,
                    name TEXT NOT NULL,
                    username TEXT NULL,
                    username_normalized TEXT NULL,
                    avatar_key TEXT NULL,
                    bio TEXT NULL,
                    country_code VARCHAR(2) NULL,
                    city TEXT NULL,
                    district TEXT NULL,
                    visibility TEXT NOT NULL DEFAULT 'private',
                    discoverable_by_username BOOLEAN NOT NULL DEFAULT TRUE,
                    show_primary_gym BOOLEAN NOT NULL DEFAULT FALSE,
                    discoverable_in_gym BOOLEAN NOT NULL DEFAULT FALSE,
                    show_district BOOLEAN NOT NULL DEFAULT FALSE,
                    show_activity BOOLEAN NOT NULL DEFAULT FALSE,
                    show_training_presence BOOLEAN NOT NULL DEFAULT FALSE,
                    allow_gym_friend_requests BOOLEAN NOT NULL DEFAULT FALSE,
                    show_body_metrics BOOLEAN NOT NULL DEFAULT FALSE,
                    friend_request_policy TEXT NOT NULL DEFAULT 'anyone',
                    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                    CONSTRAINT profiles_name_length CHECK (char_length(name) BETWEEN 1 AND 80),
                    CONSTRAINT profiles_username_pair CHECK (
                        (username IS NULL AND username_normalized IS NULL)
                        OR (username IS NOT NULL AND username_normalized IS NOT NULL)
                    ),
                    CONSTRAINT profiles_username_length CHECK (
                        username IS NULL OR char_length(username) BETWEEN 3 AND 30
                    ),
                    CONSTRAINT profiles_bio_length CHECK (bio IS NULL OR char_length(bio) <= 160),
                    CONSTRAINT profiles_country_code_format CHECK (
                        country_code IS NULL OR country_code ~ '^[A-Z]{2}$'
                    ),
                    CONSTRAINT profiles_city_length CHECK (city IS NULL OR char_length(city) <= 80),
                    CONSTRAINT profiles_district_length CHECK (
                        district IS NULL OR char_length(district) <= 80
                    ),
                    CONSTRAINT profiles_visibility_values CHECK (
                        visibility IN ('public', 'private')
                    ),
                    CONSTRAINT profiles_friend_request_policy_values CHECK (
                        friend_request_policy IN ('anyone', 'friends_of_friends', 'same_gym', 'no_one')
                    )
                )
            "#,
            r#"
                CREATE UNIQUE INDEX profiles_username_normalized_uq
                ON profiles (username_normalized)
                WHERE username_normalized IS NOT NULL
            "#,
            r#"
                CREATE INDEX profiles_username_search_idx
                ON profiles USING GIN (username_normalized gin_trgm_ops)
                WHERE username_normalized IS NOT NULL AND discoverable_by_username
            "#,
        ] {
            database.execute_unprepared(statement).await?;
        }

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared("DROP TABLE IF EXISTS profiles")
            .await?;
        Ok(())
    }
}
