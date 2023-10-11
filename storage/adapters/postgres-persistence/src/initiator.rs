use std::ops::Deref;
use std::sync::Arc;

use sea_orm::{ConnectionTrait, Database, DatabaseConnection, DbErr};
use sea_orm_migration::MigratorTrait;
use tracing::{error, warn};

use crate::migrations::Migrator;

pub async fn init_db(url: &str, db_name: &str) -> Arc<DatabaseConnection> {
    let db = Database::connect(format!("{}/postgres", &url))
        .await
        .expect(" ▸ storage: Error during connecting to database");
    db.execute_unprepared(&format!("CREATE DATABASE {};", db_name))
        .await
        .map_err(|err| match err {
            DbErr::Exec(err) => warn!("{}", err),
            err => error!("{}", err),
        })
        .unwrap();

    let db = Arc::new(
        Database::connect(format!("{}/{}", url, db_name))
            .await
            .unwrap_or_else(|_| {
                panic!(" ▸ storage: Error during connecting to '{db_name}' database")
            }),
    );
    Migrator::up(db.deref(), None)
        .await
        .expect(" ▸ storage: Failed apply db migrations");
    db
}
