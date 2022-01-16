use serde::Deserialize;

use super::id::RoleId;

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
