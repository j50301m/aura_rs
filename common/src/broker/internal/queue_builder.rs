use amqprs::channel::{QueueBindArguments, QueueDeclareArguments};

// Builder for Queue configuration
#[derive(Debug, Clone)]
pub struct QueueBuilder {
    name: String,
    durable: bool,
    auto_delete: bool,
    exchange: Option<String>,
    routing_key: Option<String>,
}

impl QueueBuilder {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            durable: false,
            auto_delete: false,
            exchange: None,
            routing_key: None,
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

    pub fn bind_to(mut self, exchange: &str, routing_key: &str) -> Self {
        self.exchange = Some(exchange.to_string());
        self.routing_key = Some(routing_key.to_string());
        self
    }

    pub fn build_queue_args(&self) -> QueueDeclareArguments {
        let mut args = QueueDeclareArguments::new(&self.name);
        args.durable(self.durable);
        args.auto_delete(self.auto_delete);
        args
    }

    pub fn build_bind_args(&self) -> Option<QueueBindArguments> {
        match (self.exchange.clone(), self.routing_key.clone()) {
            (Some(exchange), Some(routing_key)) => Some(QueueBindArguments {
                queue: self.name.clone(),
                exchange,
                routing_key,
                ..Default::default()
            }),
            _ => None,
        }
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }
}
