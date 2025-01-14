use sqlx::{PgPool, Error};
use tokio::time::Duration;

mod client;
mod command;
mod product;

async fn produce_client(pool: &PgPool) -> Result<(), Error> {
    let client = client::MyClient::generate_random();
    client.insert_into_db(pool).await?;
    // Here you would also produce the client data to Kafka

    Ok(())
}

async fn produce_order(pool: &PgPool) -> Result<(), Error> {
    let order = command::MyCommand::generate_random(); // Assuming client_id = 1 for simplicity
    order.insert_into_db(pool).await?;
    // Here you would also produce the order data to Kafka

    Ok(())
}

async fn produce_product(pool: &PgPool) -> Result<(), Error> {
    let product = product::Product::generate_random();
    product.insert_into_db(pool).await?;
    // Here you would also produce the product data to Kafka

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let db_url = "postgres://postgres:postgres@localhost/postgres";
    let pool = PgPool::connect(db_url).await?;

    loop {
        produce_client(&pool).await.unwrap();
        produce_order(&pool).await.unwrap();
        produce_product(&pool).await.unwrap();
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}