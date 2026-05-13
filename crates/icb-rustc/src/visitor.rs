//! HIR visitor that collects ICB facts.
//!
//! When the `nightly` feature is enabled, this module traverses the HIR
//! tree of a compiled crate and produces [`RawNode`] entries.
//! Without the feature it is an empty stub.

use anyhow::Result;
use icb_parser::facts::RawNode;

#[cfg(feature = "nightly")]
mod nightly_impl {
    use super::*;
    use icb_common::{Language, NodeKind};
    use rustc_hir as hir;
    use rustc_middle::ty::TyCtxt;
    use rustc_span::source_map::SourceMap;
    use rustc_span::Span;

    pub fn run(
        tcx: TyCtxt<'_>,
        hir_map: &hir::map::Map<'_>,
        source_map: &SourceMap,
    ) -> Result<Vec<RawNode>> {
        let mut collector = FactCollector {
            facts: Vec::new(),
            tcx,
            source_map,
        };
        hir_map.for_each_item(|_item_id, item| {
            collector.visit_item(item);
        });
        Ok(collector.facts)
    }

    struct FactCollector<'tcx> {
        facts: Vec<RawNode>,
        tcx: TyCtxt<'tcx>,
        source_map: &'tcx SourceMap,
    }

    impl<'tcx> FactCollector<'tcx> {
        fn push_node(
            &mut self,
            kind: NodeKind,
            name: String,
            span: Span,
            usr: Option<String>,
        ) -> usize {
            let pos = span_to_position(span, self.source_map);
            let idx = self.facts.len();
            self.facts.push(RawNode {
                language: Language::Rust,
                kind,
                name: Some(name),
                usr,
                start_line: pos.0,
                start_col: pos.1,
                end_line: pos.2,
                end_col: pos.3,
                children: vec![],
                source_file: Some(
                    self.source_map
                        .filename_for_diagnostics(&self.source_map.span_to_filename(span))
                        .to_string(),
                ),
            });
            idx
        }

        fn visit_item(&mut self, item: &hir::Item<'_>) {
            let usr = Some(def_path_hash(self.tcx, item.def_id));
            match item.kind {
                hir::ItemKind::Fn(_, _, body_id) => {
                    let name = item.ident.name.to_string();
                    let idx = self.push_node(NodeKind::Function, name, item.span, usr);
                    self.visit_body(body_id, idx);
                }
                hir::ItemKind::Mod(..) => {}
                hir::ItemKind::Struct(..)
                | hir::ItemKind::Enum(..)
                | hir::ItemKind::Trait(..)
                | hir::ItemKind::Union(..) => {
                    let name = item.ident.name.to_string();
                    self.push_node(NodeKind::Class, name, item.span, usr);
                }
                hir::ItemKind::Impl(ref impl_) => {
                    let name = format!(
                        "impl {}",
                        impl_
                            .self_ty
                            .span
                            .source_text(self.source_map)
                            .unwrap_or_else(|| "?".into())
                    );
                    let idx = self.push_node(NodeKind::Class, name, item.span, usr);
                    for &item_id in &impl_.items {
                        if let Some(method) = self.tcx.hir().item(item_id) {
                            self.visit_item(method);
                        }
                    }
                }
                _ => {}
            }
        }

        fn visit_body(&mut self, body_id: hir::BodyId, parent_idx: usize) {
            let body = self.tcx.hir().body(body_id);
            self.walk_expr(&body.value, parent_idx);
        }

        fn walk_expr(&mut self, expr: &hir::Expr<'_>, parent_idx: usize) {
            match expr.kind {
                hir::ExprKind::Call(ref func, ref args) => {
                    let name = resolve_call_name(func, self.source_map);
                    if !name.is_empty() {
                        self.push_node(NodeKind::CallSite, name, expr.span, None);
                    }
                    for a in args {
                        self.walk_expr(a, parent_idx);
                    }
                }
                hir::ExprKind::MethodCall(ref segment, _, ref args, _) => {
                    let name = segment.ident.name.to_string();
                    if !name.is_empty() {
                        self.push_node(NodeKind::CallSite, name, expr.span, None);
                    }
                    for a in args {
                        self.walk_expr(a, parent_idx);
                    }
                }
                hir::ExprKind::Block(ref blk, _) => {
                    for stmt in &blk.stmts {
                        if let hir::StmtKind::Expr(ref e) = stmt.kind {
                            self.walk_expr(e, parent_idx);
                        }
                    }
                    if let Some(ref tail) = blk.expr {
                        self.walk_expr(tail, parent_idx);
                    }
                }
                _ => {
                    expr.walk(|e| {
                        self.walk_expr(e, parent_idx);
                        true
                    });
                }
            }
        }
    }

    fn resolve_call_name(expr: &hir::Expr<'_>, source_map: &SourceMap) -> String {
        match expr.kind {
            hir::ExprKind::Path(ref qpath) => {
                let path = qpath.path;
                if let Some(segment) = path.segments.last() {
                    segment.ident.name.to_string()
                } else {
                    String::new()
                }
            }
            hir::ExprKind::MethodCall(ref segment, ..) => segment.ident.name.to_string(),
            _ => expr.span.source_text(source_map).unwrap_or_default(),
        }
    }

    fn span_to_position(span: Span, source_map: &SourceMap) -> (usize, usize, usize, usize) {
        if span.is_dummy() {
            return (0, 0, 0, 0);
        }
        let lo = source_map.lookup_char_pos(span.lo());
        let hi = source_map.lookup_char_pos(span.hi());
        (lo.line, lo.col.0, hi.line, hi.col.0)
    }

    fn def_path_hash(tcx: TyCtxt<'_>, def_id: hir::def_id::DefId) -> String {
        let hash = tcx.def_path_hash(def_id);
        format!("{:?}", hash)
    }
}

pub fn collect_facts(_tcx: (), _hir_map: (), _source_map: ()) -> Result<Vec<RawNode>> {
    #[cfg(feature = "nightly")]
    {
        nightly_impl::run(_tcx, _hir_map, _source_map)
    }
    #[cfg(not(feature = "nightly"))]
    {
        Ok(vec![])
    }
}
