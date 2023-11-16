use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(SimulationReport::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(SimulationReport::SimulationId)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(SimulationReport::Start)
                            .timestamp()
                            .not_null(),
                    )
                    .col(ColumnDef::new(SimulationReport::End).timestamp().not_null())
                    .col(
                        ColumnDef::new(SimulationReport::Deployments)
                            .json()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(SimulationReport::Ticks)
                            .unsigned()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(SimulationReport::Actions)
                            .unsigned()
                            .not_null(),
                    )
                    .col(ColumnDef::new(SimulationReport::Profit).double().not_null())
                    .col(
                        ColumnDef::new(SimulationReport::ProfitClear)
                            .double()
                            .not_null(),
                    )
                    .col(ColumnDef::new(SimulationReport::Fees).double().not_null())
                    .col(ColumnDef::new(SimulationReport::Assets).json().not_null())
                    .col(
                        ColumnDef::new(SimulationReport::ActiveOrders)
                            .json()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(SimulationReport::SlCount)
                            .unsigned()
                            .not_null()
                    )
                    .col(
                        ColumnDef::new(SimulationReport::TpCount)
                            .unsigned()
                            .not_null()
                    )
                    .col(
                        ColumnDef::new(SimulationReport::SlPercent)
                            .double()
                            .not_null()
                    )
                    .col(
                        ColumnDef::new(SimulationReport::TpPercent)
                            .double()
                            .not_null()
                    )
                    .col(
                        ColumnDef::new(SimulationReport::MaxSlStreak)
                            .unsigned()
                            .not_null()
                    )
                    .col(
                        ColumnDef::new(SimulationReport::MaxTpStreak)
                            .unsigned()
                            .not_null()
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(SimulationReport::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(Iden)]
enum SimulationReport {
    Table,
    SimulationId,
    Start,
    End,
    Deployments,
    Ticks,
    Actions,
    Profit,
    ProfitClear,
    Fees,
    Assets,
    ActiveOrders,
    SlCount,
    TpCount,
    SlPercent,
    TpPercent,
    MaxSlStreak,
    MaxTpStreak
}
