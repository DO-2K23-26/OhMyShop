use serde::Serialize;
use serde_avro_derive::BuildSchema;
use crate::client::Client;
use crate::command::MergedCommand;

#[derive(Serialize, Debug, BuildSchema)]
pub struct Invoice {
    pub timestamp: String,
    pub client: Client,
    pub command: MergedCommand,
}