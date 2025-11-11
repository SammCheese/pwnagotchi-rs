use std::{any::Any, error::Error, sync::Arc};

use futures::future::BoxFuture;
use serde::{Deserialize, Serialize};

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

#[derive(Clone, Debug)]
pub struct TypeMetadata {
  pub type_name: String,
  pub serialized: Option<Vec<u8>>,
}

pub struct CapturedArg {
  pub value: Box<dyn Any + Send + Sync>,
  pub metadata: TypeMetadata,
}

impl CapturedArg {
  pub fn capture<T>(value: T) -> Self
  where
    T: Send + Sync + Serialize + 'static,
  {
    let type_name = std::any::type_name::<T>().to_string();
    let config = bincode::config::standard();
    let serialized = bincode::serde::encode_to_vec(&value, config).ok();

    Self {
      value: Box::new(value),
      metadata: TypeMetadata { type_name, serialized },
    }
  }

  pub fn raw<T>(value: T) -> Self
  where
    T: Send + Sync + 'static,
  {
    let type_name = std::any::type_name::<T>().to_string();

    Self {
      value: Box::new(value),
      metadata: TypeMetadata { type_name, serialized: None },
    }
  }
}

pub struct HookReturn {
  value: Box<dyn Any + Send + Sync>,
  metadata: TypeMetadata,
}

impl HookReturn {
  pub fn new<T>(value: T) -> Self
  where
    T: Send + Sync + Serialize + 'static,
  {
    Self::from_captured(CapturedArg::capture(value))
  }

  pub fn raw<T>(value: T) -> Self
  where
    T: Send + Sync + 'static,
  {
    Self::from_captured(CapturedArg::raw(value))
  }

  pub fn from_captured(arg: CapturedArg) -> Self {
    Self { value: arg.value, metadata: arg.metadata }
  }

  pub fn into_captured(self) -> CapturedArg {
    CapturedArg {
      value: self.value,
      metadata: self.metadata,
    }
  }

  pub fn metadata(&self) -> &TypeMetadata {
    &self.metadata
  }

  pub fn type_name(&self) -> &str {
    &self.metadata.type_name
  }

  fn get_deserialized<T>(&self) -> Option<T>
  where
    T: for<'de> Deserialize<'de>,
  {
    let serialized = self.metadata.serialized.as_ref()?;
    let config = bincode::config::standard();
    bincode::serde::decode_from_slice(serialized, config)
      .ok()
      .map(|(value, _size)| value)
  }

  pub fn get<T>(&self) -> Option<&T>
  where
    T: 'static,
  {
    self.value.downcast_ref::<T>()
  }

  pub fn get_smart<T>(&self) -> Option<T>
  where
    T: 'static + Clone + for<'de> Deserialize<'de>,
  {
    if let Some(value) = self.get::<T>() {
      return Some(value.clone());
    }
    self.get_deserialized::<T>()
  }

  pub fn get_mut<T>(&mut self) -> Option<&mut T>
  where
    T: 'static,
  {
    self.value.downcast_mut::<T>()
  }

  pub fn set<T>(&mut self, value: T)
  where
    T: Send + Sync + 'static,
  {
    self.value = Box::new(value);
    self.metadata.type_name = std::any::type_name::<T>().to_string();
    self.metadata.serialized = None;
  }

  pub fn set_serialized<T>(&mut self, value: T) -> Result<(), String>
  where
    T: Send + Sync + Serialize + 'static,
  {
    let config = bincode::config::standard();
    let serialized = bincode::serde::encode_to_vec(&value, config)
      .map_err(|e| format!("Failed to serialize: {}", e))?;
    self.value = Box::new(value);
    self.metadata.type_name = std::any::type_name::<T>().to_string();
    self.metadata.serialized = Some(serialized);
    Ok(())
  }

  pub fn take<T>(self) -> Option<T>
  where
    T: 'static + for<'de> Deserialize<'de>,
  {
    if let Ok(value) = self.value.downcast::<T>() {
      return Some(*value);
    }

    if let Some(serialized) = self.metadata.serialized {
      let config = bincode::config::standard();
      if let Ok((value, _)) = bincode::serde::decode_from_slice(&serialized, config) {
        return Some(value);
      }
    }

    None
  }

