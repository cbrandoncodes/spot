use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "fitness_profiles")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub user_id: Uuid,
    pub experience_level: String,
    pub current_activity_level: String,
    pub training_days_per_week: i16,
    pub session_duration_minutes: i16,
    pub training_environment: String,
    pub coaching_style: String,
    pub starting_intensity: String,
    pub consistency_barrier: Option<String>,
    pub unit_system: String,
    pub questionnaire_version: i32,
    pub revision: i64,
    pub health_data_consent_at: Option<DateTimeUtc>,
    pub privacy_notice_version: Option<String>,
    pub completed_at: DateTimeUtc,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
