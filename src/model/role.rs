use serde::Deserialize;

use super::id::RoleId;
use crate::client::Context;
use crate::model::id::GuildId;
use crate::Result;

#[derive(Debug, Clone, Deserialize)]
pub struct Role {
    pub id: RoleId,
    pub name: String,
    pub color: u32,
    pub hoist: bool,
    // pub icon: Option<String>,
    // pub unicode_emoji: Option<String>,
    pub position: usize,
    pub permissions: String,
    pub managed: bool,
    pub mentionable: bool,
    // pub tags
}

impl Role {
    pub fn parse(ctx: &Context, guild_id: GuildId, text: &str) -> Result<Option<Self>> {
        let roles = ctx.get_guild_roles(guild_id)?;
        if let Some(role_id) = text
            .parse::<u64>()
            .ok()
            .or_else(|| {
                let len = text.len();
                if text.ends_with('>') && text.starts_with("<@&") {
                    return text[3..len - 1].parse().ok();
                }
                None
            })
            .map(|x| x.into())
        {
            if let Some(role) = roles.iter().find(|x| x.id == role_id) {
                return Ok(Some(role.clone()));
            }
        }
        if let Some(role) = roles
            .into_iter()
            .find(|x| x.name.eq_ignore_ascii_case(text))
        {
            return Ok(Some(role));
        }
        Ok(None)
    }
}

// id	snowflake	role id
// name	string	role name
// color	integer	integer representation of hexadecimal color code
// hoist	boolean	if this role is pinned in the user listing
// icon?	?string	role icon hash
// unicode_emoji?	?string	role unicode emoji
// position	integer	position of this role
// permissions	string	permission bit set
// managed	boolean	whether this role is managed by an integration
// mentionable	boolean	whether this role is mentionable
// tags?	role tags object	the tags this role has
