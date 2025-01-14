use sqlx::PgPool;
use chrono::DateTime;
use rand::Rng;

use crate::product::Product;
use crate::client::ClientObject;

#[derive(Debug)]
#[allow(dead_code)]
pub struct Order {
    id: i32,
    client_id: i32,
    date: DateTime<chrono::Utc>,
    products: Vec<Product>,
}

impl Order {
    pub fn generate_random() -> Self {
        Order {
            id: 0, // This will be set by the database
            client_id: 0,  //This will be set randomly
            date: chrono::Utc::now(),
            products: vec![],
        }
    }

    pub async fn insert_into_db(&self, pool: &PgPool) -> Result<(), sqlx::Error> {
        //retrieve random products from database
        let product_limit: i64 = rand::thread_rng().gen_range(1..=10);

        //retrieve a random client from the database
        let client_object = sqlx::query_as!(
            ClientObject,
            "SELECT * FROM Client ORDER BY RANDOM() LIMIT 1"
        )
        .fetch_one(pool)
        .await?;

        let products = sqlx::query_as!(
            Product,
            "SELECT * FROM Product ORDER BY RANDOM() LIMIT $1",
            product_limit
        )
        .fetch_all(pool)
        .await?;

        let order_id = sqlx::query!(
            "INSERT INTO Command (clientId, date) VALUES ($1, $2) RETURNING id",
            client_object.id,
            self.date.naive_utc().date()
        )
        .fetch_one(pool)
        .await?
        .id;

        for product in products {
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