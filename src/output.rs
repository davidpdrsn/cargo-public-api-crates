use std::io::Write;

use crate::analyze::AnalyzeOutput;
use anyhow::{Context, Result};

pub fn run(analyze_output: AnalyzeOutput) -> Result<()> {
    let AnalyzeOutput {
        krate,
        crate_id_to_public_item,
        id_to_usages,
    } = analyze_output;

    // TODO(david): sort output, indexmap doesn't do it

    let mut stdout = std::io::stdout().lock();
    for (crate_id, ids) in crate_id_to_public_item {
        let external_crate = krate
            .external_crates
            .get(&crate_id)
            .context("crate missing")?;
        writeln!(&mut stdout, "{}", external_crate.name)?;
        for id in ids {
            let item = krate.paths.get(&id).context("path missing")?;
            let name = item.path.join("::");
            writeln!(&mut stdout, "    {name}")?;
            if let Some(spans) = id_to_usages.get(&id) {
                let max_show = 3;
                if spans.len() <= max_show {
                    for span in spans {
                        writeln!(
                            &mut stdout,
                            "        {}:{}:{}",
                            span.filename.display(),
                            span.begin.0,
                            span.begin.1
                        )?;
                    }
                } else {
                    for span in spans.iter().take(max_show) {
                        writeln!(
                            &mut stdout,
                            "        {}:{}:{}",
                            span.filename.display(),
                            span.begin.0,
                            span.begin.1
                        )?;
                    }
                    writeln!(
                        &mut stdout,
                        "        and {} more...",
                        spans.len() - max_show
                    )?;
                }
            }
        }
    }

    Ok(())
}
