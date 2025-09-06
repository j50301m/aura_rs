use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;


#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.create_table(
            Table::create()
                .table(TurboTogelDrawResult::Table)
                .if_not_exists()
                .col(
                    ColumnDef::new(TurboTogelDrawResult::GameId)
                        .big_unsigned()
                        .not_null()
                )
                .col(
                    ColumnDef::new(TurboTogelDrawResult::Period)
                        .big_unsigned()
                        .not_null()
                )
                .col(
                    ColumnDef::new(TurboTogelDrawResult::Numbers)
                        .string_len(64)
                        .not_null()
                )
                .col(
                    ColumnDef::new(TurboTogelDrawResult::Remark)
                        .string_len(50)
                        .null()
                        .comment("if remark is not null, it means created by force-tool")
                )
                .col(
                    ColumnDef::new(TurboTogelDrawResult::CreatedAt)
                        .timestamp_with_time_zone()
                        .default(Expr::current_timestamp())
                        .null()
                )
                .col(
                    ColumnDef::new(TurboTogelDrawResult::DrawingAt)
                        .timestamp_with_time_zone()
                        .null()
                )
                .col(
                    ColumnDef::new(TurboTogelDrawResult::UpdatedAt)
                        .timestamp_with_time_zone()
                        .null()
                )
                .col(
                    ColumnDef::new(TurboTogelDrawResult::UpdatedBy)
                        .string_len(50)
                        .null()
                )
                .col(
                    ColumnDef::new(TurboTogelDrawResult::IsBroadcasted)
                        .boolean()
                        .default(false)
                        .not_null()
                )
                .primary_key(
                    Index::create()
                        .col(TurboTogelDrawResult::GameId)
                        .col(TurboTogelDrawResult::Period)
                )
                .to_owned(),
        ).await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(
                Table::drop()
                    .table(TurboTogelDrawResult::Table)
                    .cascade()
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum TurboTogelDrawResult {
    Table,
    GameId,
    Period,
    Numbers,
    Remark,
    CreatedAt,
    DrawingAt,
    UpdatedAt,
    UpdatedBy,
    IsBroadcasted,
}


