pub mod api;
pub mod loaders;
pub mod traits;

pub mod examples {
  pub mod awesome_hooking;
  pub mod hello_world;
}

pub mod managers {
  pub mod hook_manager;
  pub mod plugin_manager;
}

#[cfg(test)]
pub mod tests {
  pub mod hook_manager_test;
  pub mod hooks_test;
}
