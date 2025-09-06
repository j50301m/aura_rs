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
                            .big_unsigned()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(TurboTogelDrawShedule::Cron)
                            .string()
                            .not_null()
                            .comment("cron expression ex: '0 0/5 * * * ?'"),
                    )
                    .col(
                        ColumnDef::new(TurboTogelDrawShedule::Location)
                            .string()
                            .not_null()
                            .comment("format: Asia/Jakarta"),
                    )
                    .col(
                        ColumnDef::new(TurboTogelDrawShedule::FirstDraw)
                            .string_len(8)
                            .not_null()
                            .comment("format: hh:MM:ss"),
                    )
                    .col(
                        ColumnDef::new(TurboTogelDrawShedule::Interval)
                            .small_integer()
                            .not_null()
                            .comment("the interval in seconds between each draw"),
                    )
                    .col(
                        ColumnDef::new(TurboTogelDrawShedule::Close)
                            .small_integer()
                            .not_null()
                            .comment("the last N seconds before lottery draw, forbid betting"),
                    )
                    .col(
                        ColumnDef::new(TurboTogelDrawShedule::Min)
                            .small_integer()
                            .not_null()
                            .comment("the min number for the game"),
                    )
                    .col(
                        ColumnDef::new(TurboTogelDrawShedule::Max)
                            .small_integer()
                            .not_null()
                            .comment("the max number for the game"),
                    )
                    .col(
                        ColumnDef::new(TurboTogelDrawShedule::Count)
                            .small_integer()
                            .not_null()
                            .comment("the digit count for the game"),
                    )
                    .col(
                        ColumnDef::new(TurboTogelDrawShedule::Repeatable)
                            .boolean()
                            .not_null()
                            .comment("whether the numbers can be repeated, false: no, true: yes"),
                    )
                    .col(
                        ColumnDef::new(TurboTogelDrawShedule::Status)
                            .small_integer()
                            .not_null()
                            .default(1)
                            .comment("0: inactive, 1: active"),
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
