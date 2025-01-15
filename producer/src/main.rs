use clap::Parser;
use common::client::ClientInterface;
use common::command::CommandInterface;
use common::product::ProductInterface;
use rdkafka::config::ClientConfig;
use rdkafka::producer::BaseProducer;
use sqlx::{PgPool, Error};

mod client;
mod command;
mod product;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// If provided, seeds the database with clients and products
    #[arg(long)]
    seed: bool,
}

async fn produce_client(pool: &PgPool) -> Result<(), Error> {
    let client = client::MyClient::generate_random();
    client.insert_into_db(pool).await?;
    Ok(())
}

async fn produce_product(pool: &PgPool) -> Result<(), Error> {
    let product = product::MyProduct::generate_random();
    product.insert_into_db(pool).await?;
    Ok(())
}

async fn produce_command(pool: &PgPool, producer: &BaseProducer) -> Result<(), Error> {
    let command = command::MyCommand::generate_random();
    command.process_command(pool, producer).await?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let cli = Cli::parse();
    let db_url = "postgres://postgres:postgres@localhost/postgres";
    let pool = PgPool::connect(db_url).await?;
    let producer: BaseProducer = ClientConfig::new()
        .set("bootstrap.servers", "localhost:29092")
        .create()
        .expect("Failed to create Kafka producer");

    if cli.seed {
        for _ in 0..100 {
            produce_client(&pool).await.unwrap();
            produce_product(&pool).await.unwrap();
        }
        println!("Database seeded with clients and products");
    } else {
        // Loop forever on producing commands
        loop {
            produce_command(&pool, &producer).await.unwrap();
        }
    }

    Ok(())
}
