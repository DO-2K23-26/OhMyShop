use async_trait::async_trait;
use common::product::ProductInterface;
use fake::{faker::lorem::fr_fr::Word, Fake};
use rand::Rng;
use sqlx::PgPool;

#[derive(Debug)]
#[allow(dead_code)]
pub struct MyProduct {
    pub id: i32,
    pub name: String,
    pub price: f64,
}

#[async_trait]
impl ProductInterface for MyProduct {
    fn generate_random() -> Self {
        MyProduct {
            id: 0, // This will be set by the database
            name: Word().fake::<String>(),
            price: rand::thread_rng().gen_range(1.0..=100.0),
        }
    }

    async fn insert_into_db(&self, pool: &PgPool) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "INSERT INTO Product (name, price) VALUES ($1, $2)",
            self.name,
            self.price
        )
        .execute(pool)
        .await?;
        Ok(())
    }
}