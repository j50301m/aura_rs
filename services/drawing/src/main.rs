use std::sync::Arc;

use common::cache;
use serde_json::de;

mod config;
mod turbo_corn;

#[tokio::main]
async fn main() {
    common::tracing::init();

    let cfg = config::Config::new();

    let db = Arc::new(
        common::db::DbInitializer::new(&cfg.db.db_url)
            .max_connections(cfg.db.db_max_connections)
            .min_connections(cfg.db.db_min_connections)
            .connect_timeout(cfg.db.db_connect_timeout)
            .idle_timeout(cfg.db.db_idle_timeout)
            .max_lifetime(cfg.db.db_max_lifetime)
            .sqlx_logging(cfg.db.db_logging)
            .schema(&cfg.db.db_schema)
            .connect()
            .await
            .expect("Failed to connect to the database"),
    );

    let cache = Arc::new(
        cache::Cache::new(&cfg.redis_url)
            .await
            .expect("Failed to connect to Redis"),
    );

    // TODO: Use trait object for broker
    let _broker = common::broker::Broker::new(common::broker::OpenConnectionArguments::new(
        &cfg.rabbitmq.host,
        cfg.rabbitmq.port,
        &cfg.rabbitmq.username,
        &cfg.rabbitmq.password,
    ))
    .await
    .expect("Failed to connect to RabbitMQ");

    // Create and start scheduler
    let scheduler = turbo_corn::Scheduler::new(db, cache);
    scheduler.start().await.expect("Failed to start scheduler");

    tracing::info!("Drawing service started. Press Ctrl+C to stop.");

    // Wait for shutdown signal
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to listen for shutdown signal");

    tracing::info!("Shutdown signal received, exiting gracefully...");
}
