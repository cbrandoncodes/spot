use better_auth::seaorm::sea_orm::{ConnectionTrait, DatabaseConnection};

pub async fn is_ready(database: &DatabaseConnection) -> Result<(), sea_orm::DbErr> {
    database.execute_unprepared("SELECT 1").await?;
    Ok(())
}
