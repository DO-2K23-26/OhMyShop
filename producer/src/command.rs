use async_trait::async_trait;
use chrono::DateTime;
use common::client::Client;
use rand::Rng;
use rdkafka::producer::{BaseProducer, BaseRecord};
use schema_registry_converter::async_impl::avro::AvroEncoder;
use schema_registry_converter::async_impl::schema_registry::SrSettings;
use schema_registry_converter::schema_registry_common::SubjectNameStrategy;
use serde_avro_derive::BuildSchema;
use serde_avro_fast::Schema;
use sqlx::PgPool;

use common::command::{Command, CommandFromDb, CommandInterface};
use common::product::{Product, ProductFromDb};

#[derive(Debug)]
#[allow(dead_code)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct MyCommand {
    client_id: i32,
    date: DateTime<chrono::Utc>,
}

#[async_trait]
impl CommandInterface for MyCommand {
    fn generate_random() -> Self {
        MyCommand {
            client_id: 0, //This will be set randomly
            date: chrono::Utc::now(),
        }
    }

    async fn process_command(
        &self,
        pool: &PgPool,
        producer: &BaseProducer,
        sr_settings: &SrSettings,
    ) -> Result<(), sqlx::Error> {
        let encoder = AvroEncoder::new((*sr_settings).clone());

        //retrieve random products from database
        let product_limit: i32 = rand::thread_rng().gen_range(1..=10);
        let client_schema: Schema = match Client::schema() {
            Ok(schema) => schema,
            Err(e) => {
                eprintln!("Failed to get client schema: {:?}", e);
                return Err(sqlx::Error::RowNotFound); // or any other appropriate error handling
            }
        };
        let product_schema: Schema = match Product::schema() {
            Ok(schema) => schema,
            Err(e) => {
                eprintln!("Failed to get product schema: {:?}", e);
                return Err(sqlx::Error::RowNotFound); // or any other appropriate error handling
            }
        };
        let command_schema: Schema = match Command::schema() {
            Ok(schema) => schema,
            Err(e) => {
                eprintln!("Failed to get command schema: {:?}", e);
                return Err(sqlx::Error::RowNotFound); // or any other appropriate error handling
            }
        };

        //retrieve a random client from the database
        let client_object = sqlx::query_as!(
            Client,
            "SELECT * FROM Client ORDER BY RANDOM() LIMIT 1"
        )
            .fetch_one(pool)
            .await?;

        let command_from_db = sqlx::query_as!(
            CommandFromDb,
            r#"
            INSERT INTO Command (clientId, date)
            VALUES ($1, $2)
            RETURNING id, clientId AS "client_id", COALESCE(TO_CHAR(date, 'YYYY-MM-DD'), '') AS "date!"
            "#,
            client_object.id,
            self.date.naive_utc().date()
        )
            .fetch_one(pool)
            .await?;

        let command = Command::from((command_from_db, product_limit));

        let products_from_db = sqlx::query_as!(
            ProductFromDb,
            r#"SELECT id, name, price
            FROM Product
            ORDER BY RANDOM()
            LIMIT $1"#,
            product_limit as i64
        )
            .fetch_all(pool)
            .await?;

        let products: Vec<Product> = products_from_db
            .into_iter()
            .map(|product| Product::from((product, command.id)))
            .collect();

        let client_payload = encoder
            .encode_struct(
                &client_object,
                &SubjectNameStrategy::TopicNameStrategy("Client".to_string(), false),
            )
            .await
            .expect("Failed to encode client object");

        let command_payload = encoder
            .encode_struct(
                &command,
                &SubjectNameStrategy::TopicNameStrategy("Command".to_string(), false),
            )
            .await
            .expect("Failed to encode command object");



        if let Err(e) = producer.send(
            BaseRecord::to("Client")
                .payload(&client_payload)
                .key(&client_object.id.to_string()),
        ) {
            eprintln!("Failed to send message: {:?}", e);
        } else {
            let message = serde_avro_fast::from_datum_slice::<Client>(&client_payload, &client_schema).unwrap();

            println!("Message produced in Client: {}", serde_json::to_string(&message).unwrap());
        }

        if let Err(e) = producer.send(
            BaseRecord::to("Command")
                .payload(&command_payload)
                .key(&command.id.to_string()),
        ) {
            eprintln!("Failed to send message: {:?}", e);
        } else {
            let message = serde_avro_fast::from_datum_slice::<Command>(&command_payload, &command_schema).unwrap();
            println!("Message produced in Command: {}", serde_json::to_string(&message).unwrap());
        }

        for product in products {
            sqlx::query!(
                "INSERT INTO CommandProduct (commandId, productId) VALUES ($1, $2)",
                command.id,
                product.id
            )
                .execute(pool)
                .await?;
            let product_payload = encoder
                .encode_struct(
                    &product,
                    &SubjectNameStrategy::TopicNameStrategy("Product".to_string(), false),
                )
                .await
                .expect("Failed to encode product object");
            if let Err(e) = producer.send(
                BaseRecord::to("Product")
                    .payload(&product_payload)
                    .key(&product.id.to_string()),
            ) {
                eprintln!("Failed to send message: {:?}", e);
            } else {
                let message = serde_avro_fast::from_datum_slice::<Product>(&product_payload, &product_schema).unwrap();
                println!("Message produced in Product: {}", serde_json::to_string(&message).unwrap());
            }
        }

        Ok(())
    }
}
