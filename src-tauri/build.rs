fn main() {
    // The E2E build (--features wdio) needs the WebdriverIO permissions, which
    // reference plugins that are not compiled into normal builds — so we swap
    // the capability set per feature. `capabilities/` is used everywhere;
    // `capabilities-e2e/` is only added when the wdio feature is enabled.
    println!("cargo:rerun-if-changed=capabilities");
    println!("cargo:rerun-if-changed=capabilities-e2e");
    let mut attributes = tauri_build::Attributes::new();
    if std::env::var("CARGO_FEATURE_WDIO").is_ok() {
        // NOTE: the Rust `glob` crate has no brace alternation, so match both
        // `capabilities/` and `capabilities-e2e/` with a single wildcard.
        attributes = attributes.capabilities_path_pattern("./capabilities*/**/*");
    } else {
        attributes = attributes.capabilities_path_pattern("./capabilities/**/*");
    }
    tauri_build::try_build(attributes).expect("failed to run tauri-build");
}
