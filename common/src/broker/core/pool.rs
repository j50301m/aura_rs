use amqprs::{
    channel::Channel,
    connection::{Connection, OpenConnectionArguments},
};
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::{Mutex, OwnedSemaphorePermit, Semaphore};
use tracing::{debug, error, info, warn};

#[derive(Debug, Clone)]
pub struct PoolConfig {
    pub max_connections: usize,
    pub min_connections: usize,
    pub idle_timeout: Option<std::time::Duration>,
    pub max_lifetime: Option<std::time::Duration>,
    pub health_check_on_acquire: bool,
    pub maintain_min_connections: bool,
    pub maintenance_interval: std::time::Duration,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            max_connections: 10,
            min_connections: 1,
            idle_timeout: Some(std::time::Duration::from_secs(600)), // 10 minutes
            max_lifetime: Some(std::time::Duration::from_secs(1800)), // 30 minutes
            health_check_on_acquire: true,
            maintain_min_connections: true,
            maintenance_interval: std::time::Duration::from_secs(30), // 30 seconds
        }
    }
}

pub struct ConnectionPool {
    connections: Arc<Mutex<Vec<PooledConnection>>>,
    semaphore: Arc<Semaphore>,
    config: PoolConfig,
    connection_args: OpenConnectionArguments,
    shutdown: Arc<tokio::sync::RwLock<bool>>,
}

struct PooledConnection {
    connection: Connection,
    created_at: std::time::Instant,
    last_used: std::time::Instant,
}

impl PooledConnection {
    fn new(connection: Connection) -> Self {
        let now = std::time::Instant::now();
        Self {
            connection,
            created_at: now,
            last_used: now,
        }
    }

    fn is_expired(&self, config: &PoolConfig) -> bool {
        let now = std::time::Instant::now();

        // Check max lifetime
        if let Some(max_lifetime) = config.max_lifetime {
            if now.duration_since(self.created_at) > max_lifetime {
                debug!("Connection expired due to max lifetime");
                return true;
            }
        }

        // Check idle timeout
        if let Some(idle_timeout) = config.idle_timeout {
            if now.duration_since(self.last_used) > idle_timeout {
                debug!("Connection expired due to idle timeout");
                return true;
            }
        }

        false
    }

    fn is_healthy(&self) -> bool {
        // Check connection status
        if !self.connection.is_open() {
            debug!("Connection is closed");
            return false;
        }

        true
    }

    fn touch(&mut self) {
        self.last_used = std::time::Instant::now();
    }
}

impl ConnectionPool {
    pub async fn new(connection_args: OpenConnectionArguments, config: PoolConfig) -> Result<Self> {
        let pool = Self {
            connections: Arc::new(Mutex::new(Vec::new())),
            semaphore: Arc::new(Semaphore::new(config.max_connections)),
            shutdown: Arc::new(tokio::sync::RwLock::new(false)),
            config,
            connection_args,
        };

        // Initialize minimum connections
        pool.initialize_min_connections().await?;

        pool.start_maintainer();

        info!(
            "Connection pool initialized with min: {}, max: {}",
            pool.config.min_connections, pool.config.max_connections
        );

        Ok(pool)
    }

