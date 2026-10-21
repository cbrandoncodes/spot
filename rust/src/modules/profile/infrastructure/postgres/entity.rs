use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "profiles")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub user_id: Uuid,
    pub name: String,
    pub username: Option<String>,
    pub username_normalized: Option<String>,
    pub avatar_key: Option<String>,
    pub bio: Option<String>,
    pub country_code: Option<String>,
    pub city: Option<String>,
    pub district: Option<String>,
    pub visibility: String,
    pub discoverable_by_username: bool,
    pub show_primary_gym: bool,
    pub discoverable_in_gym: bool,
    pub show_district: bool,
    pub show_activity: bool,
    pub show_training_presence: bool,
    pub allow_gym_friend_requests: bool,
    pub show_body_metrics: bool,
    pub friend_request_policy: String,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
