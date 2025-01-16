use std::{collections::HashMap, sync::Arc};
use common::{client::Client, command::Command, invoice::Invoice};
use rdkafka::producer::FutureProducer;
use tokio::sync::Mutex;
use backon::{ExponentialBuilder, Retryable};
use crate::send_to_dlq;

pub async fn process_command(
    producer: Arc<FutureProducer>,
    command: Command,
    invoices: Arc<Mutex<HashMap<i32, Invoice>>>,
    clients: Arc<Mutex<HashMap<i32, Client>>>,
) {
    // Spawn a new task to process the command
    tokio::spawn(async move {
        let retry_result = (|| async {
            let clients_lock = clients.lock().await;
            clients_lock.get(&command.client_id).cloned().ok_or_else(|| {
                anyhow::anyhow!(
                    "Client with ID {} not found",
                    command.client_id
                )
            })
        })
        .retry(ExponentialBuilder::default().with_max_times(5))
        .sleep(tokio::time::sleep)
        .notify(|err, dur| {
            println!(
                "Retrying to fetch client with ID {} after {:?}: {}",
                command.client_id, dur, err
            );
        })
        .await;

        match retry_result {
            Ok(client) => {
                // Client found, create the invoice and insert into shared map
                let mut invoice = Invoice::from(command.clone());
                invoice.client = client;

                let mut invoices_lock = invoices.lock().await;
                invoices_lock.insert(invoice.id, invoice);
            }
            Err(_) => {
                // Client not found after retries, send the command to the DLQ
                send_to_dlq(&producer, command.id, command).await;
            }
        }
    });
}
