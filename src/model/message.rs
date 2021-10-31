use chrono::{DateTime, Utc};
use serde::Deserialize;
use serde_repr::Deserialize_repr;

use super::id::{ChannelId, GuildId, MessageId, RoleId, WebhookId};
use super::{PartialMember, User};

#[derive(Debug, Clone, Deserialize)]
pub struct Message {
    pub id: MessageId,
    pub channel_id: ChannelId,
    pub guild_id: Option<GuildId>,
    pub author: User,
    pub member: Option<PartialMember>,
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
    pub kind: MessageKind,
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

#[derive(Debug, Clone, Copy, Deserialize_repr)]
#[repr(u8)]
pub enum MessageKind {
    Default = 0,
    RecipientAdd = 1,
    RecipientRemove = 2,
    Call = 3,
    ChannelNameChange = 4,
    ChannelIconChange = 5,
    ChannelPinnedMessage = 6,
    GuildMemberJoin = 7,
    UserPremiumGuildSubscription = 8,
    UserPremiumGuildSubscriptionTier1 = 9,
    UserPremiumGuildSubscriptionTier2 = 10,
    UserPremiumGuildSubscriptionTier3 = 11,
    ChannelFollowAdd = 12,
    GuildDiscoveryDisqualified = 14,
    GuildDiscoveryRequalified = 15,
    GuildDiscoveryGracePeriodInitialWarning = 16,
    GuildDiscoveryGracePeriodFinalWarning = 17,
    ThreadCreated = 18,
    Reply = 19,
    ChatInputCommand = 20,
    ThreadStarterMessage = 21,
    GuildInviteReminder = 22,
    ContextMenuCommand = 23,
}
