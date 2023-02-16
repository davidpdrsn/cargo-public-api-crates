use anyhow::{ensure, Context, Result};
use glob::glob;
use std::{path::PathBuf, process::Command};

use crate::find_and_parse_cargo_toml;

pub fn run(manifest_path: Option<PathBuf>, skip_build: bool) -> Result<PathBuf> {
    let (manifest_path, toml) = find_and_parse_cargo_toml(manifest_path)?;
    let package = toml.package.name.replace('-', "_");

    if !skip_build {
        let mut cmd = Command::new("cargo");
        cmd.args(["+nightly", "rustdoc", "--all-features", "--manifest-path"]);
        cmd.args([&manifest_path]);
        cmd.args(["--", "-Z", "unstable-options", "--output-format", "json"]);
        cmd.stdout(std::process::Stdio::null());
        ensure!(cmd.spawn()?.wait()?.success(), "failed to build docs");
    }

    let mut entries = glob(&format!("target/**/doc/{package}.json"))
        .context("failed to read glob pattern")?
        .filter_map(Result::ok);

    let doc_json_path = entries
        .next()
        .with_context(|| format!("{package}.json file not found in target directory"))?;

    Ok(doc_json_path)
}
