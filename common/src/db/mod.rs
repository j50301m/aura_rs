use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use std::time::Duration;

/// A builder-style database connection initializer based on SeaORM's `ConnectOptions`.
///
/// # Features
/// - Chainable configuration for connection parameters (max/min connections, timeouts, lifetimes, etc.)
/// - Async/await connection support
/// - Optional SQLx logging
///
/// # Example
/// ```no_run
/// use common::db::DbInitializer;
/// # async fn run() -> Result<(), sea_orm::DbErr> {
/// let db = DbInitializer::new("postgres://user:pw@localhost/db")
///     .max_connections(100)
///     .min_connections(5)
///     .connect_timeout(8)
///     .idle_timeout(8)
///     .max_lifetime(8)
///     .sqlx_logging(true)
///     .schema("public")
///     .connect()
///     .await?;
/// // Use `db` for database operations
/// # Ok(())
/// # }
/// ```
pub struct DbInitializer {
    opt: ConnectOptions,
}

impl DbInitializer {
    pub fn new(db_url: &str) -> Self {
        Self {
            opt: ConnectOptions::new(db_url.to_owned()),
        }
    }

    pub fn max_connections(mut self, max: u32) -> Self {
        self.opt.max_connections(max);
        self
    }

    pub fn min_connections(mut self, min: u32) -> Self {
        self.opt.min_connections(min);
        self
    }

    pub fn connect_timeout(mut self, secs: u64) -> Self {
        self.opt.connect_timeout(Duration::from_secs(secs));
        self
    }

    pub fn idle_timeout(mut self, secs: u64) -> Self {
        self.opt.idle_timeout(Duration::from_secs(secs));
        self
    }

    pub fn max_lifetime(mut self, secs: u64) -> Self {
        self.opt.max_lifetime(Duration::from_secs(secs));
        self
    }

    pub fn sqlx_logging(mut self, enable: bool) -> Self {
        self.opt.sqlx_logging(enable);
        self
    }

    pub fn schema(mut self, schema: &str) -> Self {
        self.opt.set_schema_search_path(schema.to_owned());
        self
    }

    pub async fn connect(self) -> Result<DatabaseConnection, sea_orm::DbErr> {
        Database::connect(self.opt).await
    }
}
