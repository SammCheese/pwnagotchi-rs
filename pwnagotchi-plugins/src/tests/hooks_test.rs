use std::sync::Once;

use pwnagotchi_macros::hookable;

fn ensure_test_config() {
  static INIT: Once = Once::new();
  INIT.call_once(|| {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../test/config.toml");
    pwnagotchi_shared::config::init_config(path);
  });
}

#[cfg(test)]
pub mod hook_macro_helper_tests {
  #[allow(unused_imports)]
  use pwnagotchi_core::agent::Agent;
  use pwnagotchi_shared::{
    models::agent::RunningMode,
    types::hooks::{
      AfterHook, AfterHookResult, AsyncAfterHook, AsyncBeforeHook, AsyncInsteadHook, BeforeHook,
      BeforeHookResult, HookArgs, HookReturn, InsteadHook, InsteadHookResult,
    },
  };
  use serial_test::serial;

  use super::ensure_test_config;
  use crate::{
    after_hook, async_after_hook, async_before_hook, async_instead_hook, before_hook, instead_hook,
    managers::hook_manager::HookManager, traits::hooks::DynamicHookAPITrait,
  };

  #[test]
  fn sync_before_hook() {
    ensure_test_config();
    let manager = HookManager::new();
    let mut api = manager.scope("test_plugin");

    // Sync before hook for Agent::restart
    let before: BeforeHook = before_hook!(|args: &mut HookArgs| {
      println!("BEFORE: Agent::restart called with {} args", args.len());
      Ok(BeforeHookResult::Continue(args.unmut()))
    });

    api
      .register_before_sync("Agent::restart", before)
      .expect("Should register sync before hook");
    api.cleanup().expect("Should cleanup hooks");
  }

  #[test]
  fn sync_after_hook() {
    ensure_test_config();
    let manager = HookManager::new();
    let mut api = manager.scope("test_plugin");

    // Sync after hook for Agent::should_interact (returns bool)
    let after: AfterHook = after_hook!(|args: &mut HookArgs, ret: &mut HookReturn| {
      let _bssid: &String = args.get_by_downcast(1).unwrap();
      let result: &bool = ret.get().unwrap();
      println!("AFTER: Agent::should_interact returned {}", result);
      Ok(AfterHookResult::Continue(HookReturn::new(*result)))
    });

    api
      .register_after_sync("Agent::should_interact", after)
      .expect("Should register sync after hook");
    api.cleanup().expect("Should cleanup hooks");
  }

  #[test]
  fn sync_instead_hook() {
    ensure_test_config();
    let manager = HookManager::new();
    let mut api = manager.scope("test_plugin");

    // Sync instead hook for Agent::reboot (void return)
    let instead: InsteadHook = instead_hook!(|args: HookArgs| {
      println!("INSTEAD: Skipping Agent::reboot, args count: {}", args.len());
      // Return a value to skip the original function
      Ok(InsteadHookResult::Return(HookReturn::new(())))
    });

    api
      .register_instead_sync("Agent::reboot", instead)
      .expect("Should register sync instead hook");
    api.cleanup().expect("Should cleanup hooks");
  }

  #[test]
  #[serial(agent_set_mode)]
  fn async_before_hook() {
    ensure_test_config();
    let manager = HookManager::new();
    let mut api = manager.scope("test_plugin");

    // Async before hook for Agent::set_mode
    let before: AsyncBeforeHook = async_before_hook!(|args: &mut HookArgs| {
      let owned_args = args.unmut();
      async move {
        println!("ASYNC BEFORE: Agent::set_mode called");
        Ok(BeforeHookResult::Continue(owned_args))
      }
    });

    api
      .register_before_async("Agent::set_mode", before)
      .expect("Should register async before hook");
    api.cleanup().expect("Should cleanup hooks");
  }

  #[test]
  fn async_after_hook() {
    ensure_test_config();
    let manager = HookManager::new();
    let mut api = manager.scope("test_plugin");

    // Async after hook for Agent::recon (void return)
    let after: AsyncAfterHook = async_after_hook!(|args: &mut HookArgs, ret: &mut HookReturn| {
      let owned_args = args.unmut();
      let owned_ret = std::mem::replace(ret, HookReturn::new(()));
      async move {
        println!("ASYNC AFTER: Agent::recon completed, {} args", owned_args.len());
        Ok(AfterHookResult::Continue(owned_ret))
      }
    });

    api
      .register_after_async("Agent::recon", after)
      .expect("Should register async after hook");
    api.cleanup().expect("Should cleanup hooks");
  }

  #[test]
  #[serial(agent_set_mode)]
  fn async_instead_hook() {
    ensure_test_config();
    let manager = HookManager::new();
    let mut api = manager.scope("test_plugin");

    let instead: AsyncInsteadHook = async_instead_hook!(|args: HookArgs| {
      async move {
        let mode: &RunningMode = args.get_by_downcast(1).unwrap();
        println!("ASYNC INSTEAD: Preventing mode change to {:?}", mode);
        Ok(InsteadHookResult::Return(HookReturn::new(())))
      }
    });

    api
      .register_instead_async("Agent::set_mode", instead)
      .expect("Should register async instead hook");
    api.cleanup().expect("Should cleanup hooks");
  }
}

