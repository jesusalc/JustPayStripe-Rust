use std::fs;

fn main() {
    let cargo_toml = fs::read_to_string("Cargo.toml").expect("Failed to read Cargo.toml");
    let cargo: toml::Value = cargo_toml.parse().expect("Failed to parse Cargo.toml");

    if let Some(version) = cargo
        .get("package")
        .and_then(|pkg| pkg.get("version"))
        .and_then(|v| v.as_str())
    {
        println!("cargo:rustc-env=CARGO_PKG_VERSION={}", version);
    }
    if let Some(description) = cargo
        .get("package")
        .and_then(|pkg| pkg.get("description"))
        .and_then(|v| v.as_str())
    {
        println!("cargo:rustc-env=CARGO_PKG_DESCRIPTION={}", description);
    }
    if let Some(name) = cargo
        .get("package")
        .and_then(|pkg| pkg.get("name"))
        .and_then(|v| v.as_str())
    {
        println!("cargo:rustc-env=CARGO_PKG_NAME={}", name);
    }
}
