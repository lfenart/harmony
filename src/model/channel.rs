use chrono::{DateTime, Utc};
use serde::Deserialize;
use serde_repr::Deserialize_repr;

use super::id::{ChannelId, GuildId, MessageId, UserId};
use super::User;
use crate::client::Context;
use crate::Result;

#[derive(Debug, Clone, Deserialize)]
pub struct Channel {
    pub id: ChannelId,
    #[serde(rename = "type")]
    pub kind: ChannelKind,
    pub guild_id: Option<GuildId>,
    pub position: u64,
    //#[serde(default)]
    // permission_overwrites: Vec<Overwrite>,
    pub name: Option<String>,
    pub topic: Option<String>,
    #[serde(default)]
    pub nsfw: bool,
    pub last_message_id: Option<MessageId>,
    pub bitrate: Option<u64>,
    pub user_limit: Option<u64>,
    pub rate_limit_per_user: Option<u64>,
    #[serde(default)]
    pub recipients: Vec<User>,
    pub icon: Option<String>,
    pub owner_id: Option<UserId>,
    // application_id: Option<ApplicationId>,
    // parent_id: Option<>,
    pub last_pin_timestamp: Option<DateTime<Utc>>,
    pub rtc_region: Option<String>,
    pub video_quality_mode: Option<u64>,
    pub message_count: Option<u8>,
    pub member_count: Option<u8>,
    // thread_metadata: Option<>,
    // member: Option<>,
    // default_auto_archive_duration: ,
    pub permissions: Option<String>,
}

impl Channel {
    pub fn parse(ctx: &Context, guild_id: Option<GuildId>, text: &str) -> Result<Option<Self>> {
        if let Some(channel_id) = text.parse::<u64>().ok().or_else(|| {
            let len = text.len();
            if text.starts_with("<#") && text.ends_with('>') {
                return text[2..len - 1].parse().ok();
            }
            None
        }) {
            if let Some(channel) = ctx.channel(channel_id.into())? {
                return Ok(Some(channel));
            }
        }
        if let Some(guild_id) = guild_id {
            let channels = ctx.guild_channels(guild_id)?;
            if let Some(channel) = channels.into_iter().find(|x| {
                x.name
                    .as_ref()
                    .map_or(false, |y| y.eq_ignore_ascii_case(text))
            }) {
                return Ok(Some(channel));
            }
        }
        Ok(None)
    }
}

#[derive(Debug, Clone, Copy, Deserialize_repr)]
#[repr(u8)]
pub enum ChannelKind {
    GuildText = 0,
    Dm = 1,
    GuildVoice = 2,
    GroupDm = 3,
    GuildCategory = 4,
    GuildNews = 5,
    GuildStore = 6,
    GuildNewsThread = 10,
    GuildPublicThread = 11,
    GuildPrivateThread = 12,
    GuildStageVoice = 13,
}
