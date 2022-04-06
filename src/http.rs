mod create_embed;
mod create_guild_role;
mod create_message;
mod edit_message;
mod execute_webhook;
mod rate_limiter;
mod route;

use std::fs::File;
use std::path::Path;
use std::sync::Arc;

use multipart::client::lazy::Multipart;
use serde_json::json;
use ureq::Agent;

use crate::model::id::{ChannelId, GuildId, MessageId, RoleId, UserId, WebhookId};
use crate::model::{Channel, Member, Message, Role};
use crate::{Error, Result};
pub use create_embed::CreateEmbed;
pub use create_guild_role::CreateGuildRole;
pub use create_message::CreateMessage;
pub use edit_message::EditMessage;
pub use execute_webhook::ExecuteWebhook;
pub use rate_limiter::RateLimiter;
pub use route::Route;

#[derive(Debug, Clone)]
pub struct Http {
    agent: Agent,
    token: Arc<str>,
    rate_limiter: RateLimiter,
}

impl Http {
    pub fn new(token: Arc<str>) -> Self {
        Self {
            agent: Agent::new(),
            token,
            rate_limiter: RateLimiter::new(),
        }
    }

    pub fn get_channel(&self, channel_id: ChannelId) -> Result<Option<Channel>> {
        let request = self
            .agent
            .get(&api!("/channels/{}", channel_id.0))
            .set("AUTHORIZATION", &self.token);
        let channel = match self.rate_limiter.call(None, request) {
            Ok(reponse) => Some(reponse.into_json()?),
            Err(Error::Ureq(ureq::Error::Status(404, _))) => None,
            Err(err) => return Err(err),
        };
        Ok(channel)
    }

    pub fn create_message<F>(&self, channel_id: ChannelId, f: F) -> Result<Message>
    where
        F: FnOnce(CreateMessage) -> CreateMessage,
    {
        let msg = f(CreateMessage::default());
        let request = self
            .agent
            .post(&api!("/channels/{}/messages", channel_id.0))
            .set("AUTHORIZATION", &self.token);
        let json = serde_json::to_value(msg).unwrap();
        let response =
            self.rate_limiter
                .send_json(Some(Route::Channel(channel_id)), request, json)?;
        let message = response.into_json()?;
        Ok(message)
    }

    pub fn send_files<F, P>(&self, channel_id: ChannelId, files: &[P], f: F) -> Result<Message>
    where
        F: FnOnce(CreateMessage) -> CreateMessage,
        P: AsRef<Path>,
    {
        let msg = f(CreateMessage::default());
        let json = serde_json::to_value(msg).unwrap();
        let mut m = Multipart::new();
        for (i, file) in files.iter().enumerate() {
            let mime = mime_guess::from_path(file).first_or_octet_stream();
            m.add_stream(
                format!("files[{}]", i),
                File::open(file)?,
                Some(file.as_ref().file_name().unwrap().to_string_lossy()),
                Some(mime),
            );
        }
        m.add_text("payload_json", json.to_string());
        let mdata = m.prepare().unwrap();
        let request = self
            .agent
            .post(&api!("/channels/{}/messages", channel_id.0))
            .set("AUTHORIZATION", &self.token)
            .set(
                "Content-Type",
                &format!("multipart/form-data; boundary={}", mdata.boundary()),
            );
        let response = self
            .rate_limiter
            .send(Some(Route::Channel(channel_id)), request, mdata)?;
        let message = response.into_json()?;
        Ok(message)
    }

    pub fn edit_message<F>(
        &self,
        channel_id: ChannelId,
        message_id: MessageId,
        f: F,
    ) -> Result<Message>
    where
        F: FnOnce(EditMessage) -> EditMessage,
    {
        let edit_msg = f(EditMessage::default());
        let json = serde_json::to_value(edit_msg).unwrap();
        let request = self
            .agent
            .request(
                "PATCH",
                &api!("/channels/{}/messages/{}", channel_id.0, message_id.0),
            )
            .set("AUTHORIZATION", &self.token);
        let response = self.rate_limiter.send_json(
            Some(Route::ChannelMessage(channel_id, message_id)),
            request,
            json,
        )?;
        let message = response.into_json()?;
        Ok(message)
    }

    pub fn delete_message(&self, channel_id: ChannelId, message_id: MessageId) -> Result {
        let request = self
            .agent
            .delete(&api!(
                "/channels/{}/messages/{}",
                channel_id.0,
                message_id.0
            ))
            .set("AUTHORIZATION", &self.token);
        self.rate_limiter
            .call(Some(Route::ChannelMessage(channel_id, message_id)), request)?;
        Ok(())
    }

    pub fn get_guild_channels(&self, guild_id: GuildId) -> Result<Vec<Channel>> {
        let request = self
            .agent
            .get(&api!("/guilds/{}/channels", guild_id.0))
            .set("AUTHORIZATION", &self.token);
        let response = self
            .rate_limiter
            .call(Some(Route::Guild(guild_id)), request)?;
        let channels = response.into_json()?;
        Ok(channels)
    }

