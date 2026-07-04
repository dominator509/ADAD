fn main() {
    let name = env!("CARGO_PKG_NAME");
    let version = adad_core::version();

    if std::env::args().any(|arg| arg == "--version") {
        println!("{name} {version}");
        return;
    }

    println!("{name} {version}");
}
