extern crate app;

use app::Plugin;

// Our plugin implementation
struct Plugin1;

impl Plugin for Plugin1 {
    fn handle_command(&self, command: &str) {
        // Handle the `plugin1` command
        if command == "plugin1" {
            println!("Hey you triggered my 'plugin1' command!");

        // Handle an `echo` command
        } else if command.starts_with("echo ") {
            println!("Echo-ing what you said: {}", command);
        }
    }
}

#[no_mangle]
pub fn get_plugin() -> *mut Plugin {
    println!("Running plugin1");

    // Return a raw pointer to an instance of our plugin
    Box::into_raw(Box::new(Plugin1 {}))
}
