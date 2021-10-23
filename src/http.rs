mod create_embed;
mod create_message;
mod edit_message;
mod execute_webhook;

use std::sync::Arc;

pub use ureq::Agent;

use crate::model::id::{ChannelId, MessageId, WebhookId};
use crate::model::Message;
use crate::Result;
pub use create_embed::CreateEmbed;
pub use create_message::CreateMessage;
pub use edit_message::EditMessage;
pub use execute_webhook::ExecuteWebhook;

#[derive(Debug, Clone)]
pub struct Http {
    agent: Agent,
    token: Arc<str>,
}

impl Http {
    pub fn new(token: Arc<str>) -> Self {
        Self {
            agent: Agent::new(),
            token,
        }
    }

    pub fn send_message<F>(&self, channel_id: ChannelId, f: F) -> Result<Message>
    where
        F: FnOnce(&mut CreateMessage) -> &mut CreateMessage,
    {
        let mut msg = CreateMessage::default();
        f(&mut msg);
        let response = self
            .agent
            .post(&api!("/channels/{}/messages", channel_id.0))
            .set("AUTHORIZATION", &self.token)
            .send_json(serde_json::to_value(msg).unwrap())?;
        let message = response.into_json::<Message>()?;
        Ok(message)
    }

    pub fn edit_message<F>(&self, msg: &Message, f: F) -> Result<Message>
    where
        F: FnOnce(&mut EditMessage) -> &mut EditMessage,
    {
        let mut edit_msg = EditMessage::default();
        f(&mut edit_msg);
        let response = self
            .agent
            .request(
                "PATCH",
                &api!("/channels/{}/messages/{}", msg.channel_id.0, msg.id.0),
            )
            .set("AUTHORIZATION", &self.token)
            .send_json(serde_json::to_value(edit_msg).unwrap())?;
        let message = response.into_json::<Message>()?;
        Ok(message)
    }

    pub fn delete_message(&self, channel_id: ChannelId, message_id: MessageId) -> Result {
        self.agent
            .delete(&api!(
                "/channels/{}/messages/{}",
                channel_id.0,
                message_id.0
            ))
            .set("AUTHORIZATION", &self.token)
            .call()?;
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
        F: FnOnce(&mut ExecuteWebhook) -> &mut ExecuteWebhook,
    {
        let mut msg = ExecuteWebhook::default();
        f(&mut msg);
        let response = self
            .agent
            .post(&api!(
                "/webhooks/{}/{}?wait={}",
                webhook_id.0,
                webhook_token,
                wait
            ))
            .set("AUTHORIZATION", &self.token)
            .send_json(serde_json::to_value(msg).unwrap())?;
        Ok(if wait {
            let message = response.into_json::<Message>()?;
            Some(message)
        } else {
            None
        })
    }
}
