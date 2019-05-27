extern crate app;

#[no_mangle]
pub fn run() {
    println!("Running plugin1");
    app::test_app_func("Hello from plugin 1");
}
