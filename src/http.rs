mod create_embed;
mod create_message;
mod edit_message;
mod execute_webhook;
mod rate_limiter;
mod route;

use std::fs::File;
use std::path::Path;
use std::sync::Arc;

use multipart::client::lazy::Multipart;
use ureq::Agent;

use crate::model::id::{ChannelId, MessageId, WebhookId};
use crate::model::Message;
use crate::Result;
pub use create_embed::CreateEmbed;
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

    pub fn send_message<F>(&self, channel_id: ChannelId, f: F) -> Result<Message>
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
        let message = response.into_json::<Message>()?;
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
        let message = response.into_json::<Message>()?;
        Ok(message)
    }

    pub fn edit_message<F>(&self, msg: &Message, f: F) -> Result<Message>
    where
        F: FnOnce(EditMessage) -> EditMessage,
    {
        let edit_msg = f(EditMessage::default());
        let json = serde_json::to_value(edit_msg).unwrap();
        let request = self
            .agent
            .request(
                "PATCH",
                &api!("/channels/{}/messages/{}", msg.channel_id.0, msg.id.0),
            )
            .set("AUTHORIZATION", &self.token);
        let response = self.rate_limiter.send_json(
            Some(Route::ChannelMessage(msg.channel_id, msg.id)),
            request,
            json,
        )?;
        let message = response.into_json::<Message>()?;
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
            let message = response.into_json::<Message>()?;
            Some(message)
        } else {
            None
        })
    }

    pub fn webhook_delete_message(
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
