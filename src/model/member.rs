use chrono::{DateTime, Utc};
use serde::Deserialize;

use super::id::RoleId;
use super::User;

#[derive(Debug, Clone, Deserialize)]
pub struct Member {
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