  pub fn unmut(&mut self) -> HookReturn {
    HookReturn {
      value: std::mem::replace(&mut self.value, Box::new(()) as Box<dyn Any + Send + Sync>),
      metadata: std::mem::replace(
        &mut self.metadata,
        TypeMetadata {
          type_name: std::any::type_name::<()>().to_string(),
          serialized: None,
        },
      ),
    }
  }
}

pub struct HookArgs {
  args: Vec<Box<dyn Any + Send + Sync>>,
  metadata: Vec<TypeMetadata>,
}

impl HookArgs {
  pub fn new(args: Vec<Box<dyn Any + Send + Sync>>) -> Self {
    let metadata = args
      .iter()
      .map(|arg| TypeMetadata {
        type_name: std::any::type_name_of_val(&**arg).to_string(),
        serialized: None,
      })
      .collect();
    Self { args, metadata }
  }

  pub fn from_captured(entries: Vec<CapturedArg>) -> Self {
    let mut args: Vec<Box<dyn Any + Send + Sync>> = Vec::with_capacity(entries.len());
    let mut metadata: Vec<TypeMetadata> = Vec::with_capacity(entries.len());

    for entry in entries {
      args.push(entry.value);
      metadata.push(entry.metadata);
    }

    Self { args, metadata }
  }

  pub fn new_serialized<T: Serialize>(values: Vec<T>) -> Result<Self, Box<dyn Error>> {
    let config = bincode::config::standard();
    let mut args: Vec<Box<dyn Any + Send + Sync>> = Vec::new();
    let mut metadata: Vec<TypeMetadata> = Vec::new();

    for value in values {
      let serialized = bincode::serde::encode_to_vec(&value, config)
        .map_err(|e| format!("Failed to serialize: {}", e))?;
      let type_name = std::any::type_name::<T>().to_string();

      args.push(Box::new(serialized.clone()));
      metadata.push(TypeMetadata { type_name, serialized: Some(serialized) });
    }

    Ok(Self { args, metadata })
  }

  pub fn get_by_name<T: 'static>(&self, index: usize, expected_name: &str) -> Option<&T> {
    if let Some(meta) = self.metadata.get(index)
      && meta.type_name.contains(expected_name)
    {
      return self.args.get(index)?.downcast_ref::<T>();
    }
    None
  }

  pub fn get_deserialized<T>(&self, index: usize) -> Option<T>
  where
    T: for<'de> Deserialize<'de>,
  {
    let meta = self.metadata.get(index)?;
    let serialized = meta.serialized.as_ref()?;

    let config = bincode::config::standard();
    bincode::serde::decode_from_slice(serialized, config)
      .ok()
      .map(|(value, _size)| value)
  }

  pub fn get<T>(&self, index: usize) -> Option<T>
  where
    T: 'static + Clone + for<'de> Deserialize<'de>,
  {
    // Try direct downcast first (works for same-binary types)
    if let Some(value) = self.get_by_downcast::<T>(index) {
      return Some(value.clone());
    }

    // Fallback to deserialization (works across FFI boundary)
    self.get_deserialized::<T>(index)
  }

  pub fn get_by_downcast<T: 'static>(&self, index: usize) -> Option<&T> {
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

    if let Some(meta) = self.metadata.get_mut(index) {
      meta.type_name = std::any::type_name::<T>().to_string();
      meta.serialized = None;
    }

    Ok(())
  }

  pub fn set_serialized<T: Serialize>(&mut self, index: usize, value: &T) -> Result<(), String> {
    if index >= self.args.len() {
      return Err(format!("Index {} out of bounds", index));
    }

    let config = bincode::config::standard();
    let serialized = bincode::serde::encode_to_vec(value, config)
      .map_err(|e| format!("Failed to serialize: {}", e))?;

    self.args[index] = Box::new(serialized.clone());

    if let Some(meta) = self.metadata.get_mut(index) {
      meta.type_name = std::any::type_name::<T>().to_string();
      meta.serialized = Some(serialized);
    }

    Ok(())
  }

  pub fn iter(&self) -> impl Iterator<Item = &Box<dyn Any + Send + Sync>> {
    self.args.iter()
  }

  pub fn unmut(&mut self) -> HookArgs {
    HookArgs {
      args: std::mem::take(&mut self.args),
      metadata: std::mem::take(&mut self.metadata),
    }
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
