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

    // Windows: Tauri test binaries crash at startup with STATUS_ENTRYPOINT_NOT_FOUND
    // (0xc0000139) unless they carry the Common Controls v6 manifest —
    // TaskDialogIndirect is imported from comctl32 v6. tauri-build only embeds a
    // manifest for the app binary, not for test targets, so embed one here for the
    // `lib_tests` test target (see windows-app-manifest.xml + [[test]] in Cargo.toml).
    // `cargo:rustc-link-arg-tests` requires a declared test target, which is why
    // the [[test]] entry exists; on Windows CI we run `cargo test --test lib_tests`.
    #[cfg(all(target_os = "windows", target_env = "msvc"))]
    {
        let manifest_path = std::path::Path::new("windows-app-manifest.xml")
            .canonicalize()
            .expect("windows-app-manifest.xml not found");
        println!("cargo:rerun-if-changed=windows-app-manifest.xml");
        println!("cargo:rustc-link-arg-tests=/MANIFEST:EMBED");
        println!(
            "cargo:rustc-link-arg-tests=/MANIFESTINPUT:{}",
            manifest_path.display()
        );
    }
}
