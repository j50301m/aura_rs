mod initializer;
mod internal;

// Define constants for exchange names
const DIRECT_EXCHANGE: &str = "aura_rs.exchange.direct";
const FANOUT_EXCHANGE: &str = "aura_rs.exchange.fanout";

// Define constants for queues
const TURBO_TOGEL_SETTLE_QUEUE: &str = "aura_rs.turbo_togel_settle";
const TURBO_TOGEL_DRAWING_RESULT_QUEUE: &str = "aura_rs.turbo_togel_drawing_result"; // 這個 queue 是用來接收 drawing開獎後的任務的

// Define constants for routing keys
const TURBO_TOGEL_SETTLE_QUEUE_ROUTING_KEY: &str = "aura_rs.turbo_togel_settle.key";
const TURBO_TOGEL_DRAWING_RESULT_QUEUE_ROUTING_KEY: &str = "aura_rs.turbo_togel_drawing_result.key";

fn get_exchanges() -> Vec<internal::ExchangeBuilder> {
    vec![
        internal::ExchangeBuilder::new(DIRECT_EXCHANGE, internal::ExchangeType::Direct)
            .durable(true),
        internal::ExchangeBuilder::new(FANOUT_EXCHANGE, internal::ExchangeType::Fanout)
            .durable(true),
        // Add more exchanges as needed
    ]
}

fn get_queues() -> Vec<internal::QueueBuilder> {
    vec![
        internal::QueueBuilder::new(TURBO_TOGEL_SETTLE_QUEUE)
            .durable(true)
            .bind_to(DIRECT_EXCHANGE, TURBO_TOGEL_SETTLE_QUEUE_ROUTING_KEY),
        internal::QueueBuilder::new(TURBO_TOGEL_DRAWING_RESULT_QUEUE)
            .durable(true)
            .bind_to(
                DIRECT_EXCHANGE,
                TURBO_TOGEL_DRAWING_RESULT_QUEUE_ROUTING_KEY,
            ),
        // Add more queues as needed
    ]
}
