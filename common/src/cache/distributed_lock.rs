use anyhow::Result;
use redis::Client;
use tracing::{info, warn};

/// Internal distributed lock operations using Redis.
///
/// This struct is not exposed publicly. All distributed lock operations
/// should go through `Cache::try_acquire_lock_guard()` or other Cache methods
/// to ensure proper Redis client management and consistent API usage.
///
/// The lock implementation uses Redis SET command with NX (only if not exists)
/// and EX (expiry) options for atomic lock acquisition, and a Lua script
/// for safe lock release that checks ownership before deletion.
pub(super) struct DistributedLock;

impl DistributedLock {
    /// Try to acquire a distributed lock
    /// Returns true if lock was acquired, false otherwise
    pub async fn try_acquire_lock(
        client: &Client,
        lock_key: &str,
        lock_value: &str,
        ttl_seconds: u64,
    ) -> Result<bool> {
        let mut conn = client.get_multiplexed_async_connection().await?;

        // Use simple SET NX EX command
        let result: Option<String> = redis::cmd("SET")
            .arg(lock_key)
            .arg(lock_value)
            .arg("NX") // Only set if not exists
            .arg("EX") // Set expiry
            .arg(ttl_seconds)
            .query_async(&mut conn)
            .await?;

        let acquired = result.is_some();

        if acquired {
            info!("Successfully acquired lock: {}", lock_key);
        } else {
            info!("Failed to acquire lock: {} (already held)", lock_key);
        }

        Ok(acquired)
    }

    /// Release a distributed lock (only if we own it)
    pub async fn release_lock(client: &Client, lock_key: &str, lock_value: &str) -> Result<bool> {
        let mut conn = client.get_multiplexed_async_connection().await?;

        // Lua script to atomically check and delete the lock
        // Only delete if the value matches (we own the lock)
        let lua_script = r#"
            if redis.call("GET", KEYS[1]) == ARGV[1] then
                return redis.call("DEL", KEYS[1])
            else
                return 0
            end
        "#;

        let result: i32 = redis::cmd("EVAL")
            .arg(lua_script)
            .arg(1) // Number of keys
            .arg(lock_key) // KEYS[1]
            .arg(lock_value) // ARGV[1]
            .query_async(&mut conn)
            .await?;

        let released = result == 1;

        if released {
            info!("Successfully released lock: {}", lock_key);
        } else {
            warn!("Failed to release lock: {} (not owned by us)", lock_key);
        }

        Ok(released)
    }
}

/// RAII wrapper for distributed lock that automatically releases the lock when dropped.
///
/// This guard ensures that distributed locks are properly released even if the code panics
/// or returns early. The lock is automatically released when the guard goes out of scope.
///
/// # Usage
///
/// ```no_run
/// # use common::cache::Cache;
/// # async fn example() -> anyhow::Result<()> {
/// // Initialize cache
/// let cache = Cache::new("redis://localhost:6379").await?;
///
/// // Try to acquire a lock with automatic cleanup
/// let lock_key = cache.generate_lock_key("draw", "turbo_togel_1");
/// let lock_value = cache.generate_lock_value();
///
/// if let Some(guard) = cache.try_acquire_lock_guard(
///     lock_key,
///     lock_value,
///     30, // TTL in seconds
/// ).await? {
///     // Lock acquired successfully, do your work here
///     println!("Lock acquired: {}", guard.lock_key());
///
///     // Perform critical section work...
///     // do_critical_work().await?;
///
///     // Lock is automatically released when guard goes out of scope
/// } else {
///     // Failed to acquire lock (already held by another process)
///     println!("Lock is busy, try again later");
/// }
/// # Ok(())
/// # }
/// ```
///
/// # Thread Safety
///
/// This guard can be safely moved between threads and tasks, as it uses `Client`
/// which is thread-safe through `Arc` internally.
///
/// # Automatic Cleanup
///
/// The lock is released in a background task when the guard is dropped, ensuring
/// that the current task doesn't block on the release operation.
pub struct DistributedLockGuard {
    client: Client,
    lock_key: String,
    lock_value: String,
}

impl DistributedLockGuard {
    /// Try to acquire a distributed lock with RAII guard.
    ///
    /// This method attempts to acquire a distributed lock and returns a guard
    /// that will automatically release the lock when dropped.
    ///
    /// # Arguments
    ///
    /// * `client` - Redis client (usually obtained from `Cache::redis_client_arc()`)
    /// * `lock_key` - Unique identifier for the lock (use `Cache::generate_lock_key()`)
    /// * `lock_value` - Unique value to identify this lock holder (use `Cache::generate_lock_value()`)
    /// * `ttl_seconds` - Time-to-live for the lock in seconds (safety mechanism)
    ///
    /// # Returns
    ///
    /// * `Ok(Some(guard))` - Lock was successfully acquired
    /// * `Ok(None)` - Lock is already held by another process
    /// * `Err(e)` - Network or Redis error occurred
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use common::cache::{Cache, DistributedLockGuard};
    /// # async fn example() -> anyhow::Result<()> {
    /// # let cache = Cache::new("redis://localhost:6379").await?;
    /// // This is typically called via Cache::try_acquire_lock_guard()
    /// // Don't call this directly unless you have a specific need
    ///
    /// let client = (*cache.redis_client_arc()).clone();
    /// let guard = DistributedLockGuard::try_acquire(
    ///     client,
    ///     "lock:draw:game_1".to_string(),
    ///     uuid::Uuid::new_v4().to_string(),
    ///     30,
    /// ).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn try_acquire(
        client: Client,
        lock_key: String,
        lock_value: String,
        ttl_seconds: u64,
    ) -> Result<Option<Self>> {
        let acquired =
            DistributedLock::try_acquire_lock(&client, &lock_key, &lock_value, ttl_seconds).await?;

        if acquired {
            Ok(Some(Self {
                client,
                lock_key,
                lock_value,
            }))
        } else {
            Ok(None)
        }
    }

    /// Get the lock key that this guard is protecting.
    ///
    /// This can be useful for logging or debugging purposes.
    pub fn lock_key(&self) -> &str {
        &self.lock_key
    }

    /// Get the unique lock value that identifies this lock holder.
    ///
    /// This value is used to ensure that only the process that acquired
    /// the lock can release it.
    pub fn lock_value(&self) -> &str {
        &self.lock_value
    }
}

impl Drop for DistributedLockGuard {
    /// Automatically release the distributed lock when the guard is dropped.
    ///
    /// This ensures that locks are always released, even if:
    /// - The code panics
    /// - An early return occurs
    /// - The guard goes out of scope normally
    ///
    /// The lock release is performed in a background task to avoid blocking
    /// the current thread. If the release fails, it will be logged as a warning.
    ///
    /// Note: The TTL mechanism in Redis provides additional safety - even if
    /// this cleanup fails, the lock will automatically expire.
    fn drop(&mut self) {
        // Release lock in background when guard is dropped
        let client = self.client.clone();
        let key = self.lock_key.clone();
        let value = self.lock_value.clone();

        tokio::spawn(async move {
            if let Err(e) = DistributedLock::release_lock(&client, &key, &value).await {
                warn!("Failed to release lock on drop: {:?}", e);
            }
        });
    }
}
