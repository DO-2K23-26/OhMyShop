use serde::{Deserialize, Serialize};
use serde_avro_derive::BuildSchema;

use crate::{client::Client, command::Command, product::Product};

#[derive(Deserialize, Serialize, Debug, BuildSchema)]
pub struct Invoice {
    pub id: i32,
    pub date: String,
    pub client: Client,
    pub products: Vec<Product>,
    pub total_price: f64,
}

impl From<Command> for Invoice {
    fn from(command: Command) -> Self {
        Invoice {
            id: command.id,
            date: command.date,
            client: Client::default(),
            products: vec![],
            total_price: 0.0,
        }
    }
}