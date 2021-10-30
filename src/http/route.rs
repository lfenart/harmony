use std::hash::Hash;

use crate::model::id::{ChannelId, GuildId, MessageId, WebhookId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Route {
    Channel(ChannelId),
    ChannelMessage(ChannelId, MessageId),
    Guild(GuildId),
    Webhook(WebhookId),
}
