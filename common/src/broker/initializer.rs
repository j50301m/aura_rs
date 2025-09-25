use amqprs::channel::Channel;

use super::core::*;
use anyhow::Result;

/// Re-export commonly used types
pub use amqprs::connection::OpenConnectionArguments;

pub struct Broker {
    pool: ConnectionPool,
}

impl Broker {
    pub async fn new(args: OpenConnectionArguments) -> Result<Self> {
        let pool = ConnectionPool::new(args, PoolConfig::default()).await?;
        let broker = Self { pool };
        broker.init().await?;
        Ok(broker)
    }

    async fn init(&self) -> Result<()> {
        let channel = self.pool.get_channel().await?;

        // Declare exchanges
        for exchange in super::get_exchanges() {
            self.declare_exchange(&channel, exchange).await?;
        }

        // Declare and bind queues
        for queue in super::get_queues() {
            self.declare_and_bind_queue(&channel, queue).await?;
        }

        Ok(())
    }

    async fn declare_exchange(&self, channel: &Channel, builder: ExchangeBuilder) -> Result<()> {
        tracing::info!("Declaring exchange: {}", builder.get_name());
        channel.exchange_declare(builder.build()).await?;
        Ok(())
    }

    async fn declare_and_bind_queue(&self, channel: &Channel, queue: QueueBuilder) -> Result<()> {
        let queue_name = queue.get_name();
        tracing::info!("Declaring queue: {}", queue_name);

        // Declare queue
        let queue_args = queue.build_queue_args();
        channel.queue_declare(queue_args).await?;

        // Bind queue if binding info is provided
        if let Some(bind_args) = queue.build_bind_args() {
            tracing::info!(
                "Binding queue {} to exchange {} with routing key {}",
                bind_args.queue,
                bind_args.exchange,
                bind_args.routing_key
            );
            channel.queue_bind(bind_args).await?;
        }

        Ok(())
    }
}
