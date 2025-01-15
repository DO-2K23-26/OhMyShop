use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Product {
    pub id: i32,
    pub name: String,
    pub price: f64,
    pub command_id: i32,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ProductFromDb {
    pub id: i32,
    pub name: String,
    pub price: f64,
}

impl From<(ProductFromDb, i32)> for Product {
    fn from((product, command_id): (ProductFromDb, i32)) -> Self {
        Product {
            id: product.id,
            name: product.name,
            price: product.price,
            command_id,
        }
    }
}

#[async_trait]
pub trait ProductInterface {
    fn generate_random() -> Self;
    async fn insert_into_db(&self, pool: &PgPool) -> Result<(), sqlx::Error>;
}