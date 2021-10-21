use ureq::Agent;

use crate::http::{CreateMessage, EditMessage};
use crate::model::id::ChannelId;
use crate::model::Message;
use crate::Result;

#[derive(Debug, Clone)]
pub struct Context {
    pub token: String,
    pub agent: Agent,
}

impl Context {
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
}
