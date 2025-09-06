pub use sea_orm_migration::prelude::*;
mod ddl;
mod seed;

pub struct TableMigrator;
pub struct LocalSeed;
pub struct DevSeed;
pub struct StgSeed;
pub struct ProdSeed;

#[async_trait::async_trait]
impl MigratorTrait for TableMigrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(ddl::m20250905_113743_create_turbo_togel_draw_shedule_table::Migration,
            ),
            Box::new(ddl::m20250906_063502_create_turbo_togel_draw_result_table::Migration),
        ]
    }
}

#[async_trait::async_trait]
impl MigratorTrait for LocalSeed {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        let mut migrations = TableMigrator::migrations();
        migrations.extend(seed::local_seed_migrations());
        migrations
    }
}

#[async_trait::async_trait]
impl MigratorTrait for DevSeed {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        let mut migrations = vec![];
        migrations.extend(seed::dev_seed_migrations());
        migrations
    }
}

#[async_trait::async_trait]
impl MigratorTrait for StgSeed {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        let mut migrations = vec![];
        migrations.extend(seed::stg_seed_migrations());
        migrations
    }
}

#[async_trait::async_trait]
impl MigratorTrait for ProdSeed {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        let mut migrations = vec![];
        migrations.extend(seed::prod_seed_migrations());
        migrations
    }
}
