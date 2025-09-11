mod config;
mod turbo_corn;

#[tokio::main]
async fn main() {
    let cfg = config::Config::new();

    let db = common::db::DbInitializer::new(&cfg.db.db_url)
        .max_connections(cfg.db.db_max_connections)
        .min_connections(cfg.db.db_min_connections)
        .connect_timeout(cfg.db.db_connect_timeout)
        .idle_timeout(cfg.db.db_idle_timeout)
        .max_lifetime(cfg.db.db_max_lifetime)
        .sqlx_logging(cfg.db.db_logging)
        .schema(&cfg.db.db_schema)
        .connect()
        .await
        .expect("Failed to connect to the database");

    // Create and start scheduler
    let _ = turbo_corn::Scheduler::new(db).start().await;
}
