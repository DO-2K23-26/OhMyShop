use std::collections::HashMap;

use common::{client::Client, command::Command, invoice::Invoice};
use rdkafka::producer::FutureProducer;

use crate::send_to_dlq;

pub fn process_command(producer: &FutureProducer, command: &Command, invoices: &mut HashMap<i32, Invoice>, clients: &mut HashMap<i32, Client>) {
  // insert into the invoices
  let mut invoice = Invoice::from(command.clone());
  
  //fill the client
  let _client = match clients.get(&command.client_id) {
    Some(client) => {
      invoice.client = client.clone();
      clients.remove(&command.client_id);
      invoices.insert(invoice.id, invoice);
    },
    None => {
      let command_id = command.client_id;
      let producer_clone = producer.clone();
      let clients_clone = clients.clone();
      tokio::spawn(async move {
        std::thread::sleep(std::time::Duration::from_secs(60));
        if let Some(client) = clients_clone.get(&command_id) {
          invoice.client = client.clone();
          let mut clients = clients_clone.clone();
          clients.remove(&command_id);
          send_to_dlq(&producer_clone, command_id, invoice).await;
          println!("Client not found after 1 minute");
        }
      });
      return;
    }
  };
}