use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

type Listener<T> = Box<dyn Fn(T, T) + Send + Sync>;

#[derive(Clone)]
pub struct Element<T: Clone> {
    pub value: T,
}

pub struct State<T: Clone + Send + Sync + 'static> {
    elements: Arc<Mutex<HashMap<String, Element<T>>>>,
    listeners: Arc<Mutex<HashMap<String, Listener<T>>>>,
    changes: Arc<Mutex<HashMap<String, bool>>>,
}

impl Default for State<String> {
    fn default() -> Self {
        Self::new()
    }
    
}

impl<T: Clone + PartialEq + Send + Sync + 'static> State<T> {
    pub fn new() -> Self {
        Self {
            elements: Arc::new(Mutex::new(HashMap::new())),
            listeners: Arc::new(Mutex::new(HashMap::new())),
            changes: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn add_element(&self, key: &str, elem: Element<T>) {
        self.elements.lock().await.insert(key.to_string(), elem);
        self.changes.lock().await.insert(key.to_string(), true);
    }

    pub async fn has_element(&self, key: &str) -> bool {
        self.elements.lock().await.contains_key(key)
    }

    pub async fn remove_element(&self, key: &str) {
        self.elements.lock().await.remove(key);
        self.changes.lock().await.insert(key.to_string(), true);
    }

    pub async fn add_listener<F>(&self, key: &str, cb: F)
    where
        F: Fn(T, T) + Send + Sync + 'static,
    {
        self.listeners.lock().await.insert(key.to_string(), Box::new(cb));
    }

    pub async fn items(&self) -> HashMap<String, Element<T>> {
        self.elements.lock().await.clone()
    }

    pub async fn get(&self, key: &str) -> Option<T> {
        self.elements.lock().await.get(key).map(|e| e.value.clone())
    }

    pub async fn reset(&self) {
        self.changes.lock().await.clear();
    }

    pub async fn changes(&self, ignore: &[&str]) -> Vec<String> {
        let changes = self.changes.lock().await;
        changes
            .keys()
            .filter(|k| !ignore.contains(&k.as_str()))
            .cloned()
            .collect()
    }

    pub async fn has_changes(&self) -> bool {
        !self.changes.lock().await.is_empty()
    }

    pub async fn set(&self, key: &str, value: T) {
        let mut elements = self.elements.lock().await;
        if let Some(elem) = elements.get_mut(key) {
            let prev = elem.value.clone();
            if prev != value {
                elem.value = value.clone();
                self.changes.lock().await.insert(key.to_string(), true);

                if let Some(listener) = self.listeners.lock().await.get(key) {
                    listener(prev, value);
                }
            }
        }
    }
}
