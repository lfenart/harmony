use chrono::{DateTime, Utc};
use serde::Deserialize;

use super::id::RoleId;
use super::User;
use crate::client::Context;
use crate::model::id::GuildId;
use crate::Result;

#[derive(Debug, Clone, Deserialize)]
pub struct Member {
    pub user: User,
    pub nick: Option<String>,
    pub avatar: Option<String>,
    pub roles: Vec<RoleId>,
    pub joined_at: DateTime<Utc>,
    pub premium_since: Option<DateTime<Utc>>,
    pub deaf: bool,
    pub mute: bool,
    #[serde(default)]
    pub pending: bool,
    pub permissions: Option<String>,
}

impl Member {
    pub fn parse(ctx: &Context, guild_id: GuildId, text: &str) -> Result<Option<Self>> {
        if let Some(user_id) = text.parse::<u64>().ok().or_else(|| {
            let len = text.len();
            if len < 4 {
                return None;
            }
            if text.starts_with("<@!") {
                return text[3..len - 1].parse().ok();
            } else if text.starts_with("<@") {
                return text[2..len - 1].parse().ok();
            }
            None
        }) {
            if let Some(member) = ctx.member(guild_id, user_id.into())? {
                return Ok(Some(member));
            }
        }
        if text.chars().nth_back(4) == Some('#') {
            let name = &text[..text.len() - 5];
            let discriminator = &text[text.len() - 4..];
            let members = ctx.search_guild_members(guild_id, name)?;
            if let Some(member) = members.into_iter().find(|x| {
                x.user.discriminator == discriminator && x.user.username.eq_ignore_ascii_case(name)
            }) {
                return Ok(Some(member));
            }
        }
        let members = ctx.search_guild_members(guild_id, text)?;
        if let Some(member) = members.into_iter().find(|x| {
            x.user.username.eq_ignore_ascii_case(text)
                || x.nick
                    .as_ref()
                    .map_or(false, |y| y.eq_ignore_ascii_case(text))
        }) {
            return Ok(Some(member));
        }
        Ok(None)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct PartialMember {
    pub user: Option<User>,
    pub nick: Option<String>,
    pub avatar: Option<String>,
    pub roles: Vec<RoleId>,
    pub joined_at: DateTime<Utc>,
    pub premium_since: Option<DateTime<Utc>>,
    pub deaf: bool,
    pub mute: bool,
    #[serde(default)]
    pub pending: bool,
    pub permissions: Option<String>,
}
