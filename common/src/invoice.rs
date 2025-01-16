use serde::{Deserialize, Serialize};
use serde_avro_derive::BuildSchema;

use crate::{client::Client, command::Command, product::Product};

#[derive(Deserialize, Serialize, Debug, Clone, BuildSchema)]
pub struct Invoice {
    pub id: i32,
    pub date: String,
    pub client: Client,
    pub products: Vec<Product>,
    pub total_price: f64,
    pub size: i32,
}

impl From<Command> for Invoice {
    fn from(command: Command) -> Self {
        Invoice {
            id: command.id,
            date: command.date,
            client: Client::default(),
            products: vec![],
            total_price: 0.0,
            size: command.size,
        }
    }
}

impl Invoice {
    pub fn add_product(&mut self, product: Product) {
        self.total_price += product.price;
        self.products.push(product);
    }
}