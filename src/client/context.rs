use std::sync::Arc;

use crate::http::Http;

#[derive(Debug, Clone)]
pub struct Context {
    http: Http,
}

impl Context {
    pub fn new(token: Arc<str>) -> Self {
        Self {
            http: Http::new(token),
        }
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
