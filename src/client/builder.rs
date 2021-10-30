use super::{Callback, Client, Context};
use crate::gateway::{Intents, Ready};
use crate::model::Message;

#[derive(Default)]
pub struct ClientBuilder<'a, E = Box<dyn std::error::Error>> {
    token: Option<String>,
    intents: Intents,
    on_ready: Option<Callback<'a, Ready, Result<(), E>>>,
    on_message_create: Option<Callback<'a, Message, Result<(), E>>>,
    error_handler: Option<Callback<'a, E, ()>>,
}

impl<'a, E> ClientBuilder<'a, E> {
    pub fn new() -> Self {
        Self {
            token: None,
            intents: Intents::default(),
            on_ready: None,
            on_message_create: None,
            error_handler: None,
        }
    }

    pub fn build(self) -> Client<'a, E> {
        Client {
            token: self.token.unwrap(),
            intents: self.intents,
            on_ready: self.on_ready.unwrap_or_else(|| Box::new(|_, _| Ok(()))),
            on_message_create: self
                .on_message_create
                .unwrap_or_else(|| Box::new(|_, _| Ok(()))),
            error_handler: self.error_handler.unwrap_or_else(|| Box::new(|_, _| ())),
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
        F: Fn(Context, Ready) -> Result<(), E> + 'a,
    {
        self.on_ready = Some(Box::new(f));
        self
    }

    pub fn on_message_create<F>(mut self, f: F) -> Self
    where
        F: Fn(Context, Message) -> Result<(), E> + 'a,
    {
        self.on_message_create = Some(Box::new(f));
        self
    }

    pub fn error_handler<F>(mut self, f: F) -> Self
    where
        F: Fn(Context, E) + 'a,
    {
        self.error_handler = Some(Box::new(f));
        self
    }
}
