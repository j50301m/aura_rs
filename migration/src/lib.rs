pub use sea_orm_migration::prelude::*;
mod common;
mod local;

pub struct Migrator;
pub struct LocalSeeds;
pub struct DevSeeds;
pub struct StgSeeds;
pub struct ProdSeeds;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![Box::new(
            common::m20250905_113743_create_turbo_togel_draw_shedule_table::Migration,
        )]
    }
}

#[async_trait::async_trait]
impl MigratorTrait for LocalSeeds {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        let mut migrations: Vec<Box<dyn MigrationTrait>> = vec![Box::new(
            local::m20250905_164736_seed_draw_shedule::Migration,
        )];

        migrations.extend(Migrator::migrations());
        migrations
    }
}

#[async_trait::async_trait]
impl MigratorTrait for DevSeeds {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![]
    }
}

#[async_trait::async_trait]
impl MigratorTrait for StgSeeds {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![]
    }
}

#[async_trait::async_trait]
impl MigratorTrait for ProdSeeds {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![]
    }
}
