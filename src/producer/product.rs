use fake::Fake;
use sqlx::PgPool;

#[derive(Debug)]
pub struct Product {
    id: i32,
    name: String,
    price: f32,
}

impl Product {
    pub fn generate_random() -> Self {
        Product {
            id: 0, // This will be set by the database
            name: fake::Faker.fake(),
            price: fake::Faker.fake(),
        }
    }

    pub async fn insert_into_db(&self, pool: &PgPool) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "INSERT INTO products (name, price) VALUES ($1, $2)",
            self.name,
            self.price
        )
        .execute(pool)
        .await?;
        Ok(())
    }
}