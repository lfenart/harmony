use super::{Callback, Context};
use crate::gateway::{DispatchEvent, DispatchEventKind, Ready};
use crate::model::Message;
use std::sync::mpsc::Receiver;
use std::sync::Arc;

use crate::Result;

pub struct EventHandler<'a, E> {
    token: Arc<str>,
    event_receiver: Receiver<DispatchEvent>,
    on_ready: Callback<'a, Ready, std::result::Result<(), E>>,
    on_message_create: Callback<'a, Message, std::result::Result<(), E>>,
    error_handler: Callback<'a, E, ()>,
}

impl<'a, E> EventHandler<'a, E> {
    pub fn new(
        token: Arc<str>,
        event_receiver: Receiver<DispatchEvent>,
        on_ready: Callback<'a, Ready, std::result::Result<(), E>>,
        on_message_create: Callback<'a, Message, std::result::Result<(), E>>,
        error_handler: Callback<'a, E, ()>,
    ) -> Self {
        Self {
            token,
            event_receiver,
            on_ready,
            on_message_create,
            error_handler,
        }
    }

    pub fn run(self) -> Result {
        let context = Context::new(self.token.clone());
        loop {
            let event = self.event_receiver.recv()?;
            let result = match event.kind {
                DispatchEventKind::Ready(ready) => (self.on_ready)(context.clone(), ready),
                DispatchEventKind::MessageCreate(message) => {
                    (self.on_message_create)(context.clone(), *message)
                }
                DispatchEventKind::Unknown(_) => continue,
            };
            if let Err(err) = result {
                (self.error_handler)(context.clone(), err);
            }
        }
    }
}
