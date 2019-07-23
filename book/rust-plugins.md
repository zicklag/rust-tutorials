# Rust Plugins

This is a guide for setting your Rust application up with Rust plugins that can be loaded dynamically at runtime. Additionally, this plugin setup allows plugins to make calls to the application's public API so that it can make use of the same data structures and utilities for extending the application.

> **Warning:** After further testing, I cannot confirm that this plugin setup will work for all applications that have other crates as dependencies. It seems to work fine with the steps outlined in this tutorial, but I was not able to get it to work with a large project like [Amethyst](https://github.com/amethyst/amethyst).
>
> Additionally, this will only allow you to create plugins using the same version of Rust that the application was built with. Unfortunately, these issues defeated my use-case, but the tutorial may still be useful for one reason or another so I leave it here for reference.
>
> If you are wanting to attempt something similar, I recommend looking at the [ABI Stable Crates](https://github.com/rodrimati1992/abi_stable_crates) project.

This is a quick, somewhat unpolished tutorial that I figured I would make as I explored the subject for the first time myself.

The specific purpose for the plugins, in my case, is to allow for a Rust game to be extended/modded by writing Rust. I want the plugins to have full access to the game's API so that plugins are just about as powerful as making changes to the game core, without you having to manually create bindings to get access to engine internals.

The full source code for the final version of this tutorial can be found [here](https://github.com/zicklag/rust-plugin-tutorial/tree/master/rust-plugins).

> **Note:** This guide assumes that you are on Linux, and has not been tested on Windows, or Mac, but, conceptually, everything should work the same except for the extensions for shared libraries being different on the different platforms ( .so on Linux, .dll on Windows, .dylib on Mac ).

## Create The App

The first thing we need is a place to put our crates, like a `rust-plugins` folder. It is here that we will put our app crate and our plugin crate.

Now lets create the app directory inside the one we created for our project:

```bash
cargo new --bin app
```

Now move to the new `app` directory edit the `src/main.rs` file to look like so:

```rust
use app;

fn main() {
    app::run();
}
```

This will simply execute our app's run method. We want to keep the main function very simple. All of the application functionality will be put into the app's library. In that light, we need to create our crate's `src/lib.rs` file:

```rust
pub fn run() {
    println!("Starting App");
}
```

If you run `cargo run` now to run your app, you should get "Starting App" printed to the terminal. Now lets spend a little bit of time to understand what has happened. You've probably done this before, but in order to understand how plugins will work, we have to understand more about how different libraries and portions of our app end up a runnable program.

If we look in our `target/debug` directory, we can see the artifacts that were built when we ran `cargo run`.

```bash
> ls target/debug/
app  app.d  build/  deps/  examples/  incremental/  libapp.d  libapp.rlib  native/
```

In there we can see our program, `app`, which can be run manually:

```bash
> ./target/debug/app
Starting App
```

Rust has packed everything that your app needs to run inside of that one executable. If you copy that binary to any other system, it will run without needing any other libraries. Also the size of the binary is 1.7M.

The way that rust builds applications by default is great for most situations, and it lets you easily distribute your app just by providing a single binary, but for our use, we want to allow dynamically loading portions of the app that may not have come with it, and this requires some changes.

By default Rust will link all application dependencies **statically** into the the final executable. In this case, our app only depends on the standard library, which it uses to print to standard out. The problem with static linking is that only the app that is link to a static library can actually use the library. This means that if we have plugins, our plugins will not be able to call any of the functions in our application's library. For this tutorial we *do* want our plugins to be able to call our application's API to make use of utilities and functionality provided in our app. This requires **dynamic linking**.

To make rust create a dynamic library for our app that our plugins can link to, we first need to tell Cargo to compile the app as a dynamic library by adding this to the `Cargo.toml` file:

```toml
[lib]
crate-type = ["dylib", "rlib"]
```

In the above config we tell cargo to compile a `dylib` or dynamic library *and* an `rlib` or rust library. The `dylib` will be a `.so` shared library that will contain the machine code for our app's library and is needed when running the app and plugins. The `rlib` will make a `.rlib` file that provides rust with extra metadata that allows it to link plugins to the app's library at compile time. Without the `rlib` build of the library, our plugins would not be able to tell which functions are defined in the library without us providing the entire source-code for the app. The `rlib` is almost like a kind of header file that gives rust the info it needs to link to the crate without needing source code ( I think ).

> **Note:** There is another crate type called `cdylib` that can be used instead of `dylib`, but it behaves somewhat differently. It may be a better solution as it is not dependent on the Rust compiler version being exactly the same for the app and the plugins. I am trying to understand the full differences and have opened up a [forum topic](https://users.rust-lang.org/t/what-is-the-difference-between-dylib-and-cdylib/28847?u=zicklag) on the Rust user forum to discuss it. My current understanding can be found in [Appendix A](./appendix-a.md).

Additionally we need to tell cargo to add some flags to the its rust compiler calls. These settings go in a `.cargo/config` file:

```toml
[build]
rustflags = ["-C", "prefer-dynamic", "-C", "rpath"]
```

`prefer-dynamic` tells the compiler to prefer dynamic linking when compiling the app. This means that instead of statically linking the standard library into the our app, it will leave the standard library as a separate dynamically linked shared library ( `.so` ) file. This means that both our app and our plugins will be able to link to the same standard library, without duplicating the standard library for each plugin.

`rpath` tells the compiler to make the binary for our app look in the directory that it is in for shared libraries. This means that we can put the shared libraries that our app needs, such as the rust standard library, in the same director as the app binary and not require that the user add the libraries to the system PATH or the LD_LIBRARY_PATH.

If we run `cargo run` now, our app should still run the same, but things are a bit different under the hood.

For one, if we look in the `target/debug` directory now, we should see a `libapp.so` file in it which is about 14 kilobytes. Also, instead of our `app` binary being almost 2 megabytes, it is only 19 kilobytes. So, what happened? Well, instead of bundling everything up into our one binary, Rust has now compiled each library to its own dynamic library file ( the `.so` file, or `.dll` on Windows ) and dynamically linked our `app` binary to those libraries.

Dynamic linking means that, when you run the program, it will look around on your system for the libraries that it needs, because the libraries are not built into it. The places that the system will look for the dynamic libraries depends on the system. On Linux it will look in places like `/usr/lib` and also in any places indicated by the `LD_LIBRARY_PATH`. On Windows it will look in your `PATH`.

If you try to run the app manually, now you will actually get an error:

```bash
> ./target/debug/app
./target/debug/app: error while loading shared libraries: libstd-8e7d7d74c91e7cfe.so: cannot open shared object file: No such file or directory
```

This is because we have not put the Rust standard library somewhere that our app can find it! Because we added the `rpath` rust flag in our cargo config earlier, our app will look in the directory that it is in for dynamic libraries, as well as in the system search paths. The rust `libstd-*.so` file isn't in a system directory or in the executable's directory, so it throws an error saying that it cannot be found. All we have to do is copy that library to the our `target/debug` folder to get the app to run. If you are using rustup, you can find the libstd library in your rustup dir ( I'm using nightly rust, but make sure you choose whatever toolchain you compiled the app with ):

```bash
> cp ~/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/lib/libstd-8e7d7d74c91e7cfe.so target/debug/
> ./target/debug/app
Starting App
```

Now that our app can find the libraries it needs, it runs successfully! Also, the `libstd` file explains where the rest of the file size went when we switched to dynamic linking. The `libstd` library is 5M, which is larger than our executable was when it was statically linked, but that is probably because, when statically linking, rust can remove portions of the library that is not used, but when dynamically linking, you never know what portions of the library an app might use, so you have to make sure that it is all there all of the time.

Dynamic linking can be less convenient for distribution because you need more files with your app but it allows multiple applications to share the same libraries, which can save on disc space if there are many binaries or plugins that are using the same library. This helps us for our plugins use-case because all of the plugins will share the same app library and standard library.

### Create an App Library Function

Before we move on to creating our plugin, lets create a function that our plugin can call and put it in our app library:

**src/lib.rs:**

```rust
pub fn test_app_func(message: &str) {
    println!("test_app_func(\"{}\")", message);
}
```

Testing that our plugin can call this function in our app's library will prove that our plugin can, in fact, make use of our app's public rust API.

## Create a Plugin

The next thing we are going to do is create our plugin crate. Go ahead and cd back to your project folder and create the plugin crate alongside the app crate and the move to the plugin dir.

```bash
cargo new --lib plugin1
cd plugin1
```

For this crate we are going to make similar `Cargo.toml` and `.config/cargo` changes that we make for our app to make it dynamically link all of its dependencies. The only difference in this case is that we don't need need to set the `crate-type` to include `rlib` in the `Cargo.toml` file. Instead we set it to `dylib` only:

**Cargo.toml:**

```toml
[lib]
crate-type = ["dylib"]
```

**.cargo/config:**

```toml
[build]
rustflags = ["-C", "prefer-dynamic", "-C", "rpath"]
```

The reason the `rlib` build is not needed for plugins is because we don't plan on linking any other rust libraries to the plugin crate. The rlib build is only used when linking other rust libraries/binaries to this one. Granted, if you wanted to let your plugin have plugins, you would still want to build the `rlib`, but we're not going to take this that far here.

After that, we will add a `run()` function that will be called by our app to execute the plugin's functionality. Eventually plugins will be able to do more than just `run` but for now that is all we will do with it.

**src/lib.rs:**

```rust
extern crate app;

#[no_mangle]
pub fn run() {
    println!("Running plugin1");
    app::test_app_func("Hello from plugin 1");
}
```

Notice that we specify `app` as an external crate; if we had added `app` as a Cargo dependency, we could have done `use app;` instead. Our run function is simple and just prints some output before calling the `test_app_func` that we created in our app library. The `#[no_mangle]` attribute on the `run()` function tells the compiler not to add any extra metadata to that symbol in the compiled output, this allows us to call the function by name when we later load it into our app dynamically.

Attempting to `cargo build` the crate right now will tell us that it can't find the `app` crate. This is because we didn't add it as a dependency to our `Cargo.toml` file. Now, if we added the `app` crate to the plugin's dependencies, it would be able to compile, but it would also re-compile the app library, when we already have the app compiled. There is no reason to compile the app library twice, especially if it is a big app, so, instead, lets add the app library to our plugin's search path so that it will find our already built `app` crate.

To tell cargo how to find our app crate, we create a `build.rs` script. The `build.rs` script can be used to do any kind of setup necessary to compile a library. In our case we just need to feed cargo some specially understood flags that tell it where to find our pre-compiled `app` library.

**build.rs:**

```rust
fn main() {
    // Add our app's build directory to the lib search path.
    println!("cargo:rustc-link-search=../app/target/debug");
    // Add the app's dependency directory to the lib search path.
    // This is may be required if the app depends on any external "derive"
    // crates like the `dlopen_derive` crate that we add later.
    println!("cargo:rustc-link-search=../app/target/debug/deps");
    // Link to the `app` crate library. This tells cargo to actually link
    // to the `app` crate that we include using `extern crate app;`.
    println!("cargo:rustc-link-lib=app");
}
```

Now we can run `cargo build` and we will get a new `libplugin1.so` file in our `target/debug` ( if it fails see note below ). As we intended, the plugin only contains the code that is in the plugin and weighs only 14 kilobytes. Yay, we have successfully built a plugin! Lets go over what happened when we built it.

> **note:** If you run cargo build and get an error like `error[E0464]: multiple matching crates for 'app'`, change directory to your app directory and run `cargo clean` followed by `cargo build`. This will get rid of any extra `rlib` file that may have been left over from when we first built our app as a standalone binary. After doing that you should be able to come back to your plugin and successfully run `cargo build` to build the library.

When we run `cargo build`, cargo will first run our `build.rs` script and read the standard output of that script to look for cargo directives. In this case, our script tells cargo to look in the debug build dir of our app for libraries and to link to the `app` library. When compiling our rust library, the compiler will read our app's `libapp.rlib` which contains all of the metadata needed to compile rust code that talks to that library, similar to C/C++ header files. After the rust code is compiled, it will call the system linker to link our plugin library, `libplugin1.so`, to `libapp.so` so that it can call functions defined in our app library.

Now that we have an app and a plugin, we need to make our app load the plugin!

## Loading a Plugin

Now we are ready to actually do some awesome stuff, loading the plugin into our app. To load the plugin we are going to use the [`dlopen`](https://crates.io/crates/dlopen) crate. The `dlopen` crate will do the actual loading of the shared libraries and takes care of the lower level stuff so we don't have to. Our first step, then, is to add that crate to the `Cargo.toml` for our app.

```toml
[dependencies]
dlopen = "0.1.6"
dlopen_derive = "0.1.3"
```

Then update your app's `src/lib.rs` file to look like this:

```rust
#[macro_use]
extern crate dlopen_derive;
use dlopen::wrapper::{Container, WrapperApi};

#[derive(WrapperApi)]
struct PluginApi {
    run: extern fn(),
}

pub fn run() {
    println!("Starting App");

    let plugin_api_wrapper: Container<PluginApi> = unsafe { Container::load("plugins/libplugin1.so") }.unwrap();
    plugin_api_wrapper.run();
}

pub fn test_app_func(message: &str) {
    println!("test_app_func(\"{}\")", message);
}
```

There is a little bit going on here, but it is still fairly simple, thanks to the `dlopen` library. We create a `PluginApi` struct that represents the functions that we can call in loaded plugins. We use `dlopen` to load our plugin shared library, and store it in `plugin_api_wrapper`. We can then call the `run()` function, and it will execute the `run()` function in our plugin. The `run()` function in our plugin should then call `test_app_func` with a message that should be printed to the console.

Before we run it, lets create a `plugins` directory in our app crate directory and copy our `libplugin1.so` file into it from our plugin's build directory. After that, go ahead and test it with `cargo run`:

```bash
> cd app
> mkdir plugins
> cp ../plugin1/target/debug/libplugin1.so plugins
> cargo run
   Compiling app v0.1.0 (/home/zicklag/rust/test-lab/plugins/rust-plugins-test2/project-tutorial/app)
   ...
    Finished dev [unoptimized + debuginfo] target(s) in 1.67s
     Running `target/debug/app`
Starting App
Running plugin1
test_app_func("Hello from plugin 1")
```

It works! You should also be able to run it manually, but you will have to re-copy the libstd library back into the build directory because we ran a `cargo clean` earlier:

```bash
> cp ~/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/lib/libstd-8e7d7d74c91e7cfe.so target/debug/
> ./target/debug/app
Starting App
Running plugin1
test_app_func("Hello from plugin 1")
```

Notice that Rust bundled the new dependencies of our app, such as the `dlopen` crate, into our `libapp.so`; it is now 534 kilobytes instead of the original 14 kilobytes. Apparently, even though it dynamically links `libstd`, it decided to statically link the `dlopen` crate to `libapp`. This is fine and is nice because we don't need to have a shared library for *every* crate dependency. If we wanted to expose one of the crates that our app depends on to our plugins, we could do that simply by re-exporting the library in our app library ( this is yet to be tested ).

## Notes So Far

In this example, we added the path to our app library to our plugin's build script so that we could compile the plugin with a link to our app. In a more organized situation, as the designer of the app, you would probably provide the shared libraries and the rlibs that are required to link to the app for your users, so that they could build their plugins against those, without having to have the source code and compile the app themselves.

What we are going to focus on next is making our plugin API more powerful so that the app has a way to find out more about the plugin, instead of just having a `run` function.

## Improving the Plugin API ( And Our App )

Now that we have basics of plugin loading, lets make our app do something. We're going to setup a simple app that will infinitely prompt for a command, and respond to the user's input. The only command that comes with the app will be the `exit` command that lets the user exit the program. Otherwise, all other commands will be provided by plugins.

Let's get that loop going without plugins first:

**src/lib.rs:**

```rust
// ...
pub fn run() {
    println!("Starting App");

    // Comment this out for now
    // let plugin_api_wrapper: Container<PluginApi> = unsafe { Container::load("plugins/libplugin1.so") }.unwrap();
    // plugin_api_wrapper.run();

    loop {
        // Prompt
        println!("Enter command:");

        // Read input
        let mut message = String::new();
        std::io::stdin().read_line(&mut message).unwrap();

        // Trim newline
        message = message.trim().into();

        // Check command
        if message == "exit" {
            break
        }
    }
}
// ...
```

Now we can `cargo run` our app and get our own little command prompt.

Now we want to refine our plugin API a bit. Instead of using a `run` function to execute our plugins, we are going to use an `get_plugin` function which is expected to return a pointer to a struct that implements a `Plugin` trait. The `Plugin` trait will require that each plugin implement the `handle_command()` function so that it can handle commands pass to the user.

Here is the full updated `app/src/lib.rs`:

```rust
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
```

And our updated `plugin1/src/lib.rs`:

```rust
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
```

Now we can:

* rebuild our app
* rebuild our plugin
* copy the newly built `libplugin1.so` into our app's `plugins/` directory, and
* run our app to get our mini command prompt

Here is an example of the result:

```txt
Starting App
Running plugin1
Enter command:
plugin1
Hey you triggered my 'plugin1' command!
Enter command:
echo hello world
Echo-ing what you said: echo hello world
Enter command:
exit
```

We used our plugin to provide custom commands to our command prompt!

This is as far as this tutorial will take you and there is obviously a lot that could be improved. For one, you probably don't want to be loading plugins by name and you are going to want to be able to have more than one. All of that is simple to implement on top of the base that we have worked on here and I leave it up to the reader to explore how to do that if they so desire.

## Closing Thoughts

This is actually the first time that I have done any of this, so I'm still getting to understand how everything fits together, but hopefully this presents a good picture of how you can setup plugins in Rust.

Many thanks to @Michael-F-Bryan for the plugin section of his [Rust FFI Guide](https://michael-f-bryan.github.io/rust-ffi-guide/). I wouldn't have figure out how to do this without that. I may have missed something or given incorrect instructions somewhere in the tutorial so open an issue if you have any problems with it. :smiley:
