mod command;
mod product;
mod client;

use std::collections::HashMap;
use std::time::Duration;
use client::process_client;
use command::process_command;
use product::process_product;
use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::ClientConfig;
use rdkafka::Message;
use schema_registry_converter::async_impl::avro::AvroDecoder;
use schema_registry_converter::async_impl::schema_registry::{post_schema, SrSettings};
use schema_registry_converter::schema_registry_common::{SchemaType, SuppliedSchema};
use common::client::Client;
use common::command::Command;
use common::product::Product;
use common::invoice::Invoice;
use serde_avro_derive::BuildSchema;
use serde_json::Value;

async fn decode_payload<T: for<'de> serde::Deserialize<'de> + std::fmt::Debug>  (
    decoder: &AvroDecoder<'_>,
    payload: &[u8],
) -> Result<T, Box<dyn std::error::Error>> {
    let decoded = decoder.decode(Option::from(payload)).await?;
    let value: T = serde_json::from_value::<T>(Value::try_from(decoded.value).unwrap())?;
    println!("Decoded message: {:?}", value);
    Ok(value)
}

pub async fn send_to_dlq<T: for<'de> serde::Deserialize<'de> + serde::Serialize>(producer: &FutureProducer, key: i32, object: T) {
    // send to DLQ
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
        .set("bootstrap.servers", "localhost:29092")
        .set("auto.offset.reset", "earliest")
        .create()
        .expect("Failed to create Kafka consumer");

    let producer: FutureProducer = ClientConfig::new()
        .set("bootstrap.servers", "localhost:29092")
        .create()
        .expect("Failed to create Kafka producer");

    let mut invoices: HashMap<i32, Invoice> = HashMap::new();
    let mut clients: HashMap<i32, Client> = HashMap::new();

    consumer.subscribe(&["Client", "Command", "Product"]).expect("Failed to subscribe to topics");

    loop {
        match consumer.recv().await {
            Ok(message) => {
                if let Some(payload) = message.payload() {
                    let topic = message.topic();

                    match topic {
                        "Client" => {
                            let client = decode_payload::<Client>(&decoder, payload).await?;

                            process_client(&client, &mut clients);
                        }
                        "Command" => {
                            let command = decode_payload::<Command>(&decoder, payload).await?;

                            process_command(&producer, &command, &mut invoices, &mut clients);
                        }
                        "Product" => {
                            let product = decode_payload::<Product>(&decoder, payload).await?;

                            process_product(&producer, &product);
                        }
                        _ => (),
                    }
                }
            }
            Err(e) => eprintln!("Error while consuming: {:?}", e),
        }
    }
}