    fn start_maintainer(&self) {
        let pool = self.clone();
        let interval = self.config.maintenance_interval;
        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(interval);
            loop {
                ticker.tick().await;

                if *pool.shutdown.read().await {
                    break; // Already shutdown
                }

                pool.maintenance().await;
            }
        });
    }

    async fn maintenance(&self) {
        debug!("Running pool maintenance task");

        // Cleanup phase - hold lock
        {
            let mut connections = self.connections.lock().await;
            connections.retain(|conn| {
                if conn.is_expired(&self.config) {
                    debug!("Removing expired connection in maintainer");
                    return false;
                }
                if !conn.is_healthy() {
                    debug!("Removing unhealthy connection in maintainer");
                    return false;
                }
                true
            });
        }

        // Maintenance phase - no lock held
        if self.config.maintain_min_connections {
            self.ensure_min_connections().await;
        }
    }

    async fn initialize_min_connections(&self) -> Result<()> {
        let mut connections = self.connections.lock().await;

        for _ in 0..self.config.min_connections {
            match self.create_connection().await {
                Ok(conn) => {
                    connections.push(PooledConnection::new(conn));
                    debug!("Created initial connection for pool");
                }
                Err(e) => {
                    error!("Failed to create initial connection: {:?}", e);
                    return Err(e);
                }
            }
        }

        Ok(())
    }

    async fn create_connection(&self) -> Result<Connection> {
        debug!("Creating new AMQP connection");
        let connection = Connection::open(&self.connection_args).await?;
        Ok(connection)
    }

    pub async fn get_connection(&self) -> Result<PoolGuard> {
        // Check if pool is shutting down
        if *self.shutdown.read().await {
            return Err(anyhow::anyhow!("Connection pool is shutting down"));
        }

        // Acquire semaphore permit
        let permit = Arc::clone(&self.semaphore).acquire_owned().await.unwrap();

        // Try to get an existing connection
        let connection = {
            let mut connections = self.connections.lock().await;

            // Clean up expired and unhealthy connections
            connections.retain(|conn| {
                if conn.is_expired(&self.config) {
                    debug!("Removing expired connection");
                    return false;
                }

                if self.config.health_check_on_acquire && !conn.is_healthy() {
                    debug!("Removing unhealthy connection");
                    return false;
                }

                true
            });

            // Try to reuse an existing connection
            if let Some(mut pooled_conn) = connections.pop() {
                pooled_conn.touch();
                debug!("Reusing existing connection from pool");
                pooled_conn.connection
            } else {
                // Create new connection
                debug!("Creating new connection for pool");
                self.create_connection().await?
            }
        };

        Ok(PoolGuard {
            connection: Some(connection),
            pool: self.clone(),
            _permit: permit,
        })
    }

    async fn return_connection(&self, connection: Connection) {
        if *self.shutdown.read().await {
            debug!("Pool is shutting down, dropping connection");
            return;
        }

        if !connection.is_open() {
            debug!("Not returning closed connection to pool");
            return;
        }

        // Return connection to pool
        let returned = {
            let mut connections = self.connections.lock().await;
            if connections.len() < self.config.max_connections {
                connections.push(PooledConnection::new(connection));
                debug!("Returned connection to pool");
                true
            } else {
                debug!("Pool full, dropping connection");
                false
            }
        };

        // Only execute maintenance if successfully returned and needed
        if returned && self.config.maintain_min_connections {
            self.ensure_min_connections().await;
        }
    }

    async fn ensure_min_connections(&self) {
        let connections_count = {
            let connections = self.connections.lock().await;
            connections.len()
        };

        let needed = self
            .config
            .min_connections
            .saturating_sub(connections_count);
        if needed > 0 {
            debug!(
                "Need to create {} more connections to maintain minimum",
                needed
            );

            for _ in 0..needed {
                match self.create_connection().await {
                    Ok(conn) => {
                        let mut connections = self.connections.lock().await;
                        if connections.len() < self.config.min_connections {
                            connections.push(PooledConnection::new(conn));
                            debug!("Created connection to maintain minimum pool size");
                        } else {
                            break; // Someone else already added connections
                        }
                    }
                    Err(e) => {
                        warn!("Failed to create connection for minimum pool size: {:?}", e);
                        break;
                    }
                }
            }
        }
    }

    pub async fn get_channel(&self) -> Result<ChannelGuard> {
        let conn_guard = self.get_connection().await?;
        let channel = conn_guard.open_channel(None).await?;

        Ok(ChannelGuard {
            channel,
            _connection_guard: conn_guard,
        })
    }

    pub async fn shutdown(&self) {
        info!("Shutting down connection pool");

        // Mark as shutting down
        *self.shutdown.write().await = true;

        // Close all connections
        let mut connections = self.connections.lock().await;
        for pooled_conn in connections.drain(..) {
            if let Err(e) = pooled_conn.connection.close().await {
                warn!("Error closing connection during shutdown: {:?}", e);
            }
        }

        info!("Connection pool shutdown complete");
    }

    pub fn stats(&self) -> PoolStats {
        let available_permits = self.semaphore.available_permits();
        let active_connections = self.config.max_connections - available_permits;

        PoolStats {
            active_connections,
            available_permits,
            max_connections: self.config.max_connections,
        }
    }
}

impl Clone for ConnectionPool {
    fn clone(&self) -> Self {
        Self {
            connections: Arc::clone(&self.connections),
            semaphore: Arc::clone(&self.semaphore),
            config: self.config.clone(),
            connection_args: self.connection_args.clone(),
            shutdown: Arc::clone(&self.shutdown),
        }
    }
}

pub struct PoolGuard {
    connection: Option<Connection>,
    pool: ConnectionPool,
    _permit: OwnedSemaphorePermit,
}

impl std::ops::Deref for PoolGuard {
    type Target = Connection;

    fn deref(&self) -> &Self::Target {
        self.connection
            .as_ref()
            .expect("Connection was already returned to pool")
    }
}

impl Drop for PoolGuard {
    fn drop(&mut self) {
        if let Some(connection) = self.connection.take() {
            let pool = self.pool.clone();

            // Check if tokio runtime is available
            match tokio::runtime::Handle::try_current() {
                Ok(_handle) => {
                    tokio::spawn(async move {
                        pool.return_connection(connection).await;
                    });
                }
                Err(_) => {
                    warn!("No tokio runtime available, cannot return connection to pool");
                    // Connection will be dropped here, won't return to pool
                }
            }
        }
    }
}

pub struct ChannelGuard {
    pub channel: Channel,
    _connection_guard: PoolGuard,
}

impl ChannelGuard {
    /// Check if channel is still healthy
    pub fn is_healthy(&self) -> bool {
        // AMQP channel health check
        self.channel.is_open()
    }
}

impl std::ops::Deref for ChannelGuard {
    type Target = Channel;

    fn deref(&self) -> &Self::Target {
        &self.channel
    }
}

