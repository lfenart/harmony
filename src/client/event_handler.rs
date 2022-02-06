use std::sync::Arc;

use crossbeam_channel::Receiver;
use parking_lot::Mutex;

use super::{Callback, Context};
use crate::gateway::{DispatchEvent, DispatchEventKind, Gateway, Ready};
use crate::model::Message;
use crate::Result;

pub struct EventHandler<'a> {
    token: Arc<str>,
    event_receiver: Receiver<DispatchEvent>,
    gateway: Arc<Mutex<Gateway>>,
    on_ready: Callback<'a, Ready>,
    on_message_create: Callback<'a, Message>,
}

impl<'a> EventHandler<'a> {
    pub fn new(
        token: Arc<str>,
        event_receiver: Receiver<DispatchEvent>,
        gateway: Arc<Mutex<Gateway>>,
        on_ready: Callback<'a, Ready>,
        on_message_create: Callback<'a, Message>,
    ) -> Self {
        Self {
            token,
            event_receiver,
            gateway,
            on_ready,
            on_message_create,
        }
    }

    pub fn run(self) -> Result {
        let context = Context::new(self.token.clone(), self.gateway.clone());
        loop {
            let event = self.event_receiver.recv()?;
            match event.kind {
                DispatchEventKind::Ready(ready) => (self.on_ready.lock())(context.clone(), ready),
                DispatchEventKind::MessageCreate(message) => {
                    (self.on_message_create.lock())(context.clone(), *message)
                }
                DispatchEventKind::Unknown(_) => continue,
            };
        }
    }
}
