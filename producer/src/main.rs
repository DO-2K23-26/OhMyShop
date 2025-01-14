use sqlx::{PgPool, Error};
use tokio::time::Duration;
use common::client::ClientInterface;
use common::command::CommandInterface;
use common::product::ProductInterface;
use rdkafka::producer::BaseProducer;
use rdkafka::config::ClientConfig;

mod client;
mod command;
mod product;

async fn produce_client(pool: &PgPool) -> Result<(), Error> {
    let client = client::MyClient::generate_random();
    client.insert_into_db(pool).await?;
    // Here you would also produce the client data to Kafka

    Ok(())
}

async fn produce_command(pool: &PgPool, producer: &BaseProducer, topic_name: &str) -> Result<(), Error> {
    let command = command::MyCommand::generate_random(); // Assuming client_id = 1 for simplicity
    let _ = command.process_command(pool,producer,topic_name).await;
    // Here you would also produce the order data to Kafka

    Ok(())
}

async fn produce_product(pool: &PgPool) -> Result<(), Error> {
    let product = product::MyProduct::generate_random();
    product.insert_into_db(pool).await?;
    // Here you would also produce the product data to Kafka

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let db_url = "postgres://postgres:postgres@localhost/postgres";
    let pool = PgPool::connect(db_url).await?;
    let producer: BaseProducer = ClientConfig::new()
        .set("bootstrap.servers", "localhost:29092")
        .create()
        .expect("Failed to create Kafka producer");


    loop {
        //produce_client(&pool).await.unwrap();
        //produce_product(&pool).await.unwrap();
            
        produce_command(&pool, &producer, "Command").await.unwrap();
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}