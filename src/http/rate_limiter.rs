use std::collections::HashMap;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, SystemTime};

use multipart::client::lazy::PreparedFields;
use parking_lot::Mutex;
use serde::Deserialize;
use ureq::{Request, Response};

use super::Route;
use crate::Result;

#[derive(Debug, Default, Clone)]
pub struct RateLimiter {
    routes: Arc<Mutex<HashMap<Route, Arc<Mutex<RateLimit>>>>>,
}

impl RateLimiter {
    pub fn new() -> Self {
        Self::default()
    }

    fn lock(&self, route: Route) -> Arc<Mutex<RateLimit>> {
        let mut guard = self.routes.lock();
        guard
            .entry(route)
            .or_insert_with(|| {
                Arc::new(Mutex::new(RateLimit {
                    limit: 0,
                    remaining: 1,
                    reset: SystemTime::UNIX_EPOCH,
                }))
            })
            .clone()
    }

    pub fn call(&self, route: Option<Route>, request: Request) -> Result<Response> {
        let response = if let Some(route) = route {
            let mutex = self.lock(route);
            let mut rate_limit = mutex.lock();
            if rate_limit.remaining == 0 {
                if let Ok(delay) = rate_limit.reset.duration_since(SystemTime::now()) {
                    thread::sleep(delay);
                }
            }
            let response = loop {
                match request.clone().call() {
                    Ok(response) => break response,
                    Err(ureq::Error::Status(429, response)) => {
                        let too_many_requests = response.into_json::<TooManyRequests>()?;
                        thread::sleep(Duration::from_secs_f64(too_many_requests.retry_after));
                    }
                    Err(err) => return Err(err.into()),
                }
            };
            let limit = response
                .header("x-ratelimit-limit")
                .map(|x| x.parse::<u64>().unwrap())
                .unwrap_or(rate_limit.limit);
            let remaining = response
                .header("x-ratelimit-remaining")
                .map(|x| x.parse::<u64>().unwrap())
                .unwrap_or(rate_limit.remaining);
            let reset = response
                .header("x-ratelimit-reset")
                .map(|x| std::time::UNIX_EPOCH + Duration::from_secs_f64(x.parse().unwrap()))
                .unwrap_or(rate_limit.reset);
            *rate_limit = RateLimit {
                limit,
                remaining,
                reset,
            };
            response
        } else {
            loop {
                match request.clone().call() {
                    Ok(response) => break response,
                    Err(ureq::Error::Status(429, response)) => {
                        let too_many_requests = response.into_json::<TooManyRequests>()?;
                        thread::sleep(Duration::from_secs_f64(too_many_requests.retry_after));
                    }
                    Err(err) => return Err(err.into()),
                }
            }
        };
        Ok(response)
    }

    pub fn send(
        &self,
        route: Option<Route>,
        request: Request,
        mut fields: PreparedFields,
    ) -> Result<Response> {
        let response = if let Some(route) = route {
            let mutex = self.lock(route);
            let mut rate_limit = mutex.lock();
            if rate_limit.remaining == 0 {
                if let Ok(delay) = rate_limit.reset.duration_since(SystemTime::now()) {
                    thread::sleep(delay);
                }
            }
            let response = loop {
                match request.clone().send(&mut fields) {
                    Ok(response) => break response,
                    Err(ureq::Error::Status(429, response)) => {
                        let too_many_requests = response.into_json::<TooManyRequests>()?;
                        thread::sleep(Duration::from_secs_f64(too_many_requests.retry_after));
                    }
                    Err(err) => return Err(err.into()),
                }
            };
            let limit = response
                .header("x-ratelimit-limit")
                .map(|x| x.parse::<u64>().unwrap())
                .unwrap_or(rate_limit.limit);
            let remaining = response
                .header("x-ratelimit-remaining")
                .map(|x| x.parse::<u64>().unwrap())
                .unwrap_or(rate_limit.remaining);
            let reset = response
                .header("x-ratelimit-reset")
                .map(|x| std::time::UNIX_EPOCH + Duration::from_secs_f64(x.parse().unwrap()))
                .unwrap_or(rate_limit.reset);
            *rate_limit = RateLimit {
                limit,
                remaining,
                reset,
            };
            response
        } else {
            loop {
                match request.clone().send(&mut fields) {
                    Ok(response) => break response,
                    Err(ureq::Error::Status(429, response)) => {
                        let too_many_requests = response.into_json::<TooManyRequests>()?;
                        thread::sleep(Duration::from_secs_f64(too_many_requests.retry_after));
                    }
                    Err(err) => return Err(err.into()),
                }
            }
        };
        Ok(response)
    }

    pub fn send_json(
        &self,
        route: Option<Route>,
        request: Request,
        json: serde_json::Value,
    ) -> Result<Response> {
        let response = if let Some(route) = route {
            let mutex = self.lock(route);
            let mut rate_limit = mutex.lock();
            if rate_limit.remaining == 0 {
                if let Ok(delay) = rate_limit.reset.duration_since(SystemTime::now()) {
                    thread::sleep(delay);
                }
            }
            let response = loop {
                match request.clone().send_json(json.clone()) {
                    Ok(response) => break response,
                    Err(ureq::Error::Status(429, response)) => {
                        let too_many_requests = response.into_json::<TooManyRequests>()?;
                        thread::sleep(Duration::from_secs_f64(too_many_requests.retry_after));
                    }
                    Err(err) => return Err(err.into()),
                }
            };
            let limit = response
                .header("x-ratelimit-limit")
                .map(|x| x.parse::<u64>().unwrap())
                .unwrap_or(rate_limit.limit);
            let remaining = response
                .header("x-ratelimit-remaining")
                .map(|x| x.parse::<u64>().unwrap())
                .unwrap_or(rate_limit.remaining);
            let reset = response
                .header("x-ratelimit-reset")
                .map(|x| std::time::UNIX_EPOCH + Duration::from_secs_f64(x.parse().unwrap()))
                .unwrap_or(rate_limit.reset);
            *rate_limit = RateLimit {
                limit,
                remaining,
                reset,
            };
            response
        } else {
            loop {
                match request.clone().send_json(json.clone()) {
                    Ok(response) => break response,
                    Err(ureq::Error::Status(429, response)) => {
                        let too_many_requests = response.into_json::<TooManyRequests>()?;
                        thread::sleep(Duration::from_secs_f64(too_many_requests.retry_after));
                    }
                    Err(err) => return Err(err.into()),
                }
            }
        };
        Ok(response)
    }
}

#[derive(Debug)]
struct RateLimit {
    limit: u64,
    remaining: u64,
    reset: SystemTime,
}

#[derive(Debug, Deserialize)]
struct TooManyRequests {
    retry_after: f64,
}
