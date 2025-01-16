use std::{collections::HashMap, sync::Arc};
use common::{client::Client, command::Command, invoice::Invoice};
use rdkafka::producer::FutureProducer;
use tokio::time::{sleep, Duration};
use tokio::task;
use tokio::sync::Mutex;

use crate::send_to_dlq;

pub async fn process_command(
    producer: &FutureProducer,
    command: &Command,
    invoices: Arc<Mutex<HashMap<i32, Invoice>>>,
    clients: Arc<Mutex<HashMap<i32, Client>>>,
) {
    let mut invoice = Invoice::from(command.clone());

    // Attempt to retrieve the client from the HashMap
    if let Some(client) = clients.lock().await.remove(&command.client_id) {
        invoice.client = client;
        invoices.lock().await.insert(invoice.id, invoice);
    } else {
        // Client not found, spawn a task to wait for 20 seconds and recheck
        let arc_clients = Arc::clone(&clients);
        let arc_invoices = Arc::clone(&invoices);
        let command_clone = command.clone();
        let producer_clone = producer.clone();

        task::spawn(async move {
            sleep(Duration::from_secs(10)).await;

            // Lock the clients HashMap again to recheck
            let mut clients_lock = arc_clients.lock().await;
            let mut invoices_lock = arc_invoices.lock().await;

            if let Some(client) = clients_lock.remove(&command_clone.client_id) {
                let mut delayed_invoice = Invoice::from(command_clone.clone());
                delayed_invoice.client = client;
                invoices_lock.insert(delayed_invoice.id, delayed_invoice);
            } else {
                // Still not found, send the command to the Dead Letter Queue (DLQ)
                send_to_dlq(&producer_clone, command_clone.id, command_clone).await;
            }
        });
    }
}
