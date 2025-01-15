use async_trait::async_trait;
use rdkafka::producer::BaseProducer;
use schema_registry_converter::async_impl::schema_registry::SrSettings;
use serde::{Deserialize, Serialize};
use serde_avro_derive::BuildSchema;
use sqlx::PgPool;

#[derive(Debug, Serialize, Deserialize, Clone, BuildSchema)]
pub struct Command {
    pub id: i32,
    pub client_id: i32,
    pub date: String,
    pub size: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CommandFromDb {
    pub id: i32,
    pub client_id: i32,
    pub date: String,
}

impl From<(CommandFromDb, i32)> for Command {
    fn from((command, size): (CommandFromDb, i32)) -> Self {
        Command {
            id: command.id,
            client_id: command.client_id,
            date: command.date,
            size,
        }
    }
}

#[async_trait]
pub trait CommandInterface {
    fn generate_random() -> Self;
    async fn process_command(&self, pool: &PgPool, producer: &BaseProducer, sr_settings: &SrSettings) -> Result<(), sqlx::Error>;
}