use std::io::Write;

use crate::analyze::AnalyzeOutput;
use anyhow::{Context, Result};

pub fn run(analyze_output: AnalyzeOutput) -> Result<()> {
    let AnalyzeOutput {
        krate,
        crate_id_to_public_item,
        id_to_usages,
    } = analyze_output;

    let mut crate_id_to_public_item = crate_id_to_public_item.into_iter().collect::<Vec<_>>();
    crate_id_to_public_item.sort_by(|(a, _), (b, _)| a.cmp(b));

    let mut stdout = std::io::stdout().lock();
    for (crate_id, ids) in crate_id_to_public_item {
        let external_crate = krate
            .external_crates
            .get(&crate_id)
            .context("crate missing")?;
        writeln!(&mut stdout, "{}", external_crate.name)?;

        let mut ids = ids.into_iter().collect::<Vec<_>>();
        ids.sort_by(|a, b| a.0.cmp(&b.0));
        for id in ids {
            let item = krate.paths.get(&id).context("path missing")?;
            let name = item.path.join("::");
            writeln!(&mut stdout, "    {name}")?;

            if let Some(spans) = id_to_usages.get(&id) {
                let mut spans = spans.iter().collect::<Vec<_>>();
                spans.sort_by(|a, b| {
                    (&a.filename, a.begin.0, a.begin.1, a.end.0, a.end.1).cmp(&(
                        &b.filename,
                        b.begin.0,
                        b.begin.1,
                        b.end.0,
                        b.end.1,
                    ))
                });

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
