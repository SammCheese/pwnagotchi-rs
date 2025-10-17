use std::{any::Any, error::Error, sync::Arc};

use futures::future::BoxFuture;

pub enum BeforeHookResult {
  Continue(HookArgs),
  Stop,
}

pub enum AfterHookResult {
  Continue(HookReturn),
  Stop,
}

pub enum InsteadHookResult {
  Delegate(HookArgs),
  Return(HookReturn),
}

pub type BeforeHook = Arc<
  dyn Fn(&mut HookArgs) -> Result<BeforeHookResult, Box<dyn Error + Send + Sync>> + Send + Sync,
>;

pub type AfterHook = Arc<
  dyn Fn(&mut HookArgs, &mut HookReturn) -> Result<AfterHookResult, Box<dyn Error + Send + Sync>>
    + Send
    + Sync,
>;

pub type InsteadHook =
  Arc<dyn Fn(HookArgs) -> Result<InsteadHookResult, Box<dyn Error + Send + Sync>> + Send + Sync>;

pub type AsyncBeforeHook = Arc<
  dyn Fn(&mut HookArgs) -> BoxFuture<'static, Result<BeforeHookResult, Box<dyn Error + Send + Sync>>>
    + Send
    + Sync
    + 'static,
>;

pub type AsyncAfterHook = Arc<
  dyn Fn(
      &mut HookArgs,
      &mut HookReturn,
    ) -> BoxFuture<'static, Result<AfterHookResult, Box<dyn Error + Send + Sync>>>
    + Send
    + Sync
    + 'static,
>;

pub type AsyncInsteadHook = Arc<
  dyn Fn(HookArgs) -> BoxFuture<'static, Result<InsteadHookResult, Box<dyn Error + Send + Sync>>>
    + Send
    + Sync
    + 'static,
>;

pub type BoxFutureUnit = BoxFuture<'static, ()>;

pub struct HookReturn {
  pub value: Box<dyn Any + Send + Sync>,
}

impl HookReturn {
  pub fn new<T: Any + Send + Sync + 'static>(value: T) -> Self {
    Self { value: Box::new(value) }
  }

  pub fn get<T: 'static>(&self) -> Option<&T> {
    self.value.downcast_ref::<T>()
  }

  pub fn get_mut<T: 'static>(&mut self) -> Option<&mut T> {
    self.value.downcast_mut::<T>()
  }

  pub fn set<T: Any + Send + Sync + 'static>(&mut self, value: T) {
    self.value = Box::new(value);
  }

  pub fn take<T: Any + Send + Sync + 'static>(self) -> Option<T> {
    self.value.downcast::<T>().ok().map(|b| *b)
  }

  pub fn unmut(&mut self) -> HookReturn {
    HookReturn {
      value: std::mem::replace(&mut self.value, Box::new(()) as Box<dyn Any + Send + Sync>),
    }
  }
}

pub struct HookArgs {
  args: Vec<Box<dyn Any + Send + Sync>>,
}

impl HookArgs {
  pub fn new(args: Vec<Box<dyn Any + Send + Sync>>) -> Self {
    Self { args }
  }

  pub fn get<T: 'static>(&self, index: usize) -> Option<&T> {
    self.args.get(index)?.downcast_ref::<T>()
  }

  pub fn get_mut<T: 'static>(&mut self, index: usize) -> Option<&mut T> {
    self.args.get_mut(index)?.downcast_mut::<T>()
  }

  pub fn set<T: Any + Send + Sync + 'static>(
    &mut self,
    index: usize,
    value: T,
  ) -> Result<(), String> {
    if index >= self.args.len() {
      return Err(format!("Index {} out of bounds", index));
    }
    self.args[index] = Box::new(value);
    Ok(())
  }

  pub fn unmut(&mut self) -> HookArgs {
    HookArgs { args: std::mem::take(&mut self.args) }
  }

  pub fn len(&self) -> usize {
    self.args.len()
  }

  pub fn is_empty(&self) -> bool {
    self.args.is_empty()
  }

  pub fn take<T: Any + Send + Sync + Default + 'static>(&mut self, index: usize) -> Option<T> {
    let replacement = Box::new(T::default());
    let old = std::mem::replace(&mut self.args[index], replacement);
    old.downcast::<T>().ok().map(|b| *b)
  }
}

#[derive(Clone, Debug)]
pub struct HookParameter {
  pub name: &'static str,
  pub ty: &'static str,
}

pub struct HookDescriptor {
  pub name: &'static str,
  pub parameters: &'static [HookParameter],
  pub return_type: &'static str,
  pub register_before: fn(Box<dyn Any + Send + Sync>) -> Option<u64>,
  pub unregister_before: fn(u64) -> bool,
  pub register_after: fn(Box<dyn Any + Send + Sync>) -> Option<u64>,
  pub unregister_after: fn(u64) -> bool,
  pub register_instead: fn(Box<dyn Any + Send + Sync>) -> Option<u64>,
  pub unregister_instead: fn(u64) -> bool,
}

impl HookDescriptor {
  #[allow(clippy::too_many_arguments)]
  pub const fn new(
    name: &'static str,
    parameters: &'static [HookParameter],
    return_type: &'static str,
    register_before: fn(Box<dyn Any + Send + Sync>) -> Option<u64>,
    unregister_before: fn(u64) -> bool,
    register_after: fn(Box<dyn Any + Send + Sync>) -> Option<u64>,
    unregister_after: fn(u64) -> bool,
    register_instead: fn(Box<dyn Any + Send + Sync>) -> Option<u64>,
    unregister_instead: fn(u64) -> bool,
  ) -> Self {
    Self {
      name,
      parameters,
      return_type,
      register_before,
      unregister_before,
      register_after,
      unregister_after,
      register_instead,
      unregister_instead,
    }
  }
}

inventory::collect!(HookDescriptor);