#[cfg(test)]
pub mod hook_syntax_tests {

  use std::sync::Arc;

  use pwnagotchi_shared::{
    models::agent::RunningMode,
    types::hooks::{
      AfterHook, AfterHookResult, AsyncAfterHook, AsyncBeforeHook, AsyncInsteadHook, BeforeHook,
      BeforeHookResult, HookArgs, HookReturn, InsteadHook, InsteadHookResult,
    },
  };
  use serial_test::serial;

  use super::ensure_test_config;
  use crate::{managers::hook_manager::HookManager, traits::hooks::DynamicHookAPITrait};

  #[test]
  fn sync_before_hook() {
    ensure_test_config();
    let manager = HookManager::new();
    let mut api = manager.scope("test_plugin");

    // Sync before hook for Agent::restart
    let before: BeforeHook = Arc::new(|args: &mut HookArgs| {
      println!("BEFORE: Agent::restart called with {} args", args.len());
      Ok(BeforeHookResult::Continue(args.unmut()))
    });

    api
      .register_before("Agent::restart", Box::new(before))
      .expect("Should register sync before hook");
    api.cleanup().expect("Should cleanup hooks");
  }

  #[test]
  fn sync_after_hook() {
    ensure_test_config();
    let manager = HookManager::new();
    let mut api = manager.scope("test_plugin");

    // Sync after hook for Agent::should_interact (returns bool)
    let after: AfterHook = Arc::new(|args: &mut HookArgs, ret: &mut HookReturn| {
      let _bssid: &String = args.get_by_downcast(1).unwrap();
      let result: &bool = ret.get().unwrap();
      println!("AFTER: Agent::should_interact returned {}", result);
      Ok(AfterHookResult::Continue(HookReturn::new(*result)))
    });

    api
      .register_after("Agent::should_interact", Box::new(after))
      .expect("Should register sync after hook");
    api.cleanup().expect("Should cleanup hooks");
  }

  #[test]
  fn sync_instead_hook() {
    ensure_test_config();
    let manager = HookManager::new();
    let mut api = manager.scope("test_plugin");

    // Sync instead hook for Agent::reboot (void return)
    let instead: InsteadHook = Arc::new(|args: HookArgs| {
      println!("INSTEAD: Skipping Agent::reboot, args count: {}", args.len());
      // Return a value to skip the original function
      Ok(InsteadHookResult::Return(HookReturn::new(())))
    });

    api
      .register_instead("Agent::reboot", Box::new(instead))
      .expect("Should register sync instead hook");
    api.cleanup().expect("Should cleanup hooks");
  }

  #[test]
  #[serial(agent_set_mode)]
  fn async_before_hook() {
    ensure_test_config();
    let manager = HookManager::new();
    let mut api = manager.scope("test_plugin");

    // Async before hook for Agent::set_mode
    let before: AsyncBeforeHook = Arc::new(|args: &mut HookArgs| {
      let owned_args = args.unmut();
      Box::pin(async move {
        println!("ASYNC BEFORE: Agent::set_mode called");
        Ok(BeforeHookResult::Continue(owned_args))
      })
        as std::pin::Pin<
          Box<
            dyn std::future::Future<
                Output = Result<BeforeHookResult, Box<dyn std::error::Error + Send + Sync>>,
              > + Send,
          >,
        >
    });

    api
      .register_before("Agent::set_mode", Box::new(before))
      .expect("Should register async before hook");
    api.cleanup().expect("Should cleanup hooks");
  }

  #[test]
  fn async_after_hook() {
    ensure_test_config();
    let manager = HookManager::new();
    let mut api = manager.scope("test_plugin");

    // Async after hook for Agent::recon (void return)
    let after: AsyncAfterHook = Arc::new(|args: &mut HookArgs, ret: &mut HookReturn| {
      let owned_args = args.unmut();
      let owned_ret = std::mem::replace(ret, HookReturn::new(()));
      Box::pin(async move {
        println!("ASYNC AFTER: Agent::recon completed, {} args", owned_args.len());
        Ok(AfterHookResult::Continue(owned_ret))
      })
        as std::pin::Pin<
          Box<
            dyn std::future::Future<
                Output = Result<AfterHookResult, Box<dyn std::error::Error + Send + Sync>>,
              > + Send,
          >,
        >
    });

    api
      .register_after("Agent::recon", Box::new(after))
      .expect("Should register async after hook");
    api.cleanup().expect("Should cleanup hooks");
  }

