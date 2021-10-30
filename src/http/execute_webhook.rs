use serde::Serialize;

use super::CreateEmbed;

#[derive(Default, Serialize)]
pub struct ExecuteWebhook {
    content: Option<String>,
    username: Option<String>,
    avatar_url: Option<String>,
    #[serde(default)]
    tts: bool,
    // file: String,
    embeds: Vec<CreateEmbed>,
    payload_json: Option<String>,
    // allowed_mentions: Option<AllowedMentions>,
    // message_reference: Option<MessageReference>,
}

impl ExecuteWebhook {
    pub fn content<T: ToString>(mut self, content: T) -> Self {
        self.content = Some(content.to_string());
        self
    }

    pub fn embed<F>(mut self, f: F) -> Self
    where
        F: FnOnce(CreateEmbed) -> CreateEmbed,
    {
        let embed = f(CreateEmbed::default());
        self.embeds.push(embed);
        self
    }
}
