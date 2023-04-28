use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct InteractionObject {
    pub id: String,
    pub application_id: String,
    #[serde(rename = "type")]
    pub interaction_type: i32,
    pub data: Option<InteractionData>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct InteractionData {
    pub id: String,
    #[serde(rename = "type")]
    pub command_type: i32,
    pub name: String,
    pub resolved: Option<ResolvedData>,
}

impl InteractionData {
    pub fn get_content(&self) -> Option<String> {
        Some(
            self.clone()
                .resolved?
                .messages
                .into_values()
                .find(|_x| true)?
                .content,
        )
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ResolvedData {
    pub messages: HashMap<String, Message>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Message {
    pub content: String,
}
