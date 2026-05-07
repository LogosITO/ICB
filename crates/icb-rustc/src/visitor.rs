//! HIR visitor that collects ICB facts.
//!
//! When the `nightly` feature is enabled, this module traverses the HIR
//! tree and produces [`RawNode`] entries.
//! Without the feature it is an empty stub.

use anyhow::Result;
use icb_parser::facts::RawNode;

#[cfg(feature = "nightly")]
mod nightly_impl {
    use super::*;
    use icb_common::{Language, NodeKind};
    use rustc_hir as hir;
    use rustc_middle::ty::TyCtxt;
    use rustc_span::source_map::Spanned;

    pub fn run(tcx: TyCtxt<'_>, hir_map: &hir::map::Map<'_>) -> Result<Vec<RawNode>> {
        let mut facts = Vec::new();
        hir_map.for_each_item(|_item_id, item| {
            visit_item(item, &mut facts);
        });
        Ok(facts)
    }

    fn visit_item(item: &hir::Item<'_>, facts: &mut Vec<RawNode>) {
        let kind = match item.kind {
            hir::ItemKind::Fn(..) => NodeKind::Function,
            hir::ItemKind::Mod(..) => return,
            hir::ItemKind::Struct(..) | hir::ItemKind::Enum(..) | hir::ItemKind::Trait(..) => {
                NodeKind::Class
            }
            _ => return,
        };

        let name = item.ident.name.to_string();
        let span = item.span;
        let (start_line, start_col, end_line, end_col) = span_to_position(span);

        facts.push(RawNode {
            language: Language::Rust,
            kind,
            name: Some(name),
            usr: None,
            start_line,
            start_col,
            end_line,
            end_col,
            children: vec![],
            source_file: Some(span.source_file().name.to_string()),
        });
    }

    fn span_to_position(span: rustc_span::Span) -> (usize, usize, usize, usize) {
        (0, 0, 0, 0)
    }
}

/// Public visitor entry point.
///
/// On non‑nightly builds this returns an empty vector.
pub fn collect_facts(_tcx: (), _hir_map: ()) -> Result<Vec<RawNode>> {
    #[cfg(feature = "nightly")]
    {
        nightly_impl::run(_tcx, _hir_map)
    }
    #[cfg(not(feature = "nightly"))]
    {
        Ok(vec![])
    }
}