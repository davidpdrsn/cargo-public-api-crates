use anyhow::{ensure, Context, Result};
use glob::glob;
use serde::Deserialize;
use std::{path::PathBuf, process::Command};

pub fn run(manifest_path: Option<PathBuf>) -> Result<PathBuf> {
    let manifest_path = manifest_path.unwrap_or_else(|| PathBuf::from("Cargo.toml"));

    let toml = std::fs::read_to_string(&manifest_path)
        .with_context(|| format!("failed to read {}", manifest_path.display()))?;
    let toml = toml::from_str::<CargoToml>(&toml)
        .with_context(|| format!("failed to parse {}", manifest_path.display()))?;
    let package = toml.package.name.replace('-', "_");

    let mut cmd = Command::new("cargo");
    cmd.args(["+nightly", "rustdoc", "--all-features", "--manifest-path"]);
    cmd.args([&manifest_path]);
    cmd.args(["--", "-Z", "unstable-options", "--output-format", "json"]);
    cmd.stdout(std::process::Stdio::null());

    ensure!(cmd.spawn()?.wait()?.success(), "failed to build docs");

    let mut entries = glob(&format!("target/**/doc/{package}.json"))
        .context("failed to read glob pattern")?
        .filter_map(Result::ok);

    let doc_json_path = entries
        .next()
        .with_context(|| format!("{package}.json file not found in target directory"))?;

    Ok(doc_json_path)
}

#[derive(Deserialize, Debug)]
struct CargoToml {
    package: Package,
}

#[derive(Deserialize, Debug)]
struct Package {
    name: String,
}
