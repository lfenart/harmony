use serde::Serialize;

use super::CreateEmbed;

#[derive(Default, Serialize)]
pub struct EditMessage {
    content: Option<String>,
    embeds: Option<Vec<CreateEmbed>>,
    // TODO
    flags: Option<u64>,
    // allowed_mentions: Option<AllowedMentions>,
}

/*
content	string	the message contents (up to 2000 characters)
embeds	array of embed objects	embedded rich content (up to 6000 characters)
embed (deprecated)	embed object	embedded rich content, deprecated in favor of embeds
flags	integer	edit the flags of a message (only SUPPRESS_EMBEDS can currently be set/unset)
file	file contents	the contents of the file being sent/edited
payload_json	string	JSON encoded body of non-file params (multipart/form-data only)
allowed_mentions	allowed mention object	allowed mentions for the message
attachments	array of attachment objects	attached files to keep
components	array of message component	the components to include with the message
*/

impl EditMessage {
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
        match self.embeds.as_mut() {
            Some(embeds) => embeds.push(embed),
            None =>self.embeds = Some(vec![embed]),
        }
        self
    }
}
