use sqlx::PgPool;
use chrono::NaiveDate;

use crate::product::Product;

#[derive(Debug)]
pub struct Order {
    id: i32,
    client_id: i32,
    date: NaiveDate,
    products: Vec<Product>,
}

impl Order {
    pub fn generate_random(client_id: i32, products: Vec<Product>) -> Self {
        Order {
            id: 0, // This will be set by the database
            client_id,
            date: chrono::Utc::today().naive_utc(),
            products,
        }
    }

    pub async fn insert_into_db(&self, pool: &PgPool) -> Result<(), sqlx::Error> {
        let order_id = sqlx::query!(
            "INSERT INTO Command (clientId, date) VALUES ($1, $2) RETURNING id",
            self.client_id,
            self.date
        )
        .fetch_one(pool)
        .await?
        .id;

        for product in &self.products {
            sqlx::query!(
                "INSERT INTO CommandProduct (commandId, productId) VALUES ($1, $2)",
                order_id,
                product.id
            )
            .execute(pool)
            .await?;
        }

        Ok(())
    }
}