use super::{Callback, Context};
use crate::gateway::{DispatchEvent, DispatchEventKind, Ready};
use crate::model::Message;
use std::sync::mpsc::Receiver;
use std::sync::Arc;

use crate::Result;

pub struct EventHandler<'a, E> {
    token: Arc<str>,
    event_receiver: Receiver<DispatchEvent>,
    on_ready: Box<dyn Fn(&Context, &Ready) -> std::result::Result<(), E> + 'a>,
    on_message_create: Box<dyn Fn(&Context, &Message) -> std::result::Result<(), E> + 'a>,
    error_handler: Box<dyn Fn(&Context, &E) + 'a>,
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
                DispatchEventKind::Ready(ready) => (self.on_ready)(&context, &ready),
                DispatchEventKind::MessageCreate(message) => {
                    (self.on_message_create)(&context, &message)
                }
                DispatchEventKind::Unknown(_) => continue,
            };
            if let Err(err) = result {
                (self.error_handler)(&context, &err);
            }
        }
    }
}
