use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Client {
    pub id: i32,
    pub name: String,
    pub email: String,
    pub address: String,
}
