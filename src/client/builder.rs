use super::{Client, Context};
use crate::gateway::Ready;
use crate::model::Message;
use crate::Result;

#[derive(Default)]
pub struct ClientBuilder<'a> {
    token: Option<String>,
    on_ready: Option<Box<dyn Fn(&Context, &Ready) -> Result + 'a>>,
    on_message_create: Option<Box<dyn Fn(&Context, &Message) -> Result + 'a>>,
}

impl<'a> ClientBuilder<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn build(self) -> Client<'a> {
        Client {
            token: self.token.unwrap(),
            on_ready: self.on_ready.unwrap_or_else(|| Box::new(|_, _| Ok(()))),
            on_message_create: self
                .on_message_create
                .unwrap_or_else(|| Box::new(|_, _| Ok(()))),
        }
    }

    pub fn with_bot_token(mut self, token: &str) -> Self {
        self.token = Some(format!("Bot {}", token));
        self
    }

    pub fn with_bearer_token(mut self, token: &str) -> Self {
        self.token = Some(format!("Bearer {}", token));
        self
    }

    pub fn on_ready<F>(mut self, f: F) -> Self
    where
        F: Fn(&Context, &Ready) -> Result<()> + 'a,
    {
        self.on_ready = Some(Box::new(f));
        self
    }

    pub fn on_message_create<F>(mut self, f: F) -> Self
    where
        F: Fn(&Context, &Message) -> Result<()> + 'a,
    {
        self.on_message_create = Some(Box::new(f));
        self
    }
}
