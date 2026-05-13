//! Launcher for the `rustc_interface` session.
//!
//! Constructs a [`rustc_interface::Config`], runs the compiler, and extracts
//! the HIR map.  The actual fact collection is delegated to
//! [`visitor::collect_facts`].

use anyhow::{anyhow, Result};
use icb_parser::facts::RawNode;
use std::path::Path;

#[cfg(feature = "nightly")]
pub fn run_analysis(crate_root: &Path, _args: &[String]) -> Result<Vec<RawNode>> {
    use rustc_interface::interface;
    use std::panic;

    let crate_root = crate_root.to_path_buf();

    let result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
        interface::run_compiler(
            interface::Config {
                // Minimal config that should work with a typical nightly
                opts: rustc_session::config::Options::default(),
                input: rustc_session::config::Input::File(crate_root),
                crate_cfg: vec![],
                crate_check_cfg: vec![],
                output_dir: None,
                output_file: None,
                file_loader: None,
                locale_resources: &[],
                lint_caps: Default::default(),
                psess_created: None,
                register_lints: None,
                override_queries: None,
                make_codegen_backend: None,
                registry: vec![],
                using_internal_features: true.into(),
            },
            |compiler| {
                let tcx = compiler.tcx();
                let hir_map = tcx.hir();
                let source_map = tcx.sess.source_map();
                crate::visitor::collect_facts(tcx, hir_map, source_map)
                    .map_err(|e| anyhow!("{}", e))
            },
        )
    }));

    match result {
        Ok(Ok(facts)) => Ok(facts),
        Ok(Err(msg)) => Err(anyhow!("rustc analysis error: {}", msg)),
        Err(p) => Err(anyhow!("rustc panicked: {:?}", p)),
    }
}

#[cfg(not(feature = "nightly"))]
pub fn run_analysis(_: &Path, _: &[String]) -> Result<Vec<RawNode>> {
    Ok(vec![])
}
