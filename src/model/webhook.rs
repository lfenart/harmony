use serde_repr::Deserialize_repr;

use super::id::{ChannelId, GuildId, WebhookId};
use super::User;

pub struct Webhook {
    pub id: WebhookId,
    pub kind: WebhookKind,
    pub guild_id: Option<GuildId>,
    pub channel_id: Option<ChannelId>,
    pub user: Option<User>,
    pub name: Option<String>,
    pub avatar: Option<String>,
    pub token: Option<String>,
    // pub application_id: Option<ApplicationId>,
    // pub source_guild: Option<PartialGuild>,
    // pub source_channel: Option<PartialChannel>,
    pub url: Option<String>,
}

#[derive(Debug, Clone, Copy, Deserialize_repr)]
#[repr(u8)]
pub enum WebhookKind {
    Incoming = 1,
    ChannelFollower = 2,
    Application = 3,
}
