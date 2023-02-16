use std::{env, path::PathBuf};

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use serde::de::DeserializeOwned;

mod analyze;
mod build_docs;
mod check;
mod output;
mod visit;

#[derive(Parser, Debug)]
struct Args {
    /// Include types defined in `std`, `alloc`, and `core`.
    #[arg(long)]
    include_std: bool,

    /// Path to Cargo.toml
    #[arg(long)]
    manifest_path: Option<PathBuf>,

    /// Skip building the documentation.
    #[arg(long)]
    skip_build: bool,

    #[command(subcommand)]
    cmd: Option<Command>,
}

#[derive(Debug, Subcommand)]
enum Command {
    Check,
}

fn main() -> Result<()> {
    let raw_args = env::args().skip(if running_as_cargo_cmd() { 1 } else { 0 });
    let Args {
        include_std,
        manifest_path,
        skip_build,
        cmd,
    } = Args::parse_from(raw_args);

    let doc_json_path = build_docs::run(manifest_path.clone(), skip_build)?;
    let analyze_output = analyze::run(&doc_json_path, include_std)?;

    match cmd {
        Some(Command::Check) => {
            check::run(manifest_path, analyze_output)?;
        }
        None => {
            output::run(analyze_output)?;
        }
    }

    Ok(())
}

// https://github.com/bnjbvr/cargo-machete/blob/main/src/main.rs#L95
fn running_as_cargo_cmd() -> bool {
    env::var("CARGO").is_ok() && env::var("CARGO_PKG_NAME").is_err()
}

fn find_and_parse_cargo_toml<T>(manifest_path: Option<PathBuf>) -> Result<(PathBuf, T)>
where
    T: DeserializeOwned,
{
    let manifest_path = manifest_path.unwrap_or_else(|| PathBuf::from("Cargo.toml"));

    let toml = std::fs::read_to_string(&manifest_path)
        .with_context(|| format!("failed to read {}", manifest_path.display()))?;

    let toml = toml::from_str(&toml)
        .with_context(|| format!("failed to parse {}", manifest_path.display()))?;

    Ok((manifest_path, toml))
}
