use sea_orm_migration::prelude::*;

use storage_postgres_persistence::migrations::Migrator;

#[async_std::main]
async fn main() {
    cli::run_cli(Migrator).await;
}
