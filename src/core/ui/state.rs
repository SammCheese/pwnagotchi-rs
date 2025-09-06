use std::{
  collections::HashMap,
  sync::{Arc, Mutex},
};

use crate::core::ui::{components::Widget, view::FaceType};

type Listener<T> = Box<dyn Fn(T, T) + Send + Sync>;

#[derive(Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Debug)]
pub enum StateValue {
  None,
  Face(FaceType),
  Text(String),
  Number(u64),
  Bool(bool),
}

pub type Element = HashMap<String, Arc<Mutex<dyn Widget>>>;

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

  pub fn add_element(&self, key: &str, elem: Arc<Mutex<dyn Widget>>) {
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
    self.elements.lock().unwrap_or_else(std::sync::PoisonError::into_inner).clone()
  }

  pub fn get(&self, key: &str) -> Option<Arc<Mutex<dyn Widget>>> {
    self
      .elements
      .lock()
      .unwrap_or_else(std::sync::PoisonError::into_inner)
      .get(key)
      .cloned()
  }

  pub fn reset(&self) {
    self.changes.lock().unwrap_or_else(std::sync::PoisonError::into_inner).clear();
  }

  pub fn changes(&self, ignore: &[&str]) -> Vec<String> {
    let changes = self.changes.lock().unwrap_or_else(std::sync::PoisonError::into_inner);

    changes.keys().filter(|k| !ignore.contains(&k.as_str())).cloned().collect()
  }

  pub fn has_changes(&self) -> bool {
    !self
      .changes
      .lock()
      .unwrap_or_else(std::sync::PoisonError::into_inner)
      .is_empty()
  }

  pub fn set(&self, key: &str, value: StateValue) {
    let prev_value_opt = {
      let elements = self.elements.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
      elements.get(key).cloned()
    }
    .map(|elem| {
      let mut widget = elem.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
      let prev_value = widget.get_value();
      let new_value = value.clone();
      widget.set_value(value);
      drop(widget);
      (prev_value, new_value)
    });

    self
      .changes
      .lock()
      .unwrap_or_else(std::sync::PoisonError::into_inner)
      .insert(key.to_string(), true);

    if let Some((prev_value, new_value)) = prev_value_opt
      && prev_value != new_value
      && let Some(listener) = self
        .listeners
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .get(key)
    {
      let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        listener(prev_value, new_value);
      }));
    }
  }
}
