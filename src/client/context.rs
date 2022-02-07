use std::sync::Arc;

use parking_lot::Mutex;

use crate::gateway::{Gateway, Status};
use crate::http::Http;
use crate::model::Activity;
use crate::Result;

#[derive(Debug, Clone)]
pub struct Context {
    http: Http,
    gateway: Arc<Mutex<Gateway>>,
}

impl Context {
    pub fn new(token: Arc<str>, gateway: Arc<Mutex<Gateway>>) -> Self {
        Self {
            http: Http::new(token),
            gateway,
        }
    }

    pub fn presence_update(&self, status: Status, activity: Option<Activity>) -> Result<()> {
        self.gateway.lock().presence_update(status, activity)
    }
}

impl AsRef<Http> for Context {
    fn as_ref(&self) -> &Http {
        &self.http
    }
}

impl<'a> std::ops::Deref for Context {
    type Target = Http;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}
