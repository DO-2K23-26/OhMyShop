use serde::{Deserialize, Serialize};
use crate::common::product::Product;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Command {
    pub id: i32,
    pub client_id: i32,
    pub date: String,
    pub produits: Vec<Product>,
}
