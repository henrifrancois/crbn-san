

pub mod TextToSpeech {
    pub struct TTSMessageData {
        guildId: Option<String>,
        channelId: String,
        userId: String,
        username: String,
        displayName: String,
        messageContent: String,
        timestamp: String,
        messageId: String,
        voice: Option<String>,
    }
}