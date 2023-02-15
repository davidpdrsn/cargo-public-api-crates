use std::{env, path::PathBuf};

use anyhow::Result;
use clap::Parser;

mod analyze;
mod build_docs;
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
}

fn main() -> Result<()> {
    let raw_args = env::args().skip(if running_as_cargo_cmd() { 1 } else { 0 });
    let Args {
        include_std,
        manifest_path,
    } = Args::parse_from(raw_args);

    let doc_json_path = build_docs::run(manifest_path)?;

    let analyze_output = analyze::run(&doc_json_path, include_std)?;
    output::run(analyze_output)?;

    Ok(())
}

// https://github.com/bnjbvr/cargo-machete/blob/main/src/main.rs#L95
fn running_as_cargo_cmd() -> bool {
    env::var("CARGO").is_ok() && env::var("CARGO_PKG_NAME").is_err()
}
