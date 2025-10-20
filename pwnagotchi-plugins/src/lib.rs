pub mod api;
pub mod loaders;
pub mod traits;

pub mod managers {
  pub mod hook_manager;
  pub mod plugin_manager;
}

#[cfg(test)]
pub mod tests {
  pub mod hook_manager_test;
  pub mod hooks_test;
}
