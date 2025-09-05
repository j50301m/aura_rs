pub use sea_orm_migration::prelude::*;

pub mod common;

pub struct Common;

#[async_trait::async_trait]
impl MigratorTrait for Common {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![Box::new(
            common::m20250905_113743_create_turbo_togel_draw_shedule_table::Migration,
        )]
    }
}
