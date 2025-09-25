mod core;
mod initializer;

pub use amqprs::connection::OpenConnectionArguments;
pub use initializer::Broker;

// Define constants for exchange names
const DIRECT_EXCHANGE: &str = "aura_rs.exchange.direct";
const FANOUT_EXCHANGE: &str = "aura_rs.exchange.fanout";

// Define constants for queues
const TURBO_TOGEL_SETTLE_QUEUE: &str = "aura_rs.turbo_togel_settle";
const TURBO_TOGEL_DRAWING_RESULT_QUEUE: &str = "aura_rs.turbo_togel_drawing_result"; // This queue is used to receive drawing result tasks

// Define constants for routing keys
const TURBO_TOGEL_SETTLE_QUEUE_ROUTING_KEY: &str = "aura_rs.turbo_togel_settle.key";
const TURBO_TOGEL_DRAWING_RESULT_QUEUE_ROUTING_KEY: &str = "aura_rs.turbo_togel_drawing_result.key";

fn get_exchanges() -> Vec<core::ExchangeBuilder> {
    vec![
        core::ExchangeBuilder::new(DIRECT_EXCHANGE, core::ExchangeType::Direct).durable(true),
        core::ExchangeBuilder::new(FANOUT_EXCHANGE, core::ExchangeType::Fanout).durable(true),
        // Add more exchanges as needed
    ]
}

fn get_queues() -> Vec<core::QueueBuilder> {
    vec![
        core::QueueBuilder::new(TURBO_TOGEL_SETTLE_QUEUE)
            .durable(true)
            .bind_to(DIRECT_EXCHANGE, TURBO_TOGEL_SETTLE_QUEUE_ROUTING_KEY),
        core::QueueBuilder::new(TURBO_TOGEL_DRAWING_RESULT_QUEUE)
            .durable(true)
            .bind_to(
                DIRECT_EXCHANGE,
                TURBO_TOGEL_DRAWING_RESULT_QUEUE_ROUTING_KEY,
            ),
        // Add more queues as needed
    ]
}
