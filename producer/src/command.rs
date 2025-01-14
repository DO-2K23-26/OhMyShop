use async_trait::async_trait;
use sqlx::PgPool;
use chrono::DateTime;
use rand::Rng;
use rdkafka::producer::{BaseProducer, BaseRecord};

use common::client::Client;
use common::product::Product;
use common::command::CommandInterface;


#[derive(Debug)]
#[allow(dead_code)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct MyCommand {
    id: i32,
    client_id: i32,
    date: DateTime<chrono::Utc>,
    products: Vec<Product>,
}

#[async_trait]
impl CommandInterface for MyCommand {
    fn generate_random() -> Self {
        MyCommand {
            id: 0, // This will be set by the database
            client_id: 0,  //This will be set randomly
            date: chrono::Utc::now(),
            products: vec![],
        }
    }

    async fn process_command(&self, pool: &PgPool, producer: &BaseProducer, _topic_name: &str) -> Result<(), sqlx::Error> {
        //retrieve random products from database
        let product_limit: i64 = rand::thread_rng().gen_range(1..=10);

        //retrieve a random client from the database
        let client_object = sqlx::query_as!(
            Client,
            "SELECT * FROM Client ORDER BY RANDOM() LIMIT 1"
        )
        .fetch_one(pool)
        .await?;

        let client_payload = serde_json::to_string(&client_object).unwrap();
        if let Err(e) = producer.send(
            BaseRecord::to("Client")
                .payload(&client_payload)
                .key(&client_object.id.to_string()),
        ) {
            eprintln!("Failed to send message: {:?}", e);
        } else {
            println!("Message produced: {}", client_payload);
        }

        let products = sqlx::query_as!(
            Product,
            "SELECT * FROM Product ORDER BY RANDOM() LIMIT $1",
            product_limit
        )
        .fetch_all(pool)
        .await?;

        let command = sqlx::query!(
            r#"
            INSERT INTO Command (clientId, date) 
            VALUES ($1, $2) 
            RETURNING id, clientId AS "client_id", date
            "#,
            client_object.id,
            self.date.naive_utc().date()
        )
        .fetch_one(pool)
        .await?;

        let command_payload = serde_json::to_string(&self).unwrap();
        if let Err(e) = producer.send(
            BaseRecord::to("Command")
                .payload(&command_payload)
                .key(&command.id.to_string()),
        ) {
            eprintln!("Failed to send message: {:?}", e);
        } else {
            println!("Message produced: {}", command_payload);
        }

        for product in products {
            sqlx::query!(
                "INSERT INTO CommandProduct (commandId, productId) VALUES ($1, $2)",
                command.id,
                product.id
            )
            .execute(pool)
            .await?;
            let product_payload = serde_json::to_string(&product).unwrap();
            if let Err(e) = producer.send(
                BaseRecord::to("Product")
                    .payload(&product_payload)
                    .key(&product.id.to_string()),
            ) {
                eprintln!("Failed to send message: {:?}", e);
            } else {
                println!("Message produced: {}", product_payload);
            }    
        }

        Ok(())
    }
}