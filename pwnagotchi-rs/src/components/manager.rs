use std::{
  collections::{HashMap, VecDeque},
  sync::Arc,
};

use anyhow::{Result, anyhow};
use pwnagotchi_shared::{
  config::config,
  logger::LOGGER,
  traits::general::{Component, CoreModule, CoreModules, Dependencies},
};
use tokio::task::JoinHandle;

type BoxedComponent = Box<dyn Component + Send + Sync>;
type BoxedDependencies<'a> = Box<dyn Dependencies + Send + Sync + 'a>;
type BoxedCoreModule = Box<dyn CoreModule + Send + Sync>;

fn core_modules_as_dependencies(c: &'_ Vec<BoxedCoreModule>) -> Vec<BoxedDependencies<'_>> {
  c.iter()
    .map(|comp| Box::new(CoreModuleAsDeps::new(comp.as_ref())) as BoxedDependencies)
    .collect()
}

fn components_as_dependencies(c: &'_ Vec<BoxedComponent>) -> Vec<BoxedDependencies<'_>> {
  c.iter()
    .map(|comp| Box::new(ComponentAsDeps::new(comp.as_ref())) as BoxedDependencies)
    .collect()
}

pub struct ComponentAsDeps<'a> {
  inner: &'a (dyn Component + Send + Sync),
}

impl<'a> ComponentAsDeps<'a> {
  fn new(inner: &'a (dyn Component + Send + Sync)) -> Self {
    Self { inner }
  }
}

impl Dependencies for ComponentAsDeps<'_> {
  fn name(&self) -> &'static str {
    self.inner.name()
  }

  fn dependencies(&self) -> &[&str] {
    self.inner.dependencies()
  }
}

pub struct CoreModuleAsDeps<'a> {
  inner: &'a (dyn CoreModule + Send + Sync),
}

impl<'a> CoreModuleAsDeps<'a> {
  fn new(inner: &'a (dyn CoreModule + Send + Sync)) -> Self {
    Self { inner }
  }
}

impl Dependencies for CoreModuleAsDeps<'_> {
  fn name(&self) -> &'static str {
    self.inner.name()
  }

  fn dependencies(&self) -> &[&str] {
    self.inner.dependencies()
  }
}

pub struct ComponentManager {
  components: Vec<BoxedComponent>,
  ctx: Option<Arc<CoreModules>>,
  join_handles: Vec<(String, JoinHandle<()>)>,
}

impl Default for ComponentManager {
  fn default() -> Self {
    Self::new()
  }
}

impl ComponentManager {
  pub fn new() -> Self {
    Self {
      components: Vec::new(),
      ctx: None,
      join_handles: Vec::new(),
    }
  }

  pub fn set_core_modules(&mut self, ctx: Arc<CoreModules>) {
    self.ctx = Some(ctx);
  }

  pub fn register(&mut self, component: Box<dyn Component + Send + Sync>) {
    self.components.push(component);
  }

  /// # Panics
  /// This function will panic if `self.ctx` is `None` when calling `unwrap()`.
  pub async fn init_all(&mut self) -> Result<()> {
    if self.ctx.is_none() {
      return Err(anyhow!("CoreModules not set in ComponentManager"));
    }

    let order =
      sort_by_deps(&components_as_dependencies(&self.components), self.ctx.as_ref().unwrap())?;
    for idx in order {
      let comp = &mut self.components[idx];
      LOGGER.log_debug("Pwnagotchi", &format!("Initializing component {}", comp.name()));

      match comp.init(self.ctx.as_ref().unwrap()).await {
        Ok(()) => {}
        Err(e) => {
          let msg = format!("Failed to initialize component {}: {}", comp.name(), e);
          LOGGER.log_error("Pwnagotchi", &msg);
          return Err(anyhow!(msg));
        }
      }
    }

    LOGGER.log_info(
      "Pwnagotchi",
      &format!(
        "Pwnagotchi {}@{} (v{}) starting...",
        config().main.name,
        &self.ctx.as_ref().unwrap().identity.read().fingerprint(),
        env!("CARGO_PKG_VERSION")
      ),
    );

    Ok(())
  }

