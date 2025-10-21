use std::{
  collections::HashMap,
  error::Error,
  sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
  },
};

use async_trait::async_trait;
use futures::future::join_all;
use parking_lot::{Mutex, RwLock};
use pwnagotchi_shared::{logger::LOGGER, traits::events::EventBus, types::events::EventPayload};

use crate::traits::events::{AsyncEventHandler, DynamicEventAPITrait, EventError, EventHandler};

#[derive(Clone)]
struct ListenerEntry {
  id: u64,
  plugin: String,
  kind: EventListenerKind,
}

#[derive(Clone)]
enum EventListenerKind {
  Sync(EventHandler),
  Async(AsyncEventHandler),
}

#[derive(Clone)]
struct StoredListener {
  event: String,
  id: u64,
}

pub struct EventManager {
  listeners: RwLock<HashMap<String, Vec<Arc<ListenerEntry>>>>,
  plugin_map: Mutex<HashMap<String, Vec<StoredListener>>>,
  counter: AtomicU64,
}

impl Default for EventManager {
  fn default() -> Self {
    Self::new()
  }
}

impl EventManager {
  pub fn new() -> Self {
    Self {
      listeners: RwLock::new(HashMap::new()),
      plugin_map: Mutex::new(HashMap::new()),
      counter: AtomicU64::new(1),
    }
  }

  pub fn scope<'a>(&'a self, plugin: &'a str) -> DynamicEventAPI<'a> {
    DynamicEventAPI {
      manager: self,
      plugin_name: plugin.to_owned(),
      registered_listeners: Vec::new(),
    }
  }

  pub async fn emit_payload_native(
    &self,
    event: &str,
    payload: EventPayload,
  ) -> Result<(), EventError> {
    let entries: Vec<Arc<ListenerEntry>> = {
      let guard = self.listeners.read();
      guard.get(event).cloned().unwrap_or_default()
    };

    if entries.is_empty() {
      return Ok(());
    }

    let mut first_error: Option<EventError> = None;
    let mut async_calls = Vec::new();

    for entry in entries {
      match &entry.kind {
        EventListenerKind::Sync(handler) => {
          if let Err(err) = handler(&payload) {
            let message = format_error(&err);
            LOGGER.log_error(
              "EVENTS",
              &format!(
                "Listener error for event '{event}' in plugin '{}': {message}",
                entry.plugin
              ),
            );
            if first_error.is_none() {
              first_error = Some(EventError::ListenerFailed {
                plugin: entry.plugin.clone(),
                event: event.to_string(),
                message,
              });
            }
          }
        }
        EventListenerKind::Async(handler) => {
          async_calls.push((entry.plugin.clone(), handler(payload.clone())));
        }
      }
    }

    if !async_calls.is_empty() {
      let results =
        join_all(async_calls.into_iter().map(|(plugin, fut)| async move { (plugin, fut.await) }))
          .await;

      for (plugin, result) in results {
        if let Err(err) = result {
          let message = format_error(&err);
          LOGGER.log_error(
            "EVENTS",
            &format!("Async listener error for event '{event}' in plugin '{plugin}': {message}"),
          );
          if first_error.is_none() {
            first_error = Some(EventError::ListenerFailed {
              plugin,
              event: event.to_string(),
              message,
            });
          }
        }
      }
    }

    if let Some(err) = first_error { Err(err) } else { Ok(()) }
  }

  pub async fn emit_value<T>(&self, event: &str, value: &T) -> Result<(), EventError>
  where
    T: serde::Serialize + Send + Sync,
  {
    let payload = EventPayload::new(value).map_err(|e| EventError::ListenerFailed {
      plugin: "<emitter>".to_string(),
      event: event.to_string(),
      message: e.to_string(),
    })?;
    self.emit_payload_native(event, payload).await
  }

  /// Synchronous fire-and-forget event emission.
  /// Spawns a task to emit the event without blocking.
  ///
  /// Use this when you need to emit events from synchronous code.
  /// The event will be processed asynchronously in the background.
  ///
  /// # Example
  /// ```ignore
  /// // From sync context
  /// manager.emit_value_sync("handshake", &data);
  /// ```
  pub fn emit_value_sync<T>(&self, event: &str, value: &T)
  where
    T: serde::Serialize + Send + Sync + 'static,
  {
    let event = event.to_string();
    let payload = match EventPayload::new(value) {
      Ok(p) => p,
      Err(e) => {
        LOGGER.log_error("EVENTS", &format!("Failed to serialize event '{event}': {e}"));
        return;
      }
    };

    let entries: Vec<Arc<ListenerEntry>> = {
      let guard = self.listeners.read();
      guard.get(&event).cloned().unwrap_or_default()
    };

    if entries.is_empty() {
      return;
    }

    // Execute sync listeners immediately
    for entry in &entries {
      if let EventListenerKind::Sync(handler) = &entry.kind
        && let Err(err) = handler(&payload)
      {
        LOGGER.log_error(
          "EVENTS",
          &format!("Sync listener error for event '{event}' in plugin '{}': {}", entry.plugin, err),
        );
      }
    }

    // Spawn task for async listeners
    let async_entries: Vec<_> = entries
      .into_iter()
      .filter_map(|entry| {
        if let EventListenerKind::Async(handler) = &entry.kind {
          Some((entry.plugin.clone(), Arc::clone(handler), payload.clone()))
        } else {
          None
        }
      })
      .collect();

    if !async_entries.is_empty() {
      let event_clone = event.clone();
      tokio::spawn(async move {
        for (plugin, handler, payload) in async_entries {
          if let Err(err) = handler(payload).await {
            LOGGER.log_error(
              "EVENTS",
              &format!(
                "Async listener error for event '{event_clone}' in plugin '{plugin}': {err}"
              ),
            );
          }
        }
      });
    }
  }

