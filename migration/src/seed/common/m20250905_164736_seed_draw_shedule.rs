use common::entity::turbo_togel_draw_shedule;
use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::{entity::*, query::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Get the connection and start a transaction
        let db = manager.get_connection();
        let transaction = db.begin().await?;

        turbo_togel_draw_shedule::ActiveModel {
            id: Set(1),
            cron: Set("0,5,10,15,20,25,30,35,40,45,50,55 * * * ?".to_string()),
            location: Set("Asia/Jakarta".to_string()),
            first_draw: Set("00:00:00".to_string()),
            interval: Set(300),
            close: Set(30),
            min: Set(1),  // Changed from 0 to 1 to satisfy min > 0 constraint
            max: Set(9),
            count: Set(4),
            repeatable: Set(true),
            status: Set(1),
        }
        .insert(&transaction)
        .await?;

        transaction.commit().await?;

        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        // Do nothing on down
        Ok(())
    }
}
