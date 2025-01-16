use std::collections::HashMap;

use common::client::Client;

pub fn process_client(client: &Client, clients: &mut HashMap<i32, Client>) {
    clients.insert(client.id, client.clone());
}