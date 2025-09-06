use std::vec;
use sea_orm_migration::MigrationTrait;

mod common;
mod dev;
mod local;
mod prod;
mod stg;



fn common_seed_migrations() -> Vec<Box<dyn MigrationTrait>> {
    vec![
        Box::new(common::m20250905_164736_seed_draw_shedule::Migration),
    ]
}

pub fn local_seed_migrations() -> Vec<Box<dyn MigrationTrait>> {
    let mut migrations = common_seed_migrations();

    // Add local-specific seed migrations here if any
    migrations
}

pub fn dev_seed_migrations() -> Vec<Box<dyn MigrationTrait>> {
    let mut migrations = common_seed_migrations();

    // Add dev-specific seed migrations here if any
    migrations
}

pub fn stg_seed_migrations() -> Vec<Box<dyn MigrationTrait>> {
    let mut migrations = common_seed_migrations();

    // Add staging-specific seed migrations here if any
    migrations
}

pub fn prod_seed_migrations() -> Vec<Box<dyn MigrationTrait>> {
    let mut migrations = common_seed_migrations();

    // Add production-specific seed migrations here if any
    migrations
}