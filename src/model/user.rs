use serde::Deserialize;

use super::id::UserId;

#[derive(Debug, Clone, Deserialize)]
pub struct User {
    pub id: UserId,
    pub username: String,
    pub discriminator: String,
    pub avatar: Option<String>,
    #[serde(default)]
    pub bot: bool,
    pub public_flags: Option<u64>,
}
