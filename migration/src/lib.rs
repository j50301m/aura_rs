pub use sea_orm_migration::prelude::*;
mod common;
mod local;

pub struct Migrator;
pub struct Local;
pub struct Dev;
pub struct Stg;
pub struct Prod;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(common::m20250905_113743_create_turbo_togel_draw_shedule_table::Migration,
            ),
            Box::new(common::m20250906_063502_create_turbo_togel_draw_result_table::Migration),
        ]
    }
}

#[async_trait::async_trait]
impl MigratorTrait for Local {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        let mut migrations = Migrator::migrations();
        let mut seeds: Vec<Box<dyn MigrationTrait>> = vec![Box::new(
            local::m20250905_164736_seed_draw_shedule::Migration,
        )];

        migrations.append(&mut seeds);
        migrations
    }
}

#[async_trait::async_trait]
impl MigratorTrait for Dev {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        let mut migrations = vec![];
        migrations.extend(Local::migrations());
        migrations
    }
}

#[async_trait::async_trait]
impl MigratorTrait for Stg {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        let mut migrations = vec![];
        migrations.extend(Local::migrations());
        migrations
    }
}

#[async_trait::async_trait]
impl MigratorTrait for Prod {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        let mut migrations = vec![];
        migrations.extend(Migrator::migrations());
        migrations
    }
}
