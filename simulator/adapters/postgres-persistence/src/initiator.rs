use std::ops::Deref;
use std::sync::Arc;

use sea_orm::{ConnectionTrait, Database, DatabaseConnection, DbErr};
use sea_orm_migration::MigratorTrait;
use tracing::{error, warn};

use crate::migrations::Migrator;

pub async fn init_db(url: &str, db_name: &str) -> Arc<DatabaseConnection> {
    let db = Database::connect(format!("{}/postgres", &url))
        .await
        .expect(" ▸ simulator: Error during connecting to database");
    let _ = db
        .execute_unprepared(&format!("CREATE DATABASE {};", db_name))
        .await
        .map_err(|err| match err {
            DbErr::Exec(err) => warn!("{}", err),
            err => error!("{}", err),
        });

    let db = Arc::new(
        Database::connect(format!("{}/{}", url, db_name))
            .await
            .expect(&format!(
                " ▸ simulator: Error during connecting to '{db_name}' database"
            )),
    );
    Migrator::up(db.deref(), None)
        .await
        .expect(" ▸ simulator: Failed apply db migrations");
    db
}
