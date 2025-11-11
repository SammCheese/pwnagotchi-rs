use std::{any::Any, collections::HashMap, sync::Mutex};

use pwnagotchi_shared::{logger::LOGGER, types::hooks::HookDescriptor};

use crate::traits::hooks::{DynamicHookAPITrait, HookError};

pub struct HookManager {
  pub(crate) registrations: Mutex<HashMap<String, Vec<StoredHook>>>,
}

impl HookManager {
  pub fn new() -> Self {
    Self {
      registrations: Mutex::new(HashMap::new()),
    }
  }

  pub fn scope<'a>(&'a self, plugin: &'a str) -> DynamicHookAPI<'a> {
    DynamicHookAPI {
      manager: self,
      plugin_name: plugin.to_owned(),
      registered_hooks: Vec::new(),
    }
  }

  pub fn available_hooks(&self) -> Vec<&'static HookDescriptor> {
    inventory::iter::<HookDescriptor>().collect()
  }

  pub fn unregister_plugin(&self, plugin: &str) -> Result<(), HookError> {
    let handles = {
      let mut guard = self.registrations.lock().unwrap();
      guard.remove(plugin)
    };

    if let Some(handles) = handles {
      for handle in handles {
        self.unregister_hook_internal(&handle)?;
        LOGGER.log_debug(
          "HOOKS",
          &format!(
            "Unregistered hook '{}' with ID {} for plugin '{}'",
            handle.hook, handle.id, plugin
          ),
        );
      }
    }

    Ok(())
  }

  fn find_descriptor(&self, name: &str) -> Option<&'static HookDescriptor> {
    inventory::iter::<HookDescriptor>().find(|descriptor| descriptor.name == name)
  }

  fn register_internal(
    &self,
    name: &str,
    kind: HookKind,
    callback: Box<dyn Any + Send + Sync>,
  ) -> Result<StoredHook, HookError> {
    let descriptor = self
      .find_descriptor(name)
      .ok_or_else(|| HookError::UnknownHook(name.to_string()))?;

    let id = match kind {
      HookKind::Before => (descriptor.register_before)(callback),
      HookKind::After => (descriptor.register_after)(callback),
      HookKind::Instead => (descriptor.register_instead)(callback),
    }
    .ok_or_else(|| HookError::TypeMismatch(descriptor.name.to_string()))?;

    let message = format!("Registering hook: {} (ID {})", descriptor.name, id);
    LOGGER.log_debug("HOOKS", &message);
    eprintln!("{}", message);

    Ok(StoredHook::new(descriptor.name, kind, id))
  }

  pub fn unregister_hook(&self, plugin: &str, hook_name: &str, id: u64) -> Result<(), HookError> {
    let handle = {
      let mut guard = self.registrations.lock().unwrap();
      if let Some(hooks) = guard.get_mut(plugin) {
        if let Some(idx) = hooks.iter().position(|h| h.hook == hook_name && h.id == id) {
          Some(hooks.remove(idx))
        } else {
          return Err(HookError::UnregisterFailed(format!(
            "Hook '{}' with ID {} not found in plugin '{}'",
            hook_name, id, plugin
          )));
        }
      } else {
        return Err(HookError::UnknownHook(format!("No hooks registered for plugin '{}'", plugin)));
      }
    };

    if let Some(handle) = handle {
      self.unregister_hook_internal(&handle)?;
    }

    Ok(())
  }

  fn unregister_hook_internal(&self, handle: &StoredHook) -> Result<(), HookError> {
    let descriptor = self
      .find_descriptor(handle.hook)
      .ok_or_else(|| HookError::UnknownHook(handle.hook.to_string()))?;

    let success = match handle.kind {
      HookKind::Before => (descriptor.unregister_before)(handle.id),
      HookKind::After => (descriptor.unregister_after)(handle.id),
      HookKind::Instead => (descriptor.unregister_instead)(handle.id),
    };

    let message = format!("Unregistering hook on: {} (ID {})", handle.hook, handle.id);
    LOGGER.log_debug("HOOKS", &message);
    eprintln!("{}", message);

    drop(handle.to_owned());

    if success { Ok(()) } else { Err(HookError::UnregisterFailed(handle.hook.to_string())) }
  }

  fn track_registration(&self, plugin: &str, handle: StoredHook) {
    let mut guard = self.registrations.lock().unwrap();
    guard.entry(plugin.to_owned()).or_default().push(handle);
  }
}

impl Default for HookManager {
  fn default() -> Self {
    Self::new()
  }
}

pub struct DynamicHookAPI<'a> {
  pub(crate) manager: &'a HookManager,
  pub(crate) plugin_name: String,
  pub(crate) registered_hooks: Vec<(String, u64)>,
}

impl DynamicHookAPITrait for DynamicHookAPI<'_> {
  fn registered(&self) -> &[(String, u64)] {
    &self.registered_hooks
  }

  fn register_before(
    &mut self,
    hook_name: &str,
    callback: Box<dyn Any + Send + Sync>,
  ) -> Result<u64, HookError> {
    let handle = self.manager.register_internal(hook_name, HookKind::Before, callback)?;
    let id = handle.id;
    self.manager.track_registration(&self.plugin_name, handle);
    self.registered_hooks.push((hook_name.to_string(), id));
    Ok(id)
  }

  fn register_after(
    &mut self,
    hook_name: &str,
    callback: Box<dyn Any + Send + Sync>,
  ) -> Result<u64, HookError> {
    let handle = self.manager.register_internal(hook_name, HookKind::After, callback)?;
    let id = handle.id;
    self.manager.track_registration(&self.plugin_name, handle);
    self.registered_hooks.push((hook_name.to_string(), id));
    Ok(id)
  }

  fn register_instead(
    &mut self,
    hook_name: &str,
    callback: Box<dyn Any + Send + Sync>,
  ) -> Result<u64, HookError> {
    let handle = self.manager.register_internal(hook_name, HookKind::Instead, callback)?;
    let id = handle.id;
    self.manager.track_registration(&self.plugin_name, handle);
    self.registered_hooks.push((hook_name.to_string(), id));
    Ok(id)
  }

  fn list_hooks(&self) -> Vec<&'static HookDescriptor> {
    self.manager.available_hooks()
  }

  fn cleanup(&mut self) -> Result<(), HookError> {
    self.manager.unregister_plugin(&self.plugin_name)?;
    self.registered_hooks.clear();
    Ok(())
  }

  fn unregister(&mut self, hook_id: &u64) -> Result<(), HookError> {
    let pos = self.registered_hooks.iter().position(|(_, id)| id == hook_id);
    if let Some(idx) = pos {
      let (hook_name, id) = self.registered_hooks.remove(idx);
      self.manager.unregister_hook(&self.plugin_name, &hook_name, id)
    } else {
      Err(HookError::UnregisterFailed(format!(
        "Hook with ID {} not found in plugin '{}'",
        hook_id, self.plugin_name
      )))
    }
  }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StoredHook {
  hook: &'static str,
  kind: HookKind,
  id: u64,
}

impl StoredHook {
  const fn new(hook: &'static str, kind: HookKind, id: u64) -> Self {
    Self { hook, kind, id }
  }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum HookKind {
  Before,
  After,
  Instead,
}
