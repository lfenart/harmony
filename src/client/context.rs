use std::sync::Arc;
use std::time::SystemTime;

use parking_lot::Mutex;
use serde_json::json;

use crate::gateway::Gateway;
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

    pub fn set_activity(&self, activity: Activity) -> Result<()> {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        let map = json!({
            "op": 3,
            "d": {
                "since": now,
                "activities": [
                    activity
                ],
                "status": "online",
                "afk": false
            }
        });
        let message = tungstenite::Message::Text(serde_json::to_string(&map).unwrap());
        self.gateway.lock().write_message(message)?;
        Ok(())
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
