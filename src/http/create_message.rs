use serde::Serialize;

use super::CreateEmbed;

#[derive(Default, Serialize)]
pub struct CreateMessage {
    content: Option<String>,
    embeds: Vec<CreateEmbed>,
    // allowed_mentions: Option<AllowedMentions>,
    // message_reference: Option<MessageReference>,
}

impl CreateMessage {
    pub fn content<T: ToString>(&mut self, content: T) -> &mut Self {
        self.content = Some(content.to_string());
        self
    }

    pub fn embed<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut CreateEmbed) -> &mut CreateEmbed,
    {
        let mut embed = CreateEmbed::default();
        f(&mut embed);
        self.embeds.push(embed);
        self
    }
}
