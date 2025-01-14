use async_trait::async_trait;
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
    async fn insert_into_db(&self, pool: &PgPool) -> Result<(), sqlx::Error>;
}