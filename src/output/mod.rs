use std::{
    collections::{HashMap, HashSet},
    io::Write,
};

use crate::analyze::AnalyzeOutput;
use anyhow::{Context, Result};
use itertools::{Itertools, Position};
use rustdoc_types::{Crate, Id, Span};

use self::writer::{LendingIterator, Writer};

mod writer;

pub fn run(analyze_output: AnalyzeOutput) -> Result<()> {
    let AnalyzeOutput {
        krate,
        crate_id_to_public_item,
        id_to_usages,
    } = analyze_output;

    let mut crate_id_to_public_item = crate_id_to_public_item.into_iter().collect::<Vec<_>>();
    crate_id_to_public_item.sort_by(|(a, _), (b, _)| a.cmp(b));

    let mut stdout = std::io::stdout().lock();

    for item in crate_id_to_public_item.into_iter().with_position() {
        let (crate_id, ids, last) = match item {
            Position::First((crate_id, ids)) | Position::Middle((crate_id, ids)) => {
                (crate_id, ids, false)
            }
            Position::Last((crate_id, ids)) | Position::Only((crate_id, ids)) => {
                (crate_id, ids, true)
            }
        };

        output_crate(crate_id, ids, &krate, &id_to_usages, &mut stdout)?;

        if !last {
            writeln!(&mut stdout)?;
        }
    }

    Ok(())
}

fn output_crate(
    crate_id: u32,
    ids: HashSet<Id>,
    krate: &Crate,
    id_to_usages: &HashMap<Id, HashSet<Span>>,
    out: &mut dyn Write,
) -> anyhow::Result<()> {
    let external_crate = krate
        .external_crates
        .get(&crate_id)
        .context("crate missing")?;
    writeln!(out, "{}", external_crate.name)?;

    let mut ids = ids.into_iter().collect::<Vec<_>>();
    ids.sort_by(|a, b| a.0.cmp(&b.0));

    let mut w = Writer::new(4, out);

    let mut iter = w.iter(ids);
    while let Some((mut w, id)) = iter.next() {
        let item = krate.paths.get(&id).context("path missing")?;
        let name = item.path.join("::");
        w.write_line(format_args!("{name}"))?;

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
                let mut iter = w.iter(spans);
                while let Some((mut w, span)) = iter.next() {
                    w.write_line(format_args!(
                        "{}:{}:{}",
                        span.filename.display(),
                        span.begin.0,
                        span.begin.1
                    ))?;
                }
            } else {
                let mut iter = w.iter(
                    spans
                        .iter()
                        .take(max_show)
                        .map(Some)
                        .chain(std::iter::once(None)),
                );
                while let Some((mut w, span)) = iter.next() {
                    match span {
                        Some(span) => {
                            w.write_line(format_args!(
                                "{}:{}:{}",
                                span.filename.display(),
                                span.begin.0,
                                span.begin.1
                            ))?;
                        }
                        None => {
                            w.write_line(format_args!("and {} more...", spans.len() - max_show))?;
                        }
                    }
                }
            }
        }
    }

    Ok(())
}
