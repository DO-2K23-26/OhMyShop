use std::{collections::HashMap, sync::Arc};
use common::{invoice::Invoice, product::Product};
use rdkafka::producer::FutureProducer;
use tokio::time::{sleep, Duration};
use tokio::task;
use tokio::sync::Mutex;

use crate::{send_invoice, send_to_dlq};

pub async fn process_product(
    producer: &FutureProducer,
    product: &Product,
    invoices: Arc<Mutex<HashMap<i32, Invoice>>>,
) {
    // Lock the invoices and attempt to retrieve the corresponding invoice
    let mut invoices_lock = invoices.lock().await;

    if let Some(invoice) = invoices_lock.get_mut(&product.command_id) {
        invoice.add_product(product.clone());
        if invoice.products.len() == invoice.size as usize {
            // Invoice complete, send to producer
            send_invoice(producer, invoice).await;
            invoices_lock.remove(&product.command_id);
        }
    } else {
        // Invoice not found, spawn a task to wait for 20 seconds before rechecking
        let invoices_clone = Arc::clone(&invoices);
        let command_id = product.command_id;
        let product_clone = product.clone();
        let producer_clone = producer.clone();

        task::spawn(async move {
            sleep(Duration::from_secs(20)).await;

            // Lock the invoices again and perform the operation
            let mut invoices_lock = invoices_clone.lock().await;

            if let Some(invoice) = invoices_lock.get_mut(&command_id) {
                invoice.add_product(product_clone);
                if invoice.products.len() == invoice.size as usize {
                    // Invoice complete, send to producer
                    send_invoice(&producer_clone, invoice).await;
                    invoices_lock.remove(&command_id);
                }
            } else {
                // Still not found, send to the Dead Letter Queue (DLQ)
                send_to_dlq(&producer_clone, command_id, product_clone).await;
            }
        });
    }
}
