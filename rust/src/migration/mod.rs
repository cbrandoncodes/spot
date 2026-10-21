use sea_orm_migration::prelude::*;

mod m20261021_000001_auth_foundation;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![Box::new(m20261021_000001_auth_foundation::Migration)]
    }
}
