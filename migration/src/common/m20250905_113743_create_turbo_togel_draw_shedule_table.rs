use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create table
        manager
            .create_table(
                Table::create()
                    .table(TurboTogelDrawShedule::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(TurboTogelDrawShedule::Id)
                            .integer()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(TurboTogelDrawShedule::Cron)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(TurboTogelDrawShedule::Location)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(TurboTogelDrawShedule::FirstDraw)
                            .string_len(8)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(TurboTogelDrawShedule::Interval)
                            .small_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(TurboTogelDrawShedule::Close)
                            .small_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(TurboTogelDrawShedule::Min)
                            .small_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(TurboTogelDrawShedule::Max)
                            .small_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(TurboTogelDrawShedule::Count)
                            .small_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(TurboTogelDrawShedule::Repeatable)
                            .small_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(TurboTogelDrawShedule::Status)
                            .small_integer()
                            .not_null()
                            .default(1),
                    )
                    .to_owned(),
            )
            .await?;

        // Add check constraint for status field using raw SQL
        let sql = "ALTER TABLE turbo_togel_draw_shedule ADD CONSTRAINT turbo_togel_draw_shedule_status_check CHECK (status IN (0, 1))";
        manager.get_connection().execute_unprepared(sql).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop table
        manager
            .drop_table(
                Table::drop()
                    .table(TurboTogelDrawShedule::Table)
                    .cascade()
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum TurboTogelDrawShedule {
    Table,
    Id,
    Cron,
    Location,
    FirstDraw,
    Interval,
    Close,
    Min,
    Max,
    Count,
    Repeatable,
    Status,
}