  #[test]
  #[serial(agent_set_mode)]
  fn async_instead_hook() {
    ensure_test_config();
    let manager = HookManager::new();
    let mut api = manager.scope("test_plugin");

    let instead: AsyncInsteadHook = Arc::new(|args: HookArgs| {
      Box::pin(async move {
        let mode: &RunningMode = args.get_by_downcast(1).unwrap();
        println!("ASYNC INSTEAD: Preventing mode change to {:?}", mode);
        Ok(InsteadHookResult::Return(HookReturn::new(())))
      })
        as std::pin::Pin<
          Box<
            dyn std::future::Future<
                Output = Result<InsteadHookResult, Box<dyn std::error::Error + Send + Sync>>,
              > + Send,
          >,
        >
    });

    api
      .register_instead("Agent::set_mode", Box::new(instead))
      .expect("Should register async instead hook");
    api.cleanup().expect("Should cleanup hooks");
  }
}

#[cfg(test)]
pub mod hook_behavior_tests {
  use std::sync::Arc;

  // We need these imports for the tests even if unused for the linker
  #[allow(unused_imports)]
  use pwnagotchi_core::{agent::Agent, utils};
  use pwnagotchi_shared::types::hooks::{
    AfterHook, AfterHookResult, BeforeHook, BeforeHookResult, HookArgs, HookReturn, InsteadHook,
    InsteadHookResult,
  };
  use serial_test::serial;

  use super::ensure_test_config;
  use crate::{
    instead_hook, managers::hook_manager::HookManager, tests::hooks_test::test_add,
    traits::hooks::DynamicHookAPITrait,
  };

  #[test]
  fn before_hook_can_stop_execution() {
    ensure_test_config();
    let manager = HookManager::new();
    let mut api = manager.scope("test_plugin");

    let before: BeforeHook = Arc::new(|_args: &mut HookArgs| {
      // Return Stop to prevent the original function from running
      Ok(BeforeHookResult::Stop)
    });

    api
      .register_before("Agent::restart", Box::new(before))
      .expect("Should register");
    api.cleanup().expect("Should cleanup hooks");
  }

  #[test]
  fn after_hook_can_modify_return_value() {
    ensure_test_config();
    let manager = HookManager::new();
    let mut api = manager.scope("test_plugin");

    let after: AfterHook = Arc::new(|_args: &mut HookArgs, ret: &mut HookReturn| {
      // Modify the return value
      let original: &bool = ret.get().unwrap();
      let modified = !original; // Flip the boolean
      Ok(AfterHookResult::Continue(HookReturn::new(modified)))
    });

    api
      .register_after("Agent::should_interact", Box::new(after))
      .expect("Should register");
    api.cleanup().expect("Should cleanup hooks");
  }

  #[test]
  fn instead_hook_can_replace_function() {
    ensure_test_config();
    let manager = HookManager::new();
    let mut api = manager.scope("test_plugin");

    let instead: InsteadHook = Arc::new(|args: HookArgs| {
      // Completely replace the function logic
      let bssid: &String = args.get_by_downcast(1).unwrap();
      let custom_result = bssid.starts_with("AA:");
      Ok(InsteadHookResult::Return(HookReturn::new(custom_result)))
    });

    api
      .register_instead("Agent::should_interact", Box::new(instead))
      .expect("Should register");
    api.cleanup().expect("Should cleanup hooks");
  }

  #[test]
  #[serial]
  fn instead_hook_can_delegate() {
    ensure_test_config();
    let manager = HookManager::new();
    let mut api = manager.scope("test_plugin");

    assert_eq!(test_add(0), 42, "Original function should return 42");

    let instead: InsteadHook = instead_hook!(|args: HookArgs| {
      // If the first argument is 5, Return 50 :3
      if args.get::<i8>(0) == Some(5) {
        Ok(InsteadHookResult::Return(HookReturn::new::<i8>(50)))
      } else {
        // Otherwise call original function
        Ok(InsteadHookResult::Delegate(args))
      }
    });

    api.register_instead_sync("test_add", instead).expect("Should register");

    assert_eq!(test_add(0), 42, "Delegation should return 42");
    assert_eq!(test_add(5), 50, "Hooked condition should return 50");
    api.cleanup().expect("Should cleanup hooks");
  }

  #[test]
  #[serial]
  fn instead_hook_functional() {
    ensure_test_config();
    let manager = HookManager::new();
    let mut api = manager.scope("test_plugin2");

    let instead: InsteadHook = instead_hook!(|_args: HookArgs| {
      println!("I like 45 more");
      Ok(InsteadHookResult::Return(HookReturn::new::<i8>(45)))
    });

    api.register_instead_sync("test_add", instead).expect("Should register");

    assert_eq!(test_add(5), 45, "Hooked function should return 45");

    api.cleanup().expect("Should cleanup hooks");
  }
}

// dont listen to the error, its lying
#[hookable]
fn test_add(add: i8) -> i8 {
  println!("Original test_add called");
  42 + add
}
