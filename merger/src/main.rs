mod client;
mod command;
mod product;

use client::process_client;
use command::process_command;
use common::client::Client;
use common::command::Command;
use common::invoice::Invoice;
use common::product::Product;
use product::process_product;
use rdkafka::config::RDKafkaLogLevel;
use rdkafka::consumer::{CommitMode, Consumer, StreamConsumer};
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::ClientConfig;
use rdkafka::Message;
use schema_registry_converter::async_impl::avro::{AvroDecoder, AvroEncoder};
use schema_registry_converter::async_impl::schema_registry::{post_schema, SrSettings};
use schema_registry_converter::schema_registry_common::{SchemaType, SubjectNameStrategy, SuppliedSchema};
use serde_avro_derive::BuildSchema;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

async fn decode_payload<T: for<'de> serde::Deserialize<'de> + std::fmt::Debug>(
    decoder: &AvroDecoder<'_>,
    payload: &[u8],
) -> Result<T, Box<dyn std::error::Error>> {
    let decoded = decoder.decode(Option::from(payload)).await?;
    let value: T = serde_json::from_value::<T>(Value::try_from(decoded.value).unwrap())?;
    println!("Decoded message: {:?}", value);
    Ok(value)
}

async fn handle_client(
    decoder: &AvroDecoder<'_>,
    payload: Vec<u8>,
    clients: &Mutex<HashMap<i32, Client>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let client = decode_payload::<Client>(decoder, &payload).await?;
    let mut clients_lock = clients.lock().await;
    process_client(&client, &mut *clients_lock);
    Ok(())
}

async fn handle_product(
    decoder: &AvroDecoder<'_>,
    payload: Vec<u8>,
    producer: &FutureProducer,
    invoices: Arc<Mutex<HashMap<i32, Invoice>>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let product = decode_payload::<Product>(decoder, &payload).await?;
    let invoices_clone = Arc::clone(&invoices);
    process_product(producer, &product, invoices_clone).await;
    Ok(())
}


async fn handle_command(
    decoder: &AvroDecoder<'_>,
    payload: Vec<u8>,
    producer: &FutureProducer,
    invoices: Arc<Mutex<HashMap<i32, Invoice>>>,
    clients: Arc<Mutex<HashMap<i32, Client>>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let command = decode_payload::<Command>(decoder, &payload).await?;
    let invoices_clone = Arc::clone(&invoices);
    let clients_clone = Arc::clone(&clients);
    process_command(producer, &command, invoices_clone, clients_clone).await;
    Ok(())
}


pub async fn send_invoice(producer: &FutureProducer, invoice: &Invoice) {
    let sr_settings = SrSettings::new(String::from("http://localhost:8085"));
    let encoder = AvroEncoder::new(sr_settings.clone());

    let invoice_msg = encoder
        .encode_struct(
            invoice,
            &SubjectNameStrategy::TopicNameStrategy("Invoice".to_string(), false),
        )
        .await
        .expect("Failed to encode invoice object");
    producer
        .send(
            FutureRecord::to("Invoice")
                .key(&invoice.id.to_string())
                .payload(&invoice_msg),
            Duration::from_secs(0),
        )
        .await
        .unwrap();
    println!("Invoice {} sent to topic.", invoice.id);
}

pub async fn send_to_dlq<T: for<'de> serde::Deserialize<'de> + serde::Serialize>(
    producer: &FutureProducer,
    key: i32,
    object: T,
) {
    let dead_letter_msg = serde_json::to_string(&object).unwrap();
    producer
        .send(
            FutureRecord::to("DeadLetterQueue")
                .key(&key.to_string())
                .payload(&dead_letter_msg),
            Duration::from_secs(0),
        )
        .await
        .unwrap();
    println!("Object {} moved to dead letter queue.", dead_letter_msg);
}

// Ensure shared state is managed by Arc<Mutex<_>>
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let sr_settings = SrSettings::new(String::from("http://localhost:8085"));
    let invoice_schema = Invoice::schema().unwrap();
    let decoder = AvroDecoder::new(sr_settings.clone());

    let invoice_supplied_schema = SuppliedSchema {
        name: Some(String::from("Invoice")),
        schema_type: SchemaType::Avro,
        schema: String::from(invoice_schema.json()),
        references: vec![],
    };

    if let Err(e) = post_schema(&sr_settings, "Invoice-value".to_string(), invoice_supplied_schema).await {
        eprintln!("Failed to post invoice schema: {}", e);
    };

    let consumer: StreamConsumer = ClientConfig::new()
        .set("group.id", "example-consumer-group")
        .set("bootstrap.servers", "localhost:19092")
        .set("enable.partition.eof", "false")
        .set("session.timeout.ms", "6000")
        .set("enable.auto.commit", "true")
        .set_log_level(RDKafkaLogLevel::Debug)
        .create()
        .expect("Consumer creation failed");

    let producer: FutureProducer = ClientConfig::new()
        .set("bootstrap.servers", "localhost:19092")
        .create()
        .expect("Failed to create Kafka producer");

    // Use Arc<Mutex<_>> for shared state
    let invoices = Arc::new(Mutex::new(HashMap::new()));
    let clients = Arc::new(Mutex::new(HashMap::new()));

    consumer
        .subscribe(&["Client", "Command", "Product"])
        .expect("Failed to subscribe to topics");

    loop {
        match consumer.recv().await {
            Ok(message) => {
                println!("Message received!");
                if let Some(payload) = message.payload() {
                    let topic = message.topic();
                    let payload = payload.to_vec();

                    match topic {
                        "Client" => {
                            // Pass Arc<Mutex<HashMap>> directly
                            handle_client(&decoder, payload, &clients).await?;
                        }
                        "Command" => {
                            // Pass Arc<Mutex<HashMap>> directly
                            handle_command(&decoder, payload, &producer, Arc::clone(&invoices), Arc::clone(&clients)).await?;
                        }
                        "Product" => {
                            // Pass Arc<Mutex<HashMap>> directly
                            handle_product(&decoder, payload, &producer, Arc::clone(&invoices)).await?;
                        }
                        _ => (),
                    }
                }
                consumer.commit_message(&message, CommitMode::Async).unwrap();
            }
            Err(e) => eprintln!("Error while consuming: {:?}", e),
        }
    }
}
