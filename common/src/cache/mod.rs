pub mod distributed_lock;

use anyhow::{Context, Result};
use redis::Client;
use std::sync::Arc;

pub use distributed_lock::DistributedLockGuard;

/// Cache manager that provides Redis client and distributed lock functionality
pub struct Cache {
    redis_client: Arc<Client>,
}

impl Cache {
    /// Initialize Cache with Redis URL
    pub async fn new(redis_url: &str) -> Result<Self> {
        tracing::info!("Initializing Cache with Redis URL: {}", redis_url);

        let client = Client::open(redis_url)
            .with_context(|| format!("Failed to create Redis client with URL: {}", redis_url))?;

        // Test the connection
        let mut conn = client
            .get_connection()
            .with_context(|| "Failed to get Redis connection")?;

        // Ping the server to ensure connection is valid
        redis::cmd("PING")
            .query::<String>(&mut conn)
            .with_context(|| "Failed to ping Redis server")?;

        Ok(Self {
            redis_client: Arc::new(client),
        })
    }

    /// Get a reference to the Redis client (for immediate, short-term use)
    pub fn redis_client(&self) -> &Client {
        &self.redis_client
    }

    /// Get a cloned Arc of the Redis client (for async operations, multi-threading, or independent lifetime)
    pub fn redis_client_arc(&self) -> Arc<Client> {
        Arc::clone(&self.redis_client)
    }

    /// Try to acquire a distributed lock with automatic cleanup.
    ///
    /// This is the **recommended way** to use distributed locks. It returns a guard
    /// that automatically releases the lock when dropped, ensuring proper cleanup
    /// even in case of panics or early returns.
    ///
    /// # Arguments
    ///
    /// * `lock_key` - Unique identifier for the lock (use `generate_lock_key()`)
    /// * `lock_value` - Unique value for this lock holder (use `generate_lock_value()`)
    /// * `ttl_seconds` - Lock expiry time for safety (e.g., 30 seconds)
    ///
    /// # Returns
    ///
    /// * `Ok(Some(guard))` - Lock acquired successfully, guard will auto-release
    /// * `Ok(None)` - Lock is busy (held by another process), try again later
    /// * `Err(e)` - Redis connection or network error
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use common::cache::Cache;
    /// # async fn example() -> anyhow::Result<()> {
    /// let cache = Cache::new("redis://localhost:6379").await?;
    ///
    /// let lock_key = cache.generate_lock_key("draw", "turbo_togel_1");
    /// let lock_value = cache.generate_lock_value();
    ///
    /// if let Some(guard) = cache.try_acquire_lock_guard(
    ///     lock_key,
    ///     lock_value,
    ///     30, // 30 seconds TTL
    /// ).await? {
    ///     // Critical section - only one process can execute this
    ///     // perform_drawing_logic().await?;
    ///     // Lock automatically released when guard drops
    /// } else {
    ///     println!("Another process is already drawing, skipping...");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Multi-Pod Safety
    ///
    /// This method is designed for Kubernetes multi-pod environments where
    /// multiple instances might try to perform the same operation simultaneously.
    /// Only one pod will successfully acquire the lock and proceed.
    pub async fn try_acquire_lock_guard(
        &self,
        lock_key: String,
        lock_value: String,
        ttl_seconds: u64,
    ) -> Result<Option<distributed_lock::DistributedLockGuard>> {
        distributed_lock::DistributedLockGuard::try_acquire(
            (*self.redis_client).clone(),
            lock_key,
            lock_value,
            ttl_seconds,
        )
        .await
    }

    /// Generate a standardized lock key for a specific resource.
    ///
    /// This creates a consistent lock key format that helps avoid conflicts
    /// and makes debugging easier.
    ///
    /// # Arguments
    ///
    /// * `prefix` - Category or type of operation (e.g., "draw", "user_action")
    /// * `resource_id` - Specific resource identifier (e.g., "turbo_togel_1", "user_123")
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use common::cache::Cache;
    /// # async fn example() -> anyhow::Result<()> {
    /// # let cache = Cache::new("redis://localhost:6379").await?;
    /// let lock_key = cache.generate_lock_key("draw", "turbo_togel_1");
    /// // Result: "lock:draw:turbo_togel_1"
    ///
    /// let user_lock = cache.generate_lock_key("user_action", "user_456");
    /// // Result: "lock:user_action:user_456"
    /// # Ok(())
    /// # }
    /// ```
    pub fn generate_lock_key(&self, prefix: &str, resource_id: &str) -> String {
        format!("lock:{}:{}", prefix, resource_id)
    }
}
