use std::{collections::HashMap, sync::Arc};
use async_std::sync::Mutex;
use crate::objects::UserInfo;
use surf::Client;

#[derive(Clone)]
pub struct State {
    pub server_ids: Arc<Mutex<HashMap<String, String>>>,
    pub tokens: Arc<Mutex<HashMap<String, UserInfo>>>,
    pub http_client: Client,
}

impl State {
    pub fn new() -> Self {
        Self {
            server_ids: Arc::new(Mutex::new(HashMap::new())),
            tokens: Arc::new(Mutex::new(HashMap::new())),
            http_client: Client::new(),
        }
    }
}