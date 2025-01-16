use serde::{Deserialize, Serialize};
use serde_avro_derive::BuildSchema;
use sqlx::PgPool;
use async_trait::async_trait;

#[derive(Debug, Serialize, Deserialize, Clone, BuildSchema)]
pub struct Client {
    pub id: i32,
    pub name: String,
    pub email: String,
    pub address: String,
}

#[async_trait]
pub trait ClientInterface {
    fn generate_random() -> Self;
    async fn insert_into_db(&self, pool: &PgPool) -> Result<(), sqlx::Error>;
}

impl Default for Client {
    fn default() -> Self {
        Client {
            id: 0,
            name: "John Doe".to_string(),
            email: "john.doe@kafka.fr".to_string(),
            address: "123 Kafka Street".to_string(),
        }
    }
}