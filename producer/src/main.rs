use clap::Parser;
use common::client::{Client, ClientInterface};
use common::command::{Command, CommandInterface};
use common::product::{Product, ProductInterface};
use rdkafka::config::ClientConfig;
use rdkafka::producer::BaseProducer;
use schema_registry_converter::async_impl::schema_registry::{post_schema, SrSettings};
use schema_registry_converter::schema_registry_common::{SchemaType, SuppliedSchema};
use serde_avro_derive::BuildSchema;
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

async fn produce_command(
    pool: &PgPool,
    producer: &BaseProducer,
    sr_settings: &SrSettings,
) -> Result<(), Error> {
    let command = command::MyCommand::generate_random();
    command.process_command(pool, producer, sr_settings).await?;
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

    let sr_settings = SrSettings::new(String::from("http://localhost:8085"));
    let client_schema = Client::schema().unwrap();
    let product_schema = Product::schema().unwrap();
    let command_schema = Command::schema().unwrap();

    let client_supplied_schema = SuppliedSchema {
        name: Some(String::from("Client")),
        schema_type: SchemaType::Avro,
        schema: String::from(client_schema.json()),
        references: vec![],
    };

    let product_supplied_schema = SuppliedSchema {
        name: Some(String::from("Product")),
        schema_type: SchemaType::Avro,
        schema: String::from(product_schema.json()),
        references: vec![],
    };

    let command_supplied_schema = SuppliedSchema {
        name: Some(String::from("Command")),
        schema_type: SchemaType::Avro,
        schema: String::from(command_schema.json()),
        references: vec![],
    };

    if let Err(e) = post_schema(&sr_settings, "Client-value".to_string(), client_supplied_schema).await {
        eprintln!("Failed to post client schema: {}", e);
    };
    
    if let Err(e) = post_schema(&sr_settings, "Product-value".to_string(), product_supplied_schema).await {
        eprintln!("Failed to post product schema: {}", e);
    };
    
    if let Err(e) = post_schema(&sr_settings, "Command-value".to_string(), command_supplied_schema).await {
        eprintln!("Failed to post command schema: {}", e);
    };


    if cli.seed {
        for _ in 0..100 {
            produce_client(&pool).await.unwrap();
            produce_product(&pool).await.unwrap();
        }
        println!("Database seeded with clients and products");
    } else {
        // Loop forever on producing commands
        loop {
            produce_command(&pool, &producer, &sr_settings).await.unwrap();
        }
    }

    Ok(())
}
