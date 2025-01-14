use async_trait::async_trait;
use rdkafka::producer::BaseProducer;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::product::Product;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Command {
    pub id: i32,
    pub client_id: i32,
    pub date: String,
    pub produits: Vec<Product>,
}

#[async_trait]
pub trait CommandInterface {
    fn generate_random() -> Self;
    async fn process_command(&self, pool: &PgPool, producer: &BaseProducer, topic_name: &str) -> Result<(), sqlx::Error>;
}