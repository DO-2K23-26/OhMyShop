use serde::Serialize;
use crate::client::Client;
use crate::command::MergedCommand;

#[derive(Serialize, Debug)]
pub struct Invoice {
    pub timestamp: String,
    pub client: Client,
    pub command: MergedCommand,
}