use std::{
  any::Any,
  error::Error,
  fmt::{Display, Formatter, Result as FmtResult},
};

use pwnagotchi_shared::types::hooks::{
  AfterHook, AsyncAfterHook, AsyncBeforeHook, AsyncInsteadHook, BeforeHook, HookDescriptor,
  InsteadHook,
};

pub trait DynamicHookAPITrait {
  fn register_before(
    &mut self,
    hook_name: &str,
    callback: Box<dyn Any + Send + Sync>,
  ) -> Result<u64, HookError>;

  fn register_after(
    &mut self,
    hook_name: &str,
    callback: Box<dyn Any + Send + Sync>,
  ) -> Result<u64, HookError>;

  fn register_instead(
    &mut self,
    hook_name: &str,
    callback: Box<dyn Any + Send + Sync>,
  ) -> Result<u64, HookError>;

  fn list_hooks(&self) -> Vec<&'static HookDescriptor>;

  fn cleanup(&mut self) -> Result<(), HookError>;

  // Convenience methods with concrete types (no turbofish needed!)
  fn register_before_sync(
    &mut self,
    hook_name: &str,
    callback: BeforeHook,
  ) -> Result<u64, HookError> {
    self.register_before(hook_name, Box::new(callback))
  }

  fn register_after_sync(
    &mut self,
    hook_name: &str,
    callback: AfterHook,
  ) -> Result<u64, HookError> {
    self.register_after(hook_name, Box::new(callback))
  }

  fn register_instead_sync(
    &mut self,
    hook_name: &str,
    callback: InsteadHook,
  ) -> Result<u64, HookError> {
    self.register_instead(hook_name, Box::new(callback))
  }

  fn register_before_async(
    &mut self,
    hook_name: &str,
    callback: AsyncBeforeHook,
  ) -> Result<u64, HookError> {
    self.register_before(hook_name, Box::new(callback))
  }

  fn register_after_async(
    &mut self,
    hook_name: &str,
    callback: AsyncAfterHook,
  ) -> Result<u64, HookError> {
    self.register_after(hook_name, Box::new(callback))
  }

  fn register_instead_async(
    &mut self,
    hook_name: &str,
    callback: AsyncInsteadHook,
  ) -> Result<u64, HookError> {
    self.register_instead(hook_name, Box::new(callback))
  }

  fn unregister(&mut self, hook_id: &u64) -> Result<(), HookError>;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HookError {
  UnknownHook(String),
  TypeMismatch(String),
  UnregisterFailed(String),
}

impl Display for HookError {
  fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
    match self {
      HookError::UnknownHook(name) => write!(f, "Unknown hook '{name}'"),
      HookError::TypeMismatch(name) => {
        write!(f, "Callback type mismatch when registering hook '{name}'")
      }
      HookError::UnregisterFailed(name) => {
        write!(f, "Failed to unregister hook '{name}'")
      }
    }
  }
}

impl Error for HookError {}
