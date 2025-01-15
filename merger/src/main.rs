use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use chrono::Utc;
use tokio::time;
use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::ClientConfig;
use rdkafka::Message;

#[derive(Deserialize, Serialize, Debug, Clone)]
struct Client {
    id: i32,
    name: String,
    email: String,
    address: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
struct Command {
    id: i32,
    client_id: i32,
    date: String,
    size: usize,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
struct Product {
    command_id: i32,
    id: i32,
    name: String,
    price: f64,
}

#[derive(Serialize, Debug)]
struct Invoice {
    timestamp: String,
    client: Client,
    command: MergedCommand,
}

#[derive(Serialize, Debug)]
struct MergedCommand {
    id: i32,
    date: String,
    products: Vec<Product>,
    total_price: f64,
}

async fn process_command(
    producer: &FutureProducer,
    clients: &HashMap<i32, Client>,
    commands: Vec<Command>,
    products: Vec<Product>,
    invoice_topic: &str,
    dead_letter_topic: &str,
) {
    for command in commands {
        if let Some(client) = clients.get(&command.client_id) {
            let mut associated_products: Vec<Product> = Vec::new();
            let deadline = Utc::now() + chrono::Duration::seconds(120);

            while associated_products.len() < command.size {
                if Utc::now() >= deadline {
                    let dead_letter_msg = serde_json::to_string(&command).unwrap();
                    producer
                        .send(
                            FutureRecord::to(dead_letter_topic)
                                .key(&command.id.to_string())
                                .payload(&dead_letter_msg),
                            Duration::from_secs(0),
                        )
                        .await
                        .unwrap();
                    println!("Command {} moved to dead letter queue.", command.id);
                    break;
                }

                for product in &products {
                    if product.command_id == command.id
                        && !associated_products.iter().any(|p| p.id == product.id)
                    {
                        associated_products.push(product.clone());
                        if associated_products.len() == command.size {
                            break;
                        }
                    }
                }
                time::sleep(Duration::from_millis(100)).await;
            }

            if associated_products.len() == command.size {
                let total_price: f64 = associated_products.iter().map(|p| p.price).sum();
                let merged_command = MergedCommand {
                    id: command.id,
                    date: command.date.clone(),
                    products: associated_products,
                    total_price,
                };
                let invoice = Invoice {
                    timestamp: Utc::now().to_rfc3339(),
                    client: client.clone(),
                    command: merged_command,
                };
                let invoice_msg = serde_json::to_string(&invoice).unwrap();

                producer
                    .send(
                        FutureRecord::to(invoice_topic)
                            .key(&command.id.to_string())
                            .payload(&invoice_msg),
                        Duration::from_secs(0),
                    )
                    .await
                    .unwrap();

                println!("Invoice for command {} sent to invoice topic.", command.id);
            }
        } else {
            let dead_letter_msg = serde_json::to_string(&command).unwrap();
            producer
                .send(
                    FutureRecord::to(dead_letter_topic)
                        .key(&command.id.to_string())
                        .payload(&dead_letter_msg),
                    Duration::from_secs(0),
                )
                .await
                .unwrap();
            println!("Command {} moved to dead letter queue due to missing client.", command.id);
        }
    }
}

#[tokio::main]
async fn main() {
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

    let mut clients: HashMap<i32, Client> = HashMap::new();
    let mut commands: Vec<Command> = Vec::new();
    let mut products: Vec<Product> = Vec::new();

    consumer.subscribe(&["Client", "Command", "Product"]).expect("Failed to subscribe to topics");

    loop {
        match consumer.recv().await {
            Ok(message) => {
                if let Some(payload) = message.payload() {
                    let topic = message.topic();
                    let payload_str = String::from_utf8_lossy(payload);

                    match topic {
                        "Client" => {
                            let client: Client = serde_json::from_str(&payload_str).unwrap();
                            clients.insert(client.id, client);
                        }
                        "Command" => {
                            let command: Command = serde_json::from_str(&payload_str).unwrap();
                            commands.push(command);
                        }
                        "Product" => {
                            let product: Product = serde_json::from_str(&payload_str).unwrap();
                            products.push(product);
                        }
                        _ => (),
                    }
                }
            }
            Err(e) => eprintln!("Error while consuming: {:?}", e),
        }

        if !commands.is_empty() {
            process_command(
                &producer,
                &clients,
                commands.clone(),
                products.clone(),
                "Invoice",
                "DeadLetterQueue",
            )
                .await;
            commands.clear();
        }
    }
}