use std::{
    collections::{HashMap, HashSet},
    path::Path,
};

use crate::visit::{self, Visitor};
use anyhow::Result;
use rustdoc_types::{Crate, Id, Import, Span};

pub struct AnalyzeOutput {
    pub krate: Crate,
    pub crate_id_to_public_item: HashMap<u32, HashSet<Id>>,
    pub id_to_usages: HashMap<Id, HashSet<Span>>,
}

pub fn run(doc_json_path: &Path, include_std: bool) -> Result<AnalyzeOutput> {
    let krate = serde_json::from_str::<Crate>(&std::fs::read_to_string(doc_json_path)?)?;

    let mut crate_id_to_public_item: HashMap<u32, HashSet<Id>> = <_>::default();
    let mut id_to_usages: HashMap<Id, HashSet<Span>> = <_>::default();

    for item in krate.index.values() {
        // don't search through items defined in external crates
        if krate.external_crates.contains_key(&item.crate_id) {
            continue;
        }

        let mut item_visitor = ItemVisitor {
            krate: &krate,
            crate_id_to_public_item: <_>::default(),
            include_std,
        };
        visit::visit_item(item, &mut item_visitor);

        for (crate_id, ids) in item_visitor.crate_id_to_public_item {
            if let Some(span) = &item.span {
                for id in &ids {
                    id_to_usages
                        .entry(id.clone())
                        .or_default()
                        .insert(span.clone());
                }
            }

            crate_id_to_public_item
                .entry(crate_id)
                .or_default()
                .extend(ids);
        }
    }

    Ok(AnalyzeOutput {
        krate,
        crate_id_to_public_item,
        id_to_usages,
    })
}

struct ItemVisitor<'a> {
    krate: &'a Crate,
    crate_id_to_public_item: HashMap<u32, HashSet<Id>>,
    include_std: bool,
}

impl<'a> Visitor for ItemVisitor<'a> {
    fn visit_path(&mut self, path: &rustdoc_types::Path) {
        self.on_id(&path.id);
    }

    fn visit_import(&mut self, import: &Import) {
        let Some(id) = &import.id else { return };
        self.on_id(id);
    }
}

impl<'a> ItemVisitor<'a> {
    fn on_id(&mut self, id: &Id) {
        let Some(item) = self.krate.paths.get(id) else { return };
        let Some(krate) = self.krate.external_crates.get(&item.crate_id) else { return };

        if !self.include_std
            && (krate.name == "std" || krate.name == "alloc" || krate.name == "core")
        {
            return;
        }

        self.crate_id_to_public_item
            .entry(item.crate_id)
            .or_default()
            .insert(id.clone());
    }
}
