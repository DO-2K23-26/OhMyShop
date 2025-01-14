use async_trait::async_trait;
use common::product::ProductInterface;
use fake::Fake;
use sqlx::PgPool;

#[derive(Debug)]
pub struct Product {
    pub id: i32,
    pub name: String,
    pub price: f64,
}

#[async_trait]
impl ProductInterface for Product {
    fn generate_random() -> Self {
        Product {
            id: 0, // This will be set by the database
            name: fake::Faker.fake(),
            price: fake::Faker.fake::<f64>(),
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