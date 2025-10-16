pub trait PluginMetadata {
  fn name(&self) -> &'static str;
  fn version(&self) -> &'static str;
  fn author(&self) -> &'static str;
  fn description(&self) -> &'static str;
}

pub type BoxedPlugin = Box<dyn PluginMetadata + Send + Sync>;
