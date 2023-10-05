use sea_orm_migration::{MigrationTrait, MigratorTrait};

use crate::migrations::m20231005_000001_create_tables;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![Box::new(m20231005_000001_create_tables::Migration)]
    }
}
