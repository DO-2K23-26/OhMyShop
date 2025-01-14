use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use async_trait::async_trait;

#[derive(Debug, Serialize, Deserialize, Clone)]
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