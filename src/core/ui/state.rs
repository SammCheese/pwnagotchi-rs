use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use std::panic::{catch_unwind, AssertUnwindSafe};

use crate::core::ui::components::Widget;
use crate::core::ui::view::FaceType;

type Listener<T> = Box<dyn Fn(T, T) + Send + Sync>;


#[derive(Clone, PartialEq, Eq)]
pub enum StateValue {
    None,
    Face(FaceType),
    Text(String),
    Number(u64),
    Bool(bool),
}

pub type Element = HashMap<String, Arc<dyn Widget>>;

#[derive(Clone)]
pub struct State {
    pub elements: Arc<Mutex<Element>>,
    pub listeners: Arc<Mutex<HashMap<String, Listener<StateValue>>>>,
    pub changes: Arc<Mutex<HashMap<String, bool>>>,
}


impl Default for State {
    fn default() -> Self {
        Self::new()
    }
    
}

impl State {
    pub fn new() -> Self {
        Self {
            elements: Arc::new(Mutex::new(HashMap::new())),
            listeners: Arc::new(Mutex::new(HashMap::new())),
            changes: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn add_element(&self, key: &str, elem: Arc<dyn Widget>) {
        self
            .elements
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .insert(key.to_string(), elem);
        self
            .changes
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .insert(key.to_string(), true);
    }

    pub fn has_element(&self, key: &str) -> bool {
        self
            .elements
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .contains_key(key)
    }

    pub fn remove_element(&self, key: &str) {
        self
            .elements
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .remove(key);
        self
            .changes
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .insert(key.to_string(), true);
    }

    pub fn add_listener<F>(&self, key: &str, cb: F)
    where
        F: Fn(StateValue, StateValue) + Send + Sync + 'static,
    {
        self
            .listeners
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .insert(key.to_string(), Box::new(cb));
    }

    pub fn items(&self) -> Element {
        self
            .elements
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }

    pub fn get(&self, key: &str) -> Option<Arc<dyn Widget>> {
        self
            .elements
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get(key).cloned()
    }

    pub fn reset(&self) {
        self
            .changes
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clear();
    }

    pub fn changes(&self, ignore: &[&str]) -> Vec<String> {
        let changes = self
            .changes
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        changes
            .keys()
            .filter(|k| !ignore.contains(&k.as_str()))
            .cloned()
            .collect()
    }

    pub fn has_changes(&self) -> bool {
        !self
            .changes
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .is_empty()
    }

    pub fn set(&self, key: &str, value: &StateValue) {
        let mut elements = self
            .elements
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(elem) = elements.get_mut(key) {
            let widget = Arc::get_mut(elem).expect("Widget Arc should be uniquely owned for mutation");
            let prev_value = widget.get_value();
            if prev_value != *value {
                self
                    .changes
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .insert(key.to_string(), true);

                if let Some(listener) = self
                    .listeners
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .get(key)
                {
                    let _ = catch_unwind(AssertUnwindSafe(|| (listener)(prev_value, value.clone())));
                }
            }
        }
    }
}
