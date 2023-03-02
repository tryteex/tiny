use std::{sync::{Arc, Mutex}, collections::HashMap};

use crate::sys::log::Log;

use super::action::Data;

#[derive(Debug)]
pub struct Cache {
    data: HashMap<String, Data>,
}

impl Cache {
    pub fn new() -> Arc<Mutex<Cache>> {
        Arc::new(Mutex::new(Cache {
            data: HashMap::new(),
        }))
    }

    pub fn get(cache: Arc<Mutex<Cache>>, key: &str, log: Arc<Mutex<Log>>) -> Option<Data> {
        match Mutex::lock(&cache) {
            Ok(c) => {
                if let Some(d) = c.data.get(key) {
                    return Some(d.clone());
                 };
                 None
            },
            Err(e) => Log::error(log, e.to_string()),
        }
    }

    pub fn set(cache: Arc<Mutex<Cache>>, key: String, data: Data, log: Arc<Mutex<Log>>) -> Option<Data> {
        match Mutex::lock(&cache) {
            Ok(mut c) => c.data.insert(key, data),
            Err(e) => Log::error(log, e.to_string()),
        }
    }

    pub fn del(cache: Arc<Mutex<Cache>>, key: &str, log: Arc<Mutex<Log>>) {
        match Mutex::lock(&cache) {
            Ok(mut c) => c.data.retain(|k, _| k.starts_with(key)),
            Err(e) => Log::error(log, e.to_string()),
        }
    }

    pub fn clear(cache: Arc<Mutex<Cache>>, log: Arc<Mutex<Log>>) {
        match Mutex::lock(&cache) {
            Ok(mut c) => c.data.clear(),
            Err(e) => Log::error(log, e.to_string()),
        }
    }

}