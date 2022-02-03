use std::sync::Arc;

use parking_lot::Mutex;

use super::{Callback, Client, Context};
use crate::gateway::{Intents, Ready};
use crate::model::Message;

#[derive(Default)]
pub struct ClientBuilder<'a> {
    token: Option<String>,
    intents: Intents,
    on_ready: Option<Callback<'a, Ready>>,
    on_message_create: Option<Callback<'a, Message>>,
}

impl<'a> ClientBuilder<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn build(self) -> Client<'a> {
        Client {
            token: self.token.unwrap(),
            intents: self.intents,
            on_ready: self
                .on_ready
                .unwrap_or_else(|| Arc::new(Mutex::new(|_, _| ()))),
            on_message_create: self
                .on_message_create
                .unwrap_or_else(|| Arc::new(Mutex::new(|_, _| ()))),
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

    pub fn intents(mut self, intents: Intents) -> Self {
        self.intents = intents;
        self
    }

    pub fn on_ready<F>(mut self, f: F) -> Self
    where
        F: FnMut(Context, Ready) + 'a,
    {
        self.on_ready = Some(Arc::new(Mutex::new(f)));
        self
    }

    pub fn on_message_create<F>(mut self, f: F) -> Self
    where
        F: FnMut(Context, Message) + 'a,
    {
        self.on_message_create = Some(Arc::new(Mutex::new(f)));
        self
    }
}
