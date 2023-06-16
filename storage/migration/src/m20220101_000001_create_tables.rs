use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.create_table(Table::create()
            .table(Order::Table)
            .if_not_exists()
            .col(ColumnDef::new(Order::Id)
                .string()
                .not_null()
                .primary_key())
            .col(ColumnDef::new(Order::Timestamp)
                .timestamp()
                .not_null())
            .col(ColumnDef::new(Order::SimulationId)
                .uuid())
            .col(ColumnDef::new(Order::Status)
                .json()
                .not_null())
            .col(ColumnDef::new(Order::Exchange)
                .string()
                .not_null())
            .col(ColumnDef::new(Order::Pair)
                .json()
                .not_null())
            .col(ColumnDef::new(Order::MarketType)
                .json()
                .not_null())
            .col(ColumnDef::new(Order::OrderType)
                .json()
                .not_null())
            .col(ColumnDef::new(Order::Side)
                .string()
                .not_null())
            .col(ColumnDef::new(Order::Size)
                .double()
                .not_null())
            .col(ColumnDef::new(Order::AvgPrice)
                .double()
                .not_null())
            .to_owned())
            .await?;

        manager.create_table(Table::create()
            .table(Position::Table)
            .if_not_exists()
            .col(ColumnDef::new(Position::Id)
                .string()
                .not_null()
                .primary_key())
            .col(ColumnDef::new(Position::SimulationId)
                .uuid())
            .col(ColumnDef::new(Position::Exchange)
                .string()
                .not_null())
            .col(ColumnDef::new(Position::Currency)
                .string()
                .not_null())
            .col(ColumnDef::new(Position::Side)
                .string()
                .not_null())
            .col(ColumnDef::new(Position::Size)
                .double()
                .not_null())
            .to_owned())
            .await?;

        manager.create_table(Table::create()
            .table(Candle::Table)
            .if_not_exists()
            .col(ColumnDef::new(Candle::Id)
                .string()
                .not_null()
                .primary_key())
            .col(ColumnDef::new(Candle::Status)
                .string()
                .not_null())
            .col(ColumnDef::new(Candle::Exchange)
                .string()
                .not_null())
            .col(ColumnDef::new(Candle::Pair)
                .json()
                .not_null())
            .col(ColumnDef::new(Candle::MarketType)
                .string()
                .not_null())
            .col(ColumnDef::new(Candle::Timestamp)
                .timestamp()
                .not_null())
            .col(ColumnDef::new(Candle::Timeframe)
                .string()
                .not_null())
            .col(ColumnDef::new(Candle::OpenPrice)
                .double()
                .not_null())
            .col(ColumnDef::new(Candle::HighestPrice)
                .double()
                .not_null())
            .col(ColumnDef::new(Candle::LowerPrice)
                .double()
                .not_null())
            .col(ColumnDef::new(Candle::ClosePrice)
                .double()
                .not_null())
            .col(ColumnDef::new(Candle::TargetVolume)
                .double()
                .not_null())
            .col(ColumnDef::new(Candle::SourceVolume)
                .double()
                .not_null())
            .to_owned())
            .await?;

        manager.create_table(Table::create()
            .table(Audit::Table)
            .if_not_exists()
            .col(ColumnDef::new(Audit::Id)
                .uuid()
                .not_null()
                .primary_key())
            .col(ColumnDef::new(Audit::Timestamp)
                .timestamp()
                .not_null())
            .col(ColumnDef::new(Audit::Tags)
                .json()
                .not_null())
            .col(ColumnDef::new(Audit::Event)
                .json()
                .not_null())
            .to_owned())
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop()
            .table(Order::Table).to_owned())
            .await?;

        manager.drop_table(Table::drop()
            .table(Position::Table).to_owned())
            .await?;

        manager.drop_table(Table::drop()
            .table(Candle::Table).to_owned())
            .await?;

        manager.drop_table(Table::drop()
            .table(Audit::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(Iden)]
enum Order {
    Table,
    Id,
    Timestamp,
    SimulationId,
    Status,
    Exchange,
    Pair,
    MarketType,
    OrderType,
    Side,
    Size,
    AvgPrice,
}

#[derive(Iden)]
enum Position {
    Table,
    Id,
    SimulationId,
    Exchange,
    Currency,
    Side,
    Size,
}

#[derive(Iden)]
enum Candle {
    Table,
    Id,
    Status,
    Exchange,
    Pair,
    MarketType,
    Timestamp,
    Timeframe,
    OpenPrice,
    HighestPrice,
    LowerPrice,
    ClosePrice,
    TargetVolume,
    SourceVolume,
}

#[derive(Iden)]
enum Audit {
    Table,
    Id,
    Timestamp,
    Tags,
    Event,
}
