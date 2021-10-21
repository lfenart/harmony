use chrono::{DateTime, Utc};
use serde::Deserialize;

use super::id::{ChannelId, GuildId, MessageId, RoleId, WebhookId};
use super::{Member, User};

#[derive(Debug, Clone, Deserialize)]
pub struct Message {
    pub id: MessageId,
    pub channel_id: ChannelId,
    pub guild_id: Option<GuildId>,
    pub author: User,
    pub member: Option<Member>,
    pub content: String,
    pub timestamp: DateTime<Utc>,
    pub edited_timestamp: Option<DateTime<Utc>>,
    pub tts: bool,
    pub mention_everyone: bool,
    pub mentions: Vec<User>,
    pub mention_roles: Vec<RoleId>,
    // #[serde(default)]
    // pub mention_channels: Vec<ChannelMention>,
    // pub attachments: Vec<Attachement>,
    // pub embeds: Vec<Embed>,
    // #[serde(default)]
    // pub reactions: Vec<Reaction>,
    #[serde(default)]
    pub nonce: serde_json::Value,
    pub pinned: bool,
    pub webhook_id: Option<WebhookId>,
    #[serde(rename = "type")]
    // pub kind: MessageType,
    pub kind: i64,
    // pub activity: Option<MessageActivity>,
    // pub application: Option<MessageApplication>,
    // pub message_reference: Option<MessageReference>,
    // pub flags: Option<MessageFlags>,
    pub referenced_message: Option<Box<Message>>,
    // pub interaction: Option<MessageInteraction>,
    // pub thread: Option<Thread>,
    // #[serde(default)]
    // pub components: Vec<ActionRow>,
    // #[serde(default)]
    // pub sticker_items: Vec<StickerItem>,
}
