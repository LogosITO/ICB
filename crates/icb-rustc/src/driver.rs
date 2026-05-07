//! Launcher for the `rustc_interface` session.
//!
//! When the `nightly` feature is enabled, this module calls into the
//! compiler, obtains the HIR, and returns extracted facts.
//! Without the feature it compiles as a no‑op.

use anyhow::Result;
use icb_parser::facts::RawNode;
use std::path::Path;

#[cfg(feature = "nightly")]
mod nightly_impl {
    use anyhow::{anyhow, bail, Context};
    use icb_parser::facts::RawNode;
    use std::panic;
    use std::path::PathBuf;

    pub fn run(crate_root: &Path, args: &[String]) -> Result<Vec<RawNode>> {
        use rustc_interface::interface;
        let crate_root = crate_root.to_path_buf();
        let args = args.to_vec();

        let result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
            interface::run_compiler(
                interface::Config {
                    input: interface::Input::File(crate_root.clone()),
                    crate_name: Some("analysis_target".to_string()),
                    args,
                    using_sysroot: true,
                    ..Default::default()
                },
                |compiler| {
                    let tcx = compiler.tcx();
                    let hir_map = tcx.hir();
                    crate::visitor::collect_facts(tcx, hir_map)
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
}

/// Public driver entry point.
///
/// On non‑nightly builds this simply returns an empty vector.
pub fn run_rustc_analysis(_crate_root: &Path, _args: &[String]) -> Result<Vec<RawNode>> {
    #[cfg(feature = "nightly")]
    {
        nightly_impl::run(_crate_root, _args)
    }
    #[cfg(not(feature = "nightly"))]
    {
        log::warn!("icb-rustc driver: nightly feature not enabled");
        Ok(vec![])
    }
}