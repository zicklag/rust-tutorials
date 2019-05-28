#[macro_use]
extern crate dlopen_derive;
use dlopen::wrapper::{Container, WrapperApi};

// The trait that must be implemented by plugins to allow them to handle
// commands.
pub trait Plugin {
    fn handle_command(&self, command: &str);
}

#[derive(WrapperApi)]
struct PluginApi {
    // The plugin library must implement this function and return a raw pointer
    // to a Plugin struct.
    get_plugin: extern fn() -> *mut Plugin,
}

pub fn run() {
    println!("Starting App");

    // Load the plugin by name from the plugins directory
    let plugin_api_wrapper: Container<PluginApi> = unsafe { Container::load("plugins/libplugin1.so") }.unwrap();
    let plugin = unsafe { Box::from_raw(plugin_api_wrapper.get_plugin()) };

    loop {
        // Prompt
        println!("Enter command:");

        // Read input
        let mut message = String::new();
        std::io::stdin().read_line(&mut message).unwrap();

        // Trim newline
        message = message.trim().into();

        // Give the plugin a chance to handle the command
        plugin.handle_command(&message);

        // Check command
        if message == "exit" {
            break
        }
    }
}

