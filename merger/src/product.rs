use std::{collections::HashMap, sync::Arc};
use common::{invoice::Invoice, product::Product};
use rdkafka::producer::FutureProducer;
use tokio::sync::Mutex;
use backon::{ExponentialBuilder, Retryable};
use crate::{send_invoice, send_to_dlq};

pub async fn process_product(
    producer: Arc<FutureProducer>,
    product: Product,
    invoices: Arc<Mutex<HashMap<i32, Invoice>>>,
) {
    // Spawn a separate task for this product
    tokio::spawn(async move {
        let retry_result = (|| async {
            let mut invoices_lock = invoices.lock().await;
            if let Some(invoice) = invoices_lock.get_mut(&product.command_id) {
                invoice.add_product(product.clone());
                Ok(invoice.clone())
            } else {
                Err(anyhow::anyhow!(
                    "Invoice with command ID {} not found",
                    product.command_id
                ))
            }
        })
        .retry(ExponentialBuilder::default().with_max_times(5))
        .sleep(tokio::time::sleep)
        .notify(|err, dur| {
            println!(
                "Retrying to fetch invoice with command ID {} after {:?}: {}",
                product.command_id, dur, err
            );
        })
        .await;

        match retry_result {
            Ok(invoice) => {
                // Check if the invoice is complete
                if invoice.products.len() == invoice.size as usize {
                    send_invoice(&producer, &invoice).await;
                    // Remove the completed invoice from the shared map
                    let mut invoices_lock = invoices.lock().await;
                    invoices_lock.remove(&product.command_id);
                }
            }
            Err(_) => {
                // If invoice still not found after retries, send product to DLQ
                send_to_dlq(&producer, product.command_id, product).await;
            }
        }
    });
}
