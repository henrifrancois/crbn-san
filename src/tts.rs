use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TTSMessageData {
    pub guild_id: Option<String>,
    pub channel_id: String,
    pub user_id: String,
    pub username: String,
    pub display_name: String,
    pub message_content: String,
    pub timestamp: String,
    pub message_id: String,
    pub voice: Option<String>,
}
