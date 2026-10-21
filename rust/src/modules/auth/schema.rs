//! Application-owned Better Auth persistence models.
//!
//! Keep these models in lockstep with the auth migration. Regenerate and diff
//! them when upgrading the pinned Better Auth revision.

use better_auth::seaorm::sea_orm::entity::prelude::*;
use better_auth::{AuthSchema, seaorm::AuthEntity};

pub mod user {
    use super::*;

    #[derive(Clone, Debug, serde::Serialize, DeriveEntityModel, AuthEntity)]
    #[auth(role = "user")]
    #[sea_orm(table_name = "auth_users")]
    pub struct Model {
        #[sea_orm(primary_key, auto_increment = false)]
        pub id: Uuid,
        pub name: String,
        pub email: Option<String>,
        #[sea_orm(column_name = "email_verified")]
        #[auth(alias = "emailVerified")]
        pub email_verified: bool,
        pub image: Option<String>,
        #[sea_orm(column_name = "created_at")]
        #[auth(alias = "createdAt")]
        pub created_at: DateTimeUtc,
        #[sea_orm(column_name = "updated_at")]
        #[auth(alias = "updatedAt")]
        pub updated_at: DateTimeUtc,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod session {
    use super::*;

    #[derive(Clone, Debug, serde::Serialize, DeriveEntityModel, AuthEntity)]
    #[auth(role = "session", row_presence)]
    #[sea_orm(table_name = "auth_sessions")]
    pub struct Model {
        #[sea_orm(primary_key, auto_increment = false)]
        pub id: Uuid,
        #[sea_orm(column_name = "expires_at")]
        #[auth(alias = "expiresAt")]
        pub expires_at: DateTimeUtc,
        pub token: String,
        #[sea_orm(column_name = "created_at")]
        #[auth(alias = "createdAt")]
        pub created_at: DateTimeUtc,
        #[sea_orm(column_name = "updated_at")]
        #[auth(alias = "updatedAt")]
        pub updated_at: DateTimeUtc,
        #[sea_orm(column_name = "ip_address")]
        #[auth(alias = "ipAddress")]
        pub ip_address: Option<String>,
        #[sea_orm(column_name = "user_agent")]
        #[auth(alias = "userAgent")]
        pub user_agent: Option<String>,
        #[sea_orm(column_name = "user_id")]
        #[auth(alias = "userId")]
        pub user_id: Uuid,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod account {
    use super::*;

    #[derive(Clone, Debug, serde::Serialize, DeriveEntityModel, AuthEntity)]
    #[auth(role = "account")]
    #[sea_orm(table_name = "auth_accounts")]
    pub struct Model {
        #[sea_orm(primary_key, auto_increment = false)]
        pub id: Uuid,
        #[sea_orm(column_name = "account_id")]
        #[auth(alias = "accountId")]
        pub account_id: String,
        #[sea_orm(column_name = "provider_id")]
        #[auth(alias = "providerId")]
        pub provider_id: String,
        #[sea_orm(column_name = "user_id")]
        #[auth(alias = "userId")]
        pub user_id: Uuid,
        #[sea_orm(column_name = "access_token")]
        #[auth(alias = "accessToken")]
        pub access_token: Option<String>,
        #[sea_orm(column_name = "refresh_token")]
        #[auth(alias = "refreshToken")]
        pub refresh_token: Option<String>,
        #[sea_orm(column_name = "id_token")]
        #[auth(alias = "idToken")]
        pub id_token: Option<String>,
        #[sea_orm(column_name = "access_token_expires_at")]
        #[auth(alias = "accessTokenExpiresAt")]
        pub access_token_expires_at: Option<DateTimeUtc>,
        #[sea_orm(column_name = "refresh_token_expires_at")]
        #[auth(alias = "refreshTokenExpiresAt")]
        pub refresh_token_expires_at: Option<DateTimeUtc>,
        pub scope: Option<String>,
        pub password: Option<String>,
        #[sea_orm(column_name = "created_at")]
        #[auth(alias = "createdAt")]
        pub created_at: DateTimeUtc,
        #[sea_orm(column_name = "updated_at")]
        #[auth(alias = "updatedAt")]
        pub updated_at: DateTimeUtc,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod verification {
    use super::*;

    #[derive(Clone, Debug, serde::Serialize, DeriveEntityModel, AuthEntity)]
    #[auth(role = "verification")]
    #[sea_orm(table_name = "auth_verifications")]
    pub struct Model {
        #[sea_orm(primary_key, auto_increment = false)]
        pub id: Uuid,
        pub identifier: String,
        pub value: String,
        #[sea_orm(column_name = "expires_at")]
        #[auth(alias = "expiresAt")]
        pub expires_at: DateTimeUtc,
        #[sea_orm(column_name = "created_at")]
        #[auth(alias = "createdAt")]
        pub created_at: DateTimeUtc,
        #[sea_orm(column_name = "updated_at")]
        #[auth(alias = "updatedAt")]
        pub updated_at: DateTimeUtc,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod rate_limit {
    use super::*;

    #[derive(Clone, Debug, serde::Serialize, DeriveEntityModel, AuthEntity)]
    #[auth(role = "rate_limit")]
    #[sea_orm(table_name = "auth_rate_limits")]
    pub struct Model {
        #[sea_orm(primary_key, auto_increment = false)]
        pub id: Uuid,
        pub key: String,
        #[sea_orm(column_type = "Integer")]
        pub count: better_auth::seaorm::SqlNumber,
        #[sea_orm(column_name = "last_request")]
        #[auth(alias = "lastRequest")]
        pub last_request: i64,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

pub type AppPluginSchema = better_auth::seaorm::PluginModels<
    <better_auth::seaorm::PluginModels as better_auth::seaorm::SeaOrmPluginSchema>::ApiKey,
    <better_auth::seaorm::PluginModels as better_auth::seaorm::SeaOrmPluginSchema>::DeviceCode,
    <better_auth::seaorm::PluginModels as better_auth::seaorm::SeaOrmPluginSchema>::Passkey,
    <better_auth::seaorm::PluginModels as better_auth::seaorm::SeaOrmPluginSchema>::TwoFactor,
    <better_auth::seaorm::PluginModels as better_auth::seaorm::SeaOrmPluginSchema>::Jwk,
    <better_auth::seaorm::PluginModels as better_auth::seaorm::SeaOrmPluginSchema>::WalletAddress,
    rate_limit::Model,
>;

pub struct AppAuthSchema;

impl AuthSchema for AppAuthSchema {
    type User = user::Model;
    type Session = session::Model;
    type Account = account::Model;
    type Verification = verification::Model;

    fn model_declarations() -> &'static [better_auth::schema::ModelDeclaration] {
        &[]
    }
}
