#![allow(unused_imports)]

use pwnagotchi_core::*;
use pwnagotchi_shared::types::hooks::HookDescriptor;

fn normalize_type(raw: &str) -> String {
  raw.chars().filter(|c| !c.is_whitespace()).collect()
}

fn find_hook(name: &str) -> &'static HookDescriptor {
  inventory::iter::<HookDescriptor>()
    .find(|h| h.name == name)
    .unwrap_or_else(|| panic!("Missing hook descriptor for {name}"))
}

#[test]
fn hookables_not_empty() {
  let hooks = inventory::iter::<HookDescriptor>().collect::<Vec<_>>();

  assert!(!hooks.is_empty(), "No hookables registered");
}

#[test]
fn hookables_unique_names() {
  let hooks = inventory::iter::<HookDescriptor>().collect::<Vec<_>>();
  let mut names = std::collections::HashSet::new();
  for hook in hooks {
    assert!(names.insert(hook.name), "Duplicate hookable name found: {}", hook.name);
  }
}

#[test]
fn agent_hookables() {
  let agent_hooks = [
    "Agent::associate",
    "Agent::deauth",
    "Agent::set_channel",
    "Agent::set_mode",
    "Agent::recon",
    "Agent::get_access_points_by_channel",
    "Agent::start_pwnagotchi",
    "Agent::reboot",
    "Agent::restart",
    "Agent::should_interact",
    "Agent::get_access_points",
  ];

  let hooks = inventory::iter::<HookDescriptor>()
    .filter(|h| h.name.starts_with("Agent::"))
    .collect::<Vec<_>>();

  for expected_hook in agent_hooks {
    assert!(
      hooks.iter().any(|h| h.name == expected_hook),
      "Unexpected hook registered for Agent: {expected_hook}"
    );
  }
}

#[test]
fn agent_set_access_points_metadata() {
  let hook = find_hook("Agent::set_access_points");

  assert_eq!(hook.parameters.len(), 2, "Unexpected parameter count");
  assert_eq!(hook.parameters[0].name, "instance", "Receiver metadata missing");
  assert_eq!(normalize_type(hook.parameters[0].ty), "&Agent");
  assert_eq!(hook.parameters[1].name, "aps");
  assert_eq!(normalize_type(hook.parameters[1].ty), "&Vec<AccessPoint>");
  assert_eq!(normalize_type(hook.return_type), "()");
}

#[test]
fn agent_associate_metadata() {
  let hook = find_hook("Agent::associate");

  assert_eq!(hook.parameters.len(), 3);
  assert_eq!(hook.parameters[0].name, "instance");
  assert_eq!(normalize_type(hook.parameters[0].ty), "&Agent");
  assert_eq!(hook.parameters[1].name, "ap");
  assert_eq!(normalize_type(hook.parameters[1].ty), "&AccessPoint");
  assert_eq!(hook.parameters[2].name, "throttle");
  assert_eq!(normalize_type(hook.parameters[2].ty), "Option<f32>");
  assert_eq!(normalize_type(hook.return_type), "()");
}

#[test]
fn cli_do_custom_mode_metadata() {
  let hook = find_hook("Cli::do_custom_mode");

  assert_eq!(hook.parameters.len(), 2);
  assert_eq!(hook.parameters[0].name, "instance");
  assert_eq!(normalize_type(hook.parameters[0].ty), "&Cli");
  assert_eq!(hook.parameters[1].name, "_mode");
  assert_eq!(normalize_type(hook.parameters[1].ty), "&str");
  assert_eq!(normalize_type(hook.return_type), "()");
}

#[test]
fn hookable_methods_include_instance_metadata() {
  for hook in inventory::iter::<HookDescriptor>() {
    if hook.name.contains("::") {
      let Some(first) = hook.parameters.first() else {
        panic!("Method hook {} is missing parameters", hook.name);
      };

      assert_eq!(first.name, "instance", "Missing instance metadata for {}", hook.name);
      assert!(!first.ty.is_empty(), "Instance type missing for {}", hook.name);
    }
  }
}
