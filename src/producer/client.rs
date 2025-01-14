use fake::{faker::name::en::Name, faker::internet::en::SafeEmail, faker::address::en::SecondaryAddress, Fake};
use sqlx::PgPool;

#[derive(Debug)]
#[allow(dead_code)]
pub struct ClientObject {
    pub id: i32,
    pub name: String,
    pub email: String,
    pub address: String,
}

impl ClientObject {
    pub fn generate_random() -> Self {

        ClientObject {
            id: 0, // This will be set by the database
            name: Name().fake(),
            email: SafeEmail().fake(),
            address: SecondaryAddress().fake(),
        }
    }

    pub async fn insert_into_db(&self, pool: &PgPool) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "INSERT INTO Client (name, email, address) VALUES ($1, $2, $3)",
            self.name,
            self.email,
            self.address
        )
        .execute(pool)
        .await?;
        Ok(())
    }
}