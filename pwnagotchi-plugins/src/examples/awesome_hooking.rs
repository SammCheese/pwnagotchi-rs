use std::{error::Error, sync::Arc};

use pwnagotchi_shared::{
  models::{agent::RunningMode, net::AccessPoint},
  traits::general::CoreModules,
  types::hooks::{AfterHookResult, BeforeHookResult, HookArgs, HookReturn, InsteadHookResult},
};

use crate::{
  async_after_hook, async_before_hook, async_instead_hook,
  traits::{
    hooks::DynamicHookAPITrait,
    plugins::{Plugin, PluginInfo},
  },
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
    }
  }

  fn on_load(
    &mut self,
    hook_api: &mut dyn DynamicHookAPITrait,
    _core: Arc<CoreModules>,
  ) -> Result<(), Box<dyn Error + 'static>> {
    // Async Before Hook
    let before = async_before_hook!(|args: &mut HookArgs| {
      let owned_args = args.unmut();
      async move {
        println!("Agent is about to start Recon");
        Ok(BeforeHookResult::Continue(owned_args))
      }
    });
    hook_api.register_before_async("Agent::recon", before)?;

    // Async After Hook
    let after = async_after_hook!(|args: &mut HookArgs, ret: &mut HookReturn| {
      let owned_args = args.unmut();
      let aps: Vec<(u8, Vec<AccessPoint>)> =
        ret.get::<Vec<(u8, Vec<AccessPoint>)>>().unwrap().clone();
      let channel: u8 = *owned_args.get::<u8>(1).unwrap();

      async move {
        println!(
          "Agent finished recon on channel {} and found {} access points",
          channel,
          aps.iter().map(|(_, aps)| aps.len()).sum::<usize>()
        );
        Ok(AfterHookResult::Continue(HookReturn::new(aps)))
      }
    });
    hook_api.register_after_async("Agent::get_access_points_by_channel", after)?;

    // Async Instead Hook
    // Important note: The self parameter is NOT passed to instead hooks
    // So index 0 is the first actual argument
    let instead = async_instead_hook!(|args: HookArgs| {
      async move {
        let mode: &RunningMode = args.get(0).unwrap();
        println!("Actually, Lets not change the mode to {:?}", mode);
        Ok(InsteadHookResult::Return(HookReturn::new(())))
      }
    });
    hook_api.register_instead_async("Agent::set_mode", instead)?;
    Ok(())
  }

  fn on_unload(&mut self) -> Result<(), Box<dyn Error + 'static>> {
    println!("AwesomeHooking plugin shutting down.");
    Ok(())
  }
}
