#[cfg(test)]
mod manager_tests {
  use std::sync::Arc;

  // We need to include Hookables lmao
  #[allow(unused_imports)]
  use pwnagotchi_core::agent::Agent;
  use pwnagotchi_shared::types::hooks::{AsyncBeforeHook, BeforeHook, BeforeHookResult, HookArgs};

  use crate::{
    async_before_hook, before_hook,
    managers::hook_manager::HookManager,
    traits::hooks::{DynamicHookAPITrait, HookError},
  };

  #[test]
  fn available_hooks_are_discoverable() {
    let manager = HookManager::new();
    let hooks = manager.available_hooks();
    assert!(!hooks.is_empty(), "Expected at least one hook descriptor");
  }

  #[test]
  fn registering_unknown_hook_returns_error() {
    let manager = HookManager::new();
    let mut api = manager.scope("test_plugin");

    let callback =
      before_hook!(|args: &mut HookArgs| { Ok(BeforeHookResult::Continue(args.unmut())) });

    let err = api.register_before_sync("Missing::hook", callback).unwrap_err();
    assert_eq!(err, HookError::UnknownHook("Missing::hook".to_string()));
  }

  #[test]
  fn can_register_and_unregister_known_hook() {
    let manager = HookManager::new();
    let mut api = manager.scope("test_plugin");

    let callback: BeforeHook =
      before_hook!(|args: &mut HookArgs| { Ok(BeforeHookResult::Continue(args.unmut())) });

    api
      .register_before_sync("Agent::restart", callback)
      .expect("registration should succeed");

    manager.unregister_plugin("test_plugin").expect("unregistration should succeed");
  }

  #[test]
  fn unregistering_unknown_plugin_is_noop() {
    let manager = HookManager::new();
    assert!(manager.unregister_plugin("non_existent_plugin").is_ok());
  }

  #[test]
  fn hooks_automatically_unregistered_on_drop() {
    let manager = HookManager::new();
    {
      let mut api = manager.scope("temp_plugin");

      let callback =
        before_hook!(|args: &mut HookArgs| { Ok(BeforeHookResult::Continue(args.unmut())) });

      api
        .register_before_sync("Agent::restart", callback)
        .expect("registration should succeed");

      assert_eq!(api.registered_hooks.len(), 1, "One hook should be registered");
    } // out of scope -> dropped

    assert!(manager.registrations.lock().unwrap().get("temp_plugin").is_none());
  }

  #[test]
  fn can_register_and_unregister_individual_hooks() {
    let manager = HookManager::new();
    let mut api = manager.scope("test_plugin");

    let callback1: BeforeHook =
      Arc::new(|args: &mut HookArgs| Ok(BeforeHookResult::Continue(args.unmut())));

    let callback2: BeforeHook =
      Arc::new(|args: &mut HookArgs| Ok(BeforeHookResult::Continue(args.unmut())));

    let id1 = api
      .register_before_sync("Agent::restart", callback1)
      .expect("first registration should succeed");
    let _id2 = api
      .register_before_sync("Agent::reboot", callback2)
      .expect("second registration should succeed");

    assert_eq!(api.registered_hooks.len(), 2);

    api.unregister(&id1).expect("unregister_hook should succeed");

    assert_eq!(api.registered_hooks.len(), 1, "One hook should remain registered");

    assert!(
      api.registered_hooks.iter().all(|(name, _id)| name == "Agent::reboot"),
      "First hook should be unregistered"
    );
  }

  #[test]
  fn unregister_plugin_removes_all_hooks() {
    let manager = HookManager::new();
    let mut api = manager.scope("plugin_a");

    let callback1: BeforeHook = before_hook!(|_args: &mut HookArgs| { Ok(BeforeHookResult::Stop) });
    let callback2: AsyncBeforeHook = async_before_hook!(|args: &mut HookArgs| {
      let owned_args = args.unmut();
      async move { Ok(BeforeHookResult::Continue(owned_args)) }
    });

    api.register_before_sync("Agent::restart", callback1).unwrap();
    api.register_before_async("Agent::set_mode", callback2).unwrap();

    assert_eq!(api.registered_hooks.len(), 2);

    api.cleanup().expect("cleanup should succeed");

    assert_eq!(api.registered_hooks.len(), 0, "All hooks should be unregistered");
  }

  #[test]
  fn multiple_plugins_isolated() {
    let manager = HookManager::new();

    {
      let mut api_a = manager.scope("plugin_a");
      let mut api_b = manager.scope("plugin_b");

      let callback: BeforeHook =
        Arc::new(|args: &mut HookArgs| Ok(BeforeHookResult::Continue(args.unmut())));

      api_a.register_before_sync("Agent::restart", callback.clone()).unwrap();
      api_b.register_before_sync("Agent::restart", callback).unwrap();

      {
        let guard = manager.registrations.lock().unwrap();
        assert_eq!(guard.get("plugin_a").unwrap().len(), 1);
        assert_eq!(guard.get("plugin_b").unwrap().len(), 1);
      }

      drop(api_a);

      {
        let guard = manager.registrations.lock().unwrap();
        assert!(!guard.contains_key("plugin_a"));
        assert_eq!(guard.get("plugin_b").unwrap().len(), 1);
      }
    }

    {
      let guard = manager.registrations.lock().unwrap();
      assert!(guard.is_empty());
    }
  }
}
