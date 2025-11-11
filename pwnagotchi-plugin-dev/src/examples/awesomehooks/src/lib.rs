use std::error::Error;

use pwnagotchi_plugins::{
  async_after_hook, async_before_hook, async_instead_hook,
  traits::plugins::{Plugin, PluginAPI, PluginInfo},
};
use pwnagotchi_shared::{
  models::{agent::RunningMode, net::AccessPoint},
  types::hooks::{AfterHookResult, BeforeHookResult, HookArgs, HookReturn, InsteadHookResult},
};

#[derive(Default)]
pub struct AwesomeHooking;

impl AwesomeHooking {
  pub fn new() -> Self {
    Self {}
  }
}

impl Plugin for AwesomeHooking {
  fn info(&self) -> &PluginInfo {
    &PluginInfo {
      name: "AwesomeHooking",
      version: "0.1.0",
      author: "Sammy",
      description: "A simple example plugin for hooking functions",
      license: "MIT",
    }
  }

  fn on_load(&mut self, plugin_api: PluginAPI) -> Result<(), Box<dyn Error + 'static>> {
    // Async Before Hook
    let before = async_before_hook!(|args: &mut HookArgs| {
      let owned_args = args.unmut();
      async move {
        println!("Agent is about to start Recon");
        Ok(BeforeHookResult::Continue(owned_args))
      }
    });
    plugin_api.hook_api.register_before_async("Agent::recon", before)?;

    // Async After Hook
    let after = async_after_hook!(|_args: &mut HookArgs, ret: &mut HookReturn| {
      let owned_ret = std::mem::replace(ret, HookReturn::new(()));
      let aps = owned_ret.take::<Vec<(u8, Vec<AccessPoint>)>>().unwrap_or_else(|| {
        eprintln!("Failed to deserialize get_access_points_by_channel return value");
        Vec::new()
      });
      async move {
        let total_aps: usize = aps.iter().map(|(_, aps)| aps.len()).sum();
        let channels: Vec<u8> = aps.iter().map(|(ch, _)| *ch).collect();

        println!(
          "Agent finished get_access_points_by_channel and found {} access points across {} channels: {:?}",
          total_aps,
          channels.len(),
          channels
        );
        Ok(AfterHookResult::Continue(HookReturn::new(aps)))
      }
    });
    plugin_api
      .hook_api
      .register_after_async("Agent::get_access_points_by_channel", after)?;

    // Async Instead Hook
    // Important note: The self parameter is NOT passed to instead hooks
    // So index 0 is the first actual argument
    let instead = async_instead_hook!(|args: HookArgs| {
      async move {
        if let Some(mode) = args.get::<RunningMode>(0) {
          println!("I would prevent mode {:?} from being set.... but i wont!", mode);
        } else {
          eprintln!("Failed to get RunningMode argument!");
        }
        Ok(InsteadHookResult::Delegate(args))
      }
    });
    plugin_api.hook_api.register_instead_async("Agent::set_mode", instead)?;

    Ok(())
  }

  fn on_unload(&mut self) -> Result<(), Box<dyn Error + 'static>> {
    Ok(())
  }
}

#[allow(improper_ctypes_definitions)]
#[unsafe(no_mangle)]
pub extern "C" fn _plugin_create() -> *mut dyn Plugin {
  let plugin: Box<dyn Plugin> = Box::new(AwesomeHooking::new());
  Box::into_raw(plugin)
}

#[allow(improper_ctypes_definitions)]
#[allow(clippy::missing_safety_doc)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn _plugin_destroy(ptr: *mut dyn Plugin) {
  if !ptr.is_null() {
    unsafe {
      drop(Box::from_raw(ptr));
    }
  }
}
