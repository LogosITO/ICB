🧊 ICB — Infinite Code Blueprint

[![CI](https://github.com/LogosITO/ICB/actions/workflows/ci.yml/badge.svg)](https://github.com/LogosITO/ICB/actions/workflows/ci.yml)

**Multi-language call-graph analysis toolkit** powered by Clang and tree-sitter backends with an interactive dashboard and incremental caching support.

> **Project is in active development.**  
> API may change, documentation is being improved. Stay tuned for updates.

---

## ✨ What's Already Built

- **Multi-language support** — Call-graph analysis for C, C++, Rust, TypeScript, and more via Clang and tree-sitter backends
- **Code Property Graph (CPG)** — Universal graph representation of source code for advanced querying and pattern matching
- **Interactive Dashboard** — Web-based UI for exploring and visualizing call graphs
- **Incremental Caching** — Smart caching system for faster re-analysis of unchanged code
- **Scalable Architecture** — Rust-powered backend for performance and reliability

---

## 📚 Documentation

Auto-generated documentation from source code:

- **Local view**
  ```bash
  cargo doc --workspace --no-deps
  # Open target/doc/icb_core/index.html
  ```
- **Online** – https://logosito.github.io/ICB/

---

## 🚀 Quick Start

```bash
# Build the project
cargo build --release

# Run tests
cargo test

# Generate docs
cargo doc --workspace --no-deps --open
```

---

## 📖 License

Check LICENSE file for details.
