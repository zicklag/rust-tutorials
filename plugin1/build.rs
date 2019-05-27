fn main() {
    println!("cargo:rustc-link-search=../app/target/debug");
    println!("cargo:rustc-link-search=../app/target/debug/deps");
    println!("cargo:rustc-link-search=app");
}
