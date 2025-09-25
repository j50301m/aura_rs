use amqprs::channel::ExchangeDeclareArguments;

pub use amqprs::channel::ExchangeType;

// Builder for Exchange configuration
#[derive(Debug)]
pub struct ExchangeBuilder {
    name: String,
    exchange_type: ExchangeType,
    durable: bool,
    auto_delete: bool,
}

// Convenience methods for common patterns
impl ExchangeBuilder {
    pub fn direct(name: &str) -> Self {
        Self::new(name, ExchangeType::Direct)
    }

    pub fn fanout(name: &str) -> Self {
        Self::new(name, ExchangeType::Fanout)
    }

    pub fn topic(name: &str) -> Self {
        Self::new(name, ExchangeType::Topic)
    }
}

impl ExchangeBuilder {
    pub fn new(name: &str, exchange_type: ExchangeType) -> Self {
        Self {
            name: name.to_string(),
            exchange_type,
            durable: false,
            auto_delete: false,
        }
    }

    pub fn durable(mut self, durable: bool) -> Self {
        self.durable = durable;
        self
    }

    pub fn auto_delete(mut self, auto_delete: bool) -> Self {
        self.auto_delete = auto_delete;
        self
    }

    pub fn build(self) -> ExchangeDeclareArguments {
        ExchangeDeclareArguments {
            exchange: self.name,
            exchange_type: self.exchange_type.to_string(),
            durable: self.durable,
            auto_delete: self.auto_delete,
            ..Default::default()
        }
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }
}