    pub fn get_guild_member(&self, guild_id: GuildId, user_id: UserId) -> Result<Option<Member>> {
        let request = self
            .agent
            .get(&api!("/guilds/{}/members/{}", guild_id.0, user_id.0))
            .set("AUTHORIZATION", &self.token);
        let member = match self
            .rate_limiter
            .call(Some(Route::Guild(guild_id)), request)
        {
            Ok(response) => Some(response.into_json()?),
            Err(Error::Ureq(ureq::Error::Status(404, _))) => None,
            Err(err) => return Err(err),
        };
        Ok(member)
    }

    pub fn list_guild_members(&self, guild_id: GuildId) -> Result<Vec<Member>> {
        let request = self
            .agent
            .get(&api!("/guilds/{}/members?limit=1000", guild_id.0))
            .set("AUTHORIZATION", &self.token);
        let response = self
            .rate_limiter
            .call(Some(Route::Guild(guild_id)), request)?;
        let members = response.into_json()?;
        Ok(members)
    }

    pub fn search_guild_members(&self, guild_id: GuildId, query: &str) -> Result<Vec<Member>> {
        let request = self
            .agent
            .get(&api!(
                "/guilds/{}/members/search?query={}&limit=1000",
                guild_id.0,
                query
            ))
            .set("AUTHORIZATION", &self.token);
        let response = self
            .rate_limiter
            .call(Some(Route::Guild(guild_id)), request)?;
        let members = response.into_json()?;
        Ok(members)
    }

    pub fn add_guild_member_role(
        &self,
        guild_id: GuildId,
        user_id: UserId,
        role_id: RoleId,
    ) -> Result {
        let request = self
            .agent
            .put(&api!(
                "/guilds/{}/members/{}/roles/{}",
                guild_id.0,
                user_id.0,
                role_id.0
            ))
            .set("AUTHORIZATION", &self.token)
            .set("Content-Length", "0");
        self.rate_limiter
            .call(Some(Route::GuildMember(guild_id, user_id)), request)?;
        Ok(())
    }

    pub fn remove_guild_member_role(
        &self,
        guild_id: GuildId,
        user_id: UserId,
        role_id: RoleId,
    ) -> Result {
        let request = self
            .agent
            .delete(&api!(
                "/guilds/{}/members/{}/roles/{}",
                guild_id.0,
                user_id.0,
                role_id.0
            ))
            .set("AUTHORIZATION", &self.token);
        self.rate_limiter
            .call(Some(Route::GuildMember(guild_id, user_id)), request)?;
        Ok(())
    }

    pub fn get_guild_roles(&self, guild_id: GuildId) -> Result<Vec<Role>> {
        let request = self
            .agent
            .get(&api!("/guilds/{}/roles", guild_id.0))
            .set("AUTHORIZATION", &self.token);
        let response = self.rate_limiter.call(None, request)?;
        let roles = response.into_json()?;
        Ok(roles)
    }

    pub fn create_guild_role<F>(&self, guild_id: GuildId, f: F) -> Result<Role>
    where
        F: FnOnce(CreateGuildRole) -> CreateGuildRole,
    {
        let role = f(CreateGuildRole::default());
        let json = serde_json::to_value(role).unwrap();
        let request = self
            .agent
            .post(&api!("/guilds/{}/roles", guild_id.0))
            .set("AUTHORIZATION", &self.token);
        let response = self
            .rate_limiter
            .send_json(Some(Route::Guild(guild_id)), request, json)?;
        let role = response.into_json()?;
        Ok(role)
    }

    pub fn delete_guild_role(&self, guild_id: GuildId, role_id: RoleId) -> Result {
        let request = self
            .agent
            .delete(&api!("/guilds/{}/roles/{}", guild_id.0, role_id.0))
            .set("AUTHORIZATION", &self.token);
        self.rate_limiter.call(None, request)?;
        Ok(())
    }

    pub fn create_dm(&self, user_id: UserId) -> Result<Channel> {
        let request = self
            .agent
            .post(&api!("/users/@me/channels"))
            .set("AUTHORIZATION", &self.token);
        let json = json!({ "recipient_id": user_id });
        let response = self.rate_limiter.send_json(None, request, json)?;
        let channel = response.into_json()?;
        Ok(channel)
    }

    pub fn execute_webhook<F>(
        &self,
        webhook_id: WebhookId,
        webhook_token: &str,
        wait: bool,
        f: F,
    ) -> Result<Option<Message>>
    where
        F: FnOnce(ExecuteWebhook) -> ExecuteWebhook,
    {
        let msg = f(ExecuteWebhook::default());
        let json = serde_json::to_value(msg).unwrap();
        let request = self
            .agent
            .post(&api!(
                "/webhooks/{}/{}?wait={}",
                webhook_id.0,
                webhook_token,
                wait
            ))
            .set("AUTHORIZATION", &self.token);
        let response =
            self.rate_limiter
                .send_json(Some(Route::Webhook(webhook_id)), request, json)?;
        Ok(if wait {
            let message = response.into_json()?;
            Some(message)
        } else {
            None
        })
    }

    pub fn delete_webhook_message(
        &self,
        webhook_id: WebhookId,
        webhook_token: &str,
        message_id: MessageId,
    ) -> Result {
        let request = self
            .agent
            .delete(&api!(
                "/webhooks/{}/{}/messages/{}",
                webhook_id.0,
                webhook_token,
                message_id.0
            ))
            .set("AUTHORIZATION", &self.token);
        self.rate_limiter
            .call(Some(Route::Webhook(webhook_id)), request)?;
        Ok(())
    }
}
