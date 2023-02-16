use std::{collections::BTreeSet, io::Write, path::PathBuf};

use crate::{analyze::AnalyzeOutput, find_and_parse_cargo_toml};
use anyhow::{Context, Result};

pub fn run(manifest_path: Option<PathBuf>, analyze_output: AnalyzeOutput) -> Result<()> {
    let AnalyzeOutput {
        mut krate,
        crate_id_to_public_item,
        id_to_usages: _,
    } = analyze_output;

    let (_, toml) = find_and_parse_cargo_toml(manifest_path)?;
    let allowed = toml
        .package
        .metadata
        .cargo_public_api_crates
        .allowed
        .into_iter()
        .map(|krate| krate.replace('-', "_"))
        .collect::<BTreeSet<_>>();

    let crates_in_public_api = crate_id_to_public_item
        .into_keys()
        .map(|crate_id| {
            Ok(krate
                .external_crates
                .remove(&crate_id)
                .context("crate missing")?
                .name
                .replace('-', "_"))
        })
        .collect::<Result<BTreeSet<_>>>()?;

    let mut in_api_but_not_allowed = Vec::new();
    let mut allowed_but_not_in_api = Vec::new();

    for krate in &crates_in_public_api {
        if !allowed.contains(krate) {
            in_api_but_not_allowed.push(krate);
        }
    }

    for krate in &allowed {
        if !crates_in_public_api.contains(krate) {
            allowed_but_not_in_api.push(krate);
        }
    }

    let status = if in_api_but_not_allowed.is_empty() && allowed_but_not_in_api.is_empty() {
        0
    } else {
        1
    };

    let mut stdout = std::io::stdout().lock();

    if !in_api_but_not_allowed.is_empty() {
        println!("Crates in public API that weren't allowed:");
        for krate in in_api_but_not_allowed {
            writeln!(&mut stdout, "    {krate}")?;
        }
    }

    if !allowed_but_not_in_api.is_empty() {
        println!("Crates that were allowed but not in public API:");
        for krate in allowed_but_not_in_api {
            writeln!(&mut stdout, "    {krate}")?;
        }
    }

    std::process::exit(status)
}
