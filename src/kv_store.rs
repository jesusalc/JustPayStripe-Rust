use std::collections::HashMap;
use tokio::sync::Mutex;
use std::sync::Arc;

#[derive(Clone)]
pub struct KVStore {
    inner: Arc<Mutex<HashMap<String, String>>>,
}

impl KVStore {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn set(&self, key: &str, value: String) {
        let mut store = self.inner.lock().await;
        store.insert(key.to_string(), value);
    }

    pub async fn get(&self, key: &str) -> Option<String> {
        let store = self.inner.lock().await;
        store.get(key).cloned()
    }
}