  fn register_sync(&self, plugin: &str, event: &str, handler: EventHandler) -> u64 {
    self.register(plugin, event, EventListenerKind::Sync(handler))
  }

  fn register_async(&self, plugin: &str, event: &str, handler: AsyncEventHandler) -> u64 {
    self.register(plugin, event, EventListenerKind::Async(handler))
  }

  fn register(&self, plugin: &str, event: &str, kind: EventListenerKind) -> u64 {
    let id = self.counter.fetch_add(1, Ordering::Relaxed);
    {
      let mut guard = self.listeners.write();
      guard.entry(event.to_string()).or_default().push(Arc::new(ListenerEntry {
        id,
        plugin: plugin.to_string(),
        kind,
      }));
    }

    let mut map = self.plugin_map.lock();
    map
      .entry(plugin.to_string())
      .or_default()
      .push(StoredListener { event: event.to_string(), id });

    id
  }

  fn unregister(&self, plugin: &str, event: &str, id: u64) -> Result<(), EventError> {
    let mut should_remove_key = false;
    let removed = {
      let mut guard = self.listeners.write();
      if let Some(list) = guard.get_mut(event) {
        let len_before = list.len();
        list.retain(|entry| entry.id != id);
        if list.is_empty() {
          should_remove_key = true;
        }
        len_before != list.len()
      } else {
        return Err(EventError::UnregisterFailed(format!("event '{event}' has no listeners")));
      }
    };

    if should_remove_key {
      let mut guard = self.listeners.write();
      guard.remove(event);
    }

    if !removed {
      return Err(EventError::UnregisterFailed(format!(
        "listener {id} not found for event '{event}'"
      )));
    }

    let mut map = self.plugin_map.lock();
    if let Some(entries) = map.get_mut(plugin) {
      entries.retain(|stored| stored.id != id);
      if entries.is_empty() {
        map.remove(plugin);
      }
    }

    Ok(())
  }

  pub fn unregister_plugin(&self, plugin: &str) -> Result<(), EventError> {
    let entries = {
      let mut map = self.plugin_map.lock();
      map.remove(plugin).unwrap_or_default()
    };

    if entries.is_empty() {
      return Ok(());
    }

    let mut guard = self.listeners.write();
    for entry in entries {
      if let Some(list) = guard.get_mut(&entry.event) {
        list.retain(|listener| listener.id != entry.id);
        if list.is_empty() {
          guard.remove(&entry.event);
        }
      }
    }

    Ok(())
  }
}

#[allow(clippy::borrowed_box)]
fn format_error(err: &Box<dyn Error + Send + Sync>) -> String {
  err.to_string()
}

pub struct DynamicEventAPI<'a> {
  pub(crate) manager: &'a EventManager,
  pub(crate) plugin_name: String,
  pub(crate) registered_listeners: Vec<(String, u64)>,
}

impl DynamicEventAPITrait for DynamicEventAPI<'_> {
  fn registered(&self) -> &[(String, u64)] {
    &self.registered_listeners
  }

  fn register_listener(&mut self, event: &str, handler: EventHandler) -> Result<u64, EventError> {
    let id = self.manager.register_sync(&self.plugin_name, event, handler);
    self.registered_listeners.push((event.to_string(), id));
    Ok(id)
  }

  fn register_async_listener(
    &mut self,
    event: &str,
    handler: AsyncEventHandler,
  ) -> Result<u64, EventError> {
    let id = self.manager.register_async(&self.plugin_name, event, handler);
    self.registered_listeners.push((event.to_string(), id));
    Ok(id)
  }

  fn cleanup(&mut self) -> Result<(), EventError> {
    self.manager.unregister_plugin(&self.plugin_name)?;
    self.registered_listeners.clear();
    Ok(())
  }

  fn unregister(&mut self, listener_id: &u64) -> Result<(), EventError> {
    if let Some((event, _)) =
      self.registered_listeners.iter().find(|(_, id)| id == listener_id).cloned()
    {
      self.manager.unregister(&self.plugin_name, &event, *listener_id)?;
      self.registered_listeners.retain(|(_, id)| id != listener_id);
      Ok(())
    } else {
      Err(EventError::UnregisterFailed(format!(
        "listener {listener_id} not registered for plugin '{}'",
        self.plugin_name
      )))
    }
  }
}

#[async_trait]
impl EventBus for EventManager {
  async fn emit_payload(
    &self,
    event: &str,
    payload: EventPayload,
  ) -> Result<(), Box<dyn Error + Send + Sync>> {
    self
      .emit_payload_native(event, payload)
      .await
      .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)
  }
}
