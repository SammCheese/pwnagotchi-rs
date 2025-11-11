# Pwnagotchi-rs


Do you like pwnagotchi? Do you hate Python?

Awesome! Me too!

-----

This is a Work-in-Progress rewrite of Pwnagotchi in Rust.


It is unfinished, poorly Structured, does not include plugin nor display support (yet) and is broken/unimplemented in many parts.
The WebUI is also fairly ugly and partly not working yet.

I actually have no actual experience in Rust. This is a learning project for me so dont judge me for the poor execution or suboptimal implementations! ;D


Dependencies, Code and any API available WILL Rapidly change, refactor and/or get removed. Be warned!


### Disclaimer

> As with the original Project, only use this where you are authorized to do so, check in with your regions laws, bla bla bla bla.
> I take no responsibility for you getting fined, arrested, or any other repercussions you might face.

### Plugin Development

There is a very basic devkit to make Plugins (only in rust for now) included.
You can find it in the `pwnagotchi-plugin-dev` folder.

You should make your plugins under `pwnagotchi-plugin-dev > src > plugins`. 

Simply create a new folder with your plugin name there. 

In that folder, create another folder called `src` and a file called `Cargo.toml`

The content of the `Cargo.toml` should always contain this base template. You can and should of course replace the actual package fields to your own detes.

```toml
# Cargo.toml
[package]
name = "PluginName"
version = "1.0.0"
edition = "2024"
description = "Your Plugin Description"
authors = ["Username <myemail@example.com>"]

[lib]
crate-type = ["cdylib"]

[dependencies]
pwnagotchi-plugins = { workspace = true }
pwnagotchi-shared = { workspace = true }
pwnagotchi-macros = { workspace = true }

# Any other dependencies you need here
``` 


After that, you can move on to the `src` folder.
Navigate into it and create a `lib.rs` inside it.

As of right now, this is the plugin template that you should generally follow in your `lib.rs`

```rs
// lib.rs
use std::{error::Error, sync::Arc};
use pwnagotchi_plugins::traits::{
  plugins::{Plugin, PluginAPI, PluginInfo},
};

#[derive(Default)]
pub struct MyPlugin;

// You should always have new() function for your Plugin....
impl MyPlugin {
  pub fn new() -> Self {
    Self {}
  }
}

// Implements the required functions for your plugin to work
impl Plugin for MyPlugin {
  // Used to show the details of your Plugin
  // You should change the details to your liking
  fn info(&self) -> &PluginInfo {
    &PluginInfo {
      name: "MyPlugin",
      version: "0.1.0",
      author: "YourName",
      description: "My Awesome Plugin",
      license: "MIT",
    }
  }

  // called whenever the plugin is loaded. Provides the PluginAPI
  fn on_load(&mut self, plugin_api: PluginAPI) -> Result<(), Box<dyn Error + 'static>> {
    Ok(())
  }

  // called whenever the plugin shuts down
  fn on_unload(&mut self) -> Result<(), Box<dyn Error + 'static>> {
    Ok(())
  }
}

// Constructor
#[allow(improper_ctypes_definitions)]
#[unsafe(no_mangle)]
pub extern "C" fn _plugin_create() -> *mut dyn Plugin {
  let plugin: Box<dyn Plugin> = Box::new(MyPlugin::new());
  Box::into_raw(plugin)
}


// Destructor 
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
``` 

#### Plugin Building

Once done, you can build your plugins by running the provided `build_plugins.sh` script.

If you want to additionally build the Example Plugins and/or the Default Plugins, you can do that by adding the `BUILD_EXAMPLES=1` and/or `BUILD_DEFAULTS=1` env var to the script like this:

```sh
BUILD_EXAMPLES=1 BUILD_DEFAULTS=1 ./build_plugins.sh release
```

The Built Plugins can then be found in the `dist/plugins` folder.