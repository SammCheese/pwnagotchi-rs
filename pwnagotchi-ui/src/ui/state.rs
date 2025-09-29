use std::{collections::HashMap, sync::Arc};

use parking_lot::Mutex;
use pwnagotchi_shared::traits::{general::Dependencies, ui::Widget};

type Listener<T> = Box<dyn Fn(T, T) + Send + Sync>;

pub type Element = HashMap<String, Arc<Mutex<Box<dyn Widget>>>>;

#[derive(Clone)]

pub struct State {
  pub elements: Arc<Mutex<Element>>,
  pub listeners: Arc<Mutex<HashMap<String, Listener<String>>>>,
  pub changes: Arc<Mutex<HashMap<String, bool>>>,
}

impl Dependencies for State {
  fn name(&self) -> &'static str {
    "State"
  }
}

impl Default for State {
  fn default() -> Self {
    Self::new()
  }
}

impl State {
  #[must_use]
  pub fn new() -> Self {
    Self {
      elements: Arc::new(Mutex::new(HashMap::new())),
      listeners: Arc::new(Mutex::new(HashMap::new())),
      changes: Arc::new(Mutex::new(HashMap::new())),
    }
  }

  pub fn add_element(&self, key: &str, elem: Arc<Mutex<Box<dyn Widget>>>) {
    self.elements.lock().insert(key.to_string(), elem);

    self.changes.lock().insert(key.to_string(), true);
  }

  #[must_use]
  pub fn has_element(&self, key: &str) -> bool {
    self.elements.lock().contains_key(key)
  }

  pub fn remove_element(&self, key: &str) {
    self.elements.lock().remove(key);

    self.changes.lock().insert(key.to_string(), true);
  }

  pub fn add_listener<F>(&self, key: &str, cb: F)
  where
    F: Fn(String, String) + Send + Sync + 'static,
  {
    self.listeners.lock().insert(key.to_string(), Box::new(cb));
  }

  #[must_use]
  pub fn items(&self) -> Element {
    self.elements.lock().clone()
  }

  #[must_use]
  pub fn get(&self, key: &str) -> Option<Arc<Mutex<Box<dyn Widget>>>> {
    self.elements.lock().get(key).cloned()
  }

  pub fn reset(&self) {
    self.changes.lock().clear();
  }

  #[must_use]
  pub fn changes(&self, ignore: &[&str]) -> Vec<String> {
    let changes = self.changes.lock();

    changes.keys().filter(|k| !ignore.contains(&k.as_str())).cloned().collect()
  }

  #[must_use]
  pub fn has_changes(&self) -> bool {
    !self.changes.lock().is_empty()
  }

  pub fn set(&self, key: &str, value: &str) {
    let prev_value_opt = {
      let elements = self.elements.lock();
      elements.get(key).cloned()
    }
    .map(|elem| {
      let mut widget = elem.lock();
      let prev_value = widget.get_value().to_string();
      let new_value = value;
      widget.set_value(value);
      drop(widget);
      (prev_value, new_value)
    });

    self.changes.lock().insert(key.to_string(), true);

    if let Some((prev_value, new_value)) = prev_value_opt
      && prev_value != new_value
      && let Some(listener) = self.listeners.lock().get(key)
    {
      let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        listener(prev_value, new_value.to_owned());
      }));
    }
  }
}