  /// # Panics
  /// This function will panic if `self.ctx` is `None` when calling `unwrap()`.
  pub async fn start_all(&mut self) -> Result<()> {
    if self.ctx.is_none() {
      return Err(anyhow!("CoreModules not set in ComponentManager"));
    }
    let order =
      sort_by_deps(&components_as_dependencies(&self.components), self.ctx.as_ref().unwrap())?;
    for idx in order {
      let comp = &self.components[idx];
      LOGGER.log_debug("Pwnagotchi", &format!("Starting component {}", comp.name()));

      match comp.start().await {
        Ok(Some(handle)) => {
          self.join_handles.push((comp.name().to_string(), handle));
        }
        Ok(None) => {}
        Err(e) => {
          let msg = format!("Failed to start component {}: {}", comp.name(), e);
          LOGGER.log_error("Pwnagotchi", &msg);
          return Err(anyhow!(msg));
        }
      }
    }
    Ok(())
  }

  /// # Panics
  /// This function will panic if `self.ctx` is `None` when calling `unwrap()`.
  pub async fn shutdown(&mut self) {
    LOGGER.log_debug("Pwnagotchi", "Shutting down components...");

    if let Ok(mut order) =
      sort_by_deps(&components_as_dependencies(&self.components), self.ctx.as_ref().unwrap())
    {
      order.reverse();
      for idx in order {
        let comp = &self.components[idx];
        let _ = comp.stop().await;
      }
    }

    // cancel/join handles
    for (name, handle) in self.join_handles.drain(..) {
      LOGGER.log_debug("Pwnagotchi", &format!("Stopping background task for component {name}"));
      handle.abort();
      let _ = handle.await;
    }
  }
}

fn sort_by_deps(arr: &Vec<BoxedDependencies<'_>>, ctx: &Arc<CoreModules>) -> Result<Vec<usize>> {
  let n = arr.len();
  let mut name_to_idx = HashMap::new();

  for (i, comp) in arr.iter().enumerate() {
    name_to_idx.insert(comp.name(), i);
  }

  let mut indeg = vec![0usize; n];
  let mut adj = vec![Vec::new(); n];

  let core_mod_names: Vec<&str> = vec![
    ctx.session_manager.name(),
    ctx.identity.read().name(),
    ctx.epoch.read().name(),
    ctx.bettercap.name(),
    ctx.view.name(),
    ctx.agent.name(),
    ctx.automata.name(),
  ];

  for (i, comp) in arr.iter().enumerate() {
    for &dep in comp.dependencies() {
      if core_mod_names.contains(&dep) {
        continue;
      }

      if let Some(&dep_idx) = name_to_idx.get(dep) {
        adj[dep_idx].push(i);
        indeg[i] += 1;
      } else {
        LOGGER.log_error(
          "Pwnagotchi",
          &format!("component '{}' depends on unknown component '{}'", comp.name(), dep),
        );
        return Err(anyhow!("component '{}' depends on unknown component '{}'", comp.name(), dep));
      }
    }
  }

  let mut q = VecDeque::new();
  for (i, &d) in indeg.iter().enumerate() {
    if d == 0 {
      q.push_back(i);
    }
  }

  let mut order = Vec::new();
  while let Some(u) = q.pop_front() {
    order.push(u);
    for &v in &adj[u] {
      indeg[v] -= 1;
      if indeg[v] == 0 {
        q.push_back(v);
      }
    }
  }

  if order.len() != n {
    LOGGER.log_error("Pwnagotchi", "dependency cycle detected in components");
    return Err(anyhow!("dependency cycle detected in components"));
  }
  Ok(order)
}
