use std::sync::Arc;

use parking_lot::Mutex;
use pwnagotchi_shared::types::events::EventPayload;

use crate::{managers::event_manager::EventManager, traits::events::DynamicEventAPITrait};

#[tokio::test]
async fn sync_listener_receives_payload() {
  let manager = EventManager::new();
  let received = Arc::new(Mutex::new(Vec::new()));
  let capture = Arc::clone(&received);

  {
    let mut api = manager.scope("test_plugin");
    api
      .register_listener(
        "test.event",
        Arc::new(move |payload: &EventPayload| {
          let value: String = payload.deserialize().expect("deserialize string payload");
          capture.lock().push(value);
          Ok(())
        }),
      )
      .expect("register listener");
  }

  manager
    .emit_value("test.event", &"hello".to_string())
    .await
    .expect("emit succeeds");

  let values = received.lock();
  assert_eq!(values.as_slice(), &[String::from("hello")]);
}

#[tokio::test]
async fn async_listener_handles_payload() {
  let manager = EventManager::new();
  let received = Arc::new(Mutex::new(Vec::new()));
  let capture = Arc::clone(&received);

  {
    let mut api = manager.scope("async_plugin");
    api
      .register_async_listener(
        "async.event",
        Arc::new(move |payload: EventPayload| {
          let capture = Arc::clone(&capture);
          Box::pin(async move {
            let value: i32 = payload.deserialize().expect("deserialize i32 payload");
            capture.lock().push(value);
            Ok(())
          })
        }),
      )
      .expect("register async listener");
  }

  manager.emit_value("async.event", &42_i32).await.expect("emit succeeds");

  let values = received.lock();
  assert_eq!(values.as_slice(), &[42]);
}

#[tokio::test]
async fn unregistering_plugin_removes_listeners() {
  let manager = EventManager::new();
  let received = Arc::new(Mutex::new(0));
  let counter = Arc::clone(&received);

  {
    let mut api = manager.scope("cleanup_plugin");
    api
      .register_listener(
        "cleanup.event",
        Arc::new(move |_payload: &EventPayload| {
          *counter.lock() += 1;
          Ok(())
        }),
      )
      .expect("register listener");
  }

  manager.emit_value("cleanup.event", &()).await.expect("emit before cleanup");
  manager.unregister_plugin("cleanup_plugin").expect("cleanup succeeds");
  manager.emit_value("cleanup.event", &()).await.expect("emit after cleanup");

  assert_eq!(*received.lock(), 1);
}

#[tokio::test]
async fn sync_emit_from_non_async_context() {
  let manager = EventManager::new();
  let received = Arc::new(Mutex::new(Vec::new()));
  let capture = Arc::clone(&received);

  {
    let mut api = manager.scope("sync_emit_plugin");
    api
      .register_listener(
        "sync.emit.event",
        Arc::new(move |payload: &EventPayload| {
          let value: String = payload.deserialize().expect("deserialize string payload");
          capture.lock().push(value);
          Ok(())
        }),
      )
      .expect("register listener");
  }

  // Call from non-async function
  fn emit_from_sync_context(manager: &EventManager) {
    manager.emit_value_sync("sync.emit.event", &"from sync".to_string());
  }

  emit_from_sync_context(&manager);

  // Give async tasks time to complete
  tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

  let values = received.lock();
  assert_eq!(values.as_slice(), &[String::from("from sync")]);
}