#[derive(Debug)]
pub struct PoolStats {
    pub active_connections: usize,
    pub available_permits: usize,
    pub max_connections: usize,
}

// Add graceful shutdown Drop implementation
impl Drop for ConnectionPool {
    fn drop(&mut self) {
        // Attempt graceful shutdown when pool is dropped
        if let Ok(handle) = tokio::runtime::Handle::try_current() {
            let shutdown_flag = Arc::clone(&self.shutdown);
            let connections = Arc::clone(&self.connections);

            handle.spawn(async move {
                *shutdown_flag.write().await = true;
                let mut conns = connections.lock().await;
                for pooled_conn in conns.drain(..) {
                    let _ = pooled_conn.connection.close().await;
                }
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::Duration;

    fn test_connection_args() -> OpenConnectionArguments {
        OpenConnectionArguments::new("localhost", 5672, "guest", "guest")
    }

    async fn create_test_pool() -> Result<ConnectionPool> {
        let config = PoolConfig {
            max_connections: 3,
            min_connections: 1,
            idle_timeout: Some(Duration::from_secs(10)),
            max_lifetime: Some(Duration::from_secs(30)),
            health_check_on_acquire: true,
            maintain_min_connections: true,
            maintenance_interval: Duration::from_secs(5),
        };
        ConnectionPool::new(test_connection_args(), config).await
    }

    #[tokio::test]
    async fn test_pool_creation_and_stats() -> Result<()> {
        let pool = create_test_pool().await?;
        let stats = pool.stats();

        assert_eq!(stats.max_connections, 3);
        assert_eq!(stats.active_connections, 0);
        assert_eq!(stats.available_permits, 3);

        Ok(())
    }

    #[tokio::test]
    async fn test_get_and_return_connection() -> Result<()> {
        let pool = create_test_pool().await?;

        // Test getting a connection
        {
            let _conn_guard = pool.get_connection().await?;
            let stats = pool.stats();
            assert_eq!(stats.active_connections, 1);
            assert_eq!(stats.available_permits, 2);
            // conn_guard is dropped here and the connection should be returned
        }

        // Wait a bit for async return to complete
        tokio::time::sleep(Duration::from_millis(10)).await;

        let stats = pool.stats();
        assert_eq!(stats.active_connections, 0);
        assert_eq!(stats.available_permits, 3);

        Ok(())
    }

    #[tokio::test]
    async fn test_channel_health_check() -> Result<()> {
        let pool = create_test_pool().await?;
        let chan_guard = pool.get_channel().await?;

        // Newly created channel should be healthy
        assert!(chan_guard.is_healthy());

        Ok(())
    }

    #[tokio::test]
    async fn test_pool_shutdown() -> Result<()> {
        let pool = create_test_pool().await?;

        // Get a connection
        let _conn = pool.get_connection().await?;

        // Shutdown pool
        pool.shutdown().await;

        // Attempting to get a new connection should fail
        let result = pool.get_connection().await;
        assert!(result.is_err());

        Ok(())
    }

    #[tokio::test]
    async fn test_maintain_min_connections() -> Result<()> {
        let config = PoolConfig {
            max_connections: 5,
            min_connections: 2,
            maintain_min_connections: true,
            ..Default::default()
        };

        let pool = ConnectionPool::new(test_connection_args(), config).await?;

        // Initially should have min_connections connections
        {
            let connections = pool.connections.lock().await;
            assert!(connections.len() >= 2);
        }

        Ok(())
    }

    // Other tests remain unchanged...
    #[tokio::test]
    async fn test_multiple_connections() -> Result<()> {
        let pool = create_test_pool().await?;

        // Get multiple connections simultaneously
        let conn1 = pool.get_connection().await?;
        let conn2 = pool.get_connection().await?;
        let conn3 = pool.get_connection().await?;

        let stats = pool.stats();
        assert_eq!(stats.active_connections, 3);
        assert_eq!(stats.available_permits, 0);

        // Test that the 4th connection will wait
        let pool_clone = pool.clone();
        let handle = tokio::spawn(async move { pool_clone.get_connection().await });

        // Wait a bit to ensure the 4th request is waiting
        tokio::time::sleep(Duration::from_millis(10)).await;

        // Release a connection
        drop(conn1);
        tokio::time::sleep(Duration::from_millis(10)).await;

        // Now the 4th connection should be available
        let _conn4 = handle.await??;

        drop(conn2);
        drop(conn3);

        Ok(())
    }

    #[tokio::test]
    async fn test_maintainer_refills_connections() -> Result<()> {
        let config = PoolConfig {
            max_connections: 5,
            min_connections: 2,
            maintenance_interval: Duration::from_millis(50),
            ..Default::default()
        };

        let pool = ConnectionPool::new(test_connection_args(), config).await?;

        {
            let mut connections = pool.connections.lock().await;
            connections.clear(); // Simulate all connections lost
        }

        // Wait for maintainer to refill
        tokio::time::sleep(Duration::from_millis(120)).await;

        let connections = pool.connections.lock().await;
        assert!(
            connections.len() >= 2,
            "maintainer didn't fill up to min_connections"
        );

        Ok(())
    }
}
