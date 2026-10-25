//! End-to-end check of the official Rust plugin SDK (`sdk/rust`) against Rust
//! Core, with the plugin backend scenario in `plugin_sdk/backend.rs`.
//!
//! The sample is installed the way a plugin ships: `plugin.json`, the
//! backend executable at its packaged path, and the web page that calls it.
//! It runs only when `LITHE_PLUGIN_SDK_RUST_SAMPLE` names the built
//! `sample-backend` executable; `scripts/sdk-e2e.mjs` sets it.

mod plugin_sdk;

#[test]
fn the_rust_sdk_sample_backend_serves_functions_and_host_credentials() {
    let Some(executable) = std::env::var_os("LITHE_PLUGIN_SDK_RUST_SAMPLE") else {
        eprintln!("skipped: set LITHE_PLUGIN_SDK_RUST_SAMPLE to run the Rust SDK end-to-end check");
        return;
    };
    let sample = |file: &str| {
        (
            file.to_string(),
            plugin_sdk::repository_path(&format!("sdk/rust/examples/sample-backend/{file}")),
        )
    };
    let files = [
        sample("plugin.json"),
        sample("ui/index.html"),
        sample("ui/main.js"),
        sample("ui/style.css"),
        ("bin/sample-backend".to_string(), executable.into()),
    ];
    let files: Vec<_> = files
        .iter()
        .map(|(destination, source)| (destination.as_str(), source.clone()))
        .collect();
    let package = plugin_sdk::TemporaryPackage::new("rust", &files);
    plugin_sdk::backend::run_backend_scenario(&package.root);
}
