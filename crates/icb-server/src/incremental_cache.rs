//! Incremental fact cache for the ICB server.
//!
//! # Purpose
//!
//! Avoid re‑parsing source files that have not changed since the previous
//! analysis.  The first run extracts facts from every file and saves them
//! into a cache directory together with a SHA‑256 hash of the file content.
//! On subsequent runs the hash is compared; if it matches, the facts are
//! loaded directly from the cache, **skipping the parser entirely**.
//!
//! # Cache layout
//!
//! Given a cache directory (e.g. `.icb_cache`) and a file
//! `modules/api/src/admin_controller.cpp`, two files are created:
//!
//! ```text
//! .icb_cache/
//!   modules_api_src_admin_controller_cpp.facts.bincode
//!   modules_api_src_admin_controller_cpp.hash
//! ```
//!
//! The sanitised name replaces every `/` and `\` with `_`.
//!
//! # Safety
//!
//! The hash is computed with SHA‑256, making accidental collisions
//! practically impossible.  The facts are serialised with `bincode`, which
//! is fast and produces a compact binary representation.

use anyhow::anyhow;
use icb_parser::facts::RawNode;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};

/// A boxed closure that takes a source string and returns parsed facts.
pub type ParseFn = Box<dyn FnOnce(&str) -> anyhow::Result<Vec<RawNode>>>;

/// Stores facts for a single file, along with the relative path.
pub struct FileFacts {
    pub relative_path: String,
    pub facts: Vec<RawNode>,
}

/// Manages the cache directory and provides the core `process_file` method.
pub struct IncrementalCache {
    cache_dir: PathBuf,
}

impl IncrementalCache {
    /// Create a new cache manager.
    ///
    /// `cache_dir` will be created if it does not exist.
    pub fn new(cache_dir: &Path) -> anyhow::Result<Self> {
        fs::create_dir_all(cache_dir)?;
        Ok(Self {
            cache_dir: cache_dir.to_path_buf(),
        })
    }

    /// Process a single source file.
    ///
    /// * `file_path` – absolute path to the source file.
    /// * `relative_path` – the path that will be used in the cache name
    ///   (usually the path relative to the project root).
    /// * `parse_fn` – a boxed closure that parses the source and returns facts;
    ///   it is called **only** if the file has changed or is not cached.
    ///
    /// Returns [`FileFacts`] containing the extracted or cached facts.
    pub fn process_file(
        &self,
        file_path: &Path,
        relative_path: &str,
        parse_fn: ParseFn,
    ) -> anyhow::Result<FileFacts> {
        let (facts_path, hash_path) = self.cache_paths(relative_path);

        let source = fs::read_to_string(file_path)
            .map_err(|e| anyhow!("cannot read {}: {}", file_path.display(), e))?;
        let current_hash = hex::encode(Sha256::digest(source.as_bytes()));

        if let Ok(saved_hash) = fs::read_to_string(&hash_path) {
            if saved_hash.trim() == current_hash && facts_path.exists() {
                let data = fs::read(&facts_path)?;
                let facts: Vec<RawNode> = bincode::deserialize(&data)
                    .map_err(|e| anyhow!("cache deserialisation error: {}", e))?;
                return Ok(FileFacts {
                    relative_path: relative_path.to_string(),
                    facts,
                });
            }
        }

        let facts = parse_fn(&source)?;

        let data = bincode::serialize(&facts)
            .map_err(|e| anyhow!("bincode serialisation error: {}", e))?;
        fs::write(&facts_path, data)?;
        fs::write(&hash_path, current_hash)?;

        Ok(FileFacts {
            relative_path: relative_path.to_string(),
            facts,
        })
    }

    fn cache_paths(&self, relative_path: &str) -> (PathBuf, PathBuf) {
        let sanitised = relative_path.replace(['/', '\\'], "_").replace(':', "_");
        let facts_path = self.cache_dir.join(format!("{}.facts.bincode", sanitised));
        let hash_path = self.cache_dir.join(format!("{}.hash", sanitised));
        (facts_path, hash_path)
    }
}
