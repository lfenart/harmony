use std::hash::Hash;

use crate::model::id::{ChannelId, GuildId, MessageId, UserId, WebhookId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Route {
    Channel(ChannelId),
    ChannelMessage(ChannelId, MessageId),
    Guild(GuildId),
    GuildMember(GuildId, UserId),
    Webhook(WebhookId),
}
