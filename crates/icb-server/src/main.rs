//! # icb-server
//!
//! Web server for ICB graph visualization and analytics.
//!
//! Serves static files from `web/` and exposes a REST API to explore
//! the Code Property Graph, retrieve node details, collect project
//! metrics, compare snapshots, upload ZIP archives, and **load new
//! projects on‑the‑fly**.  All endpoints that build a graph respect the
//! `--no-system-headers` flag passed to the server.
//!
//! # Quick start
//!
//! ```bash
//! # Start with a project (Clang, no system headers)
//! cargo run -p icb-server -- --project ./my_project --language cpp --no-system-headers
//!
//! # Start empty, then load from UI
//! cargo run -p icb-server
//! ```

use actix_files::Files;
use actix_web::{web, App, HttpServer};
use clap::Parser;
use icb_graph::graph::CodePropertyGraph;
use std::path::PathBuf;
use std::sync::Mutex;

mod analytics;
mod diff;
mod display_name;
mod graph_builder;
mod incremental_cache;
mod routes;
mod upload;

/// Command‑line arguments for the ICB server.
#[derive(Parser)]
#[command(name = "icb-server")]
#[command(about = "ICB web server for graph visualization")]
pub struct Cli {
    /// Path to a project directory or file (optional – can be loaded later via the UI).
    #[arg(short, long)]
    pub project: Option<PathBuf>,

    /// Programming language of the initial project (default: "python").
    #[arg(short, long, default_value = "python")]
    pub language: String,

    /// Path to a compile_commands.json file (C/C++ only).
    #[arg(long)]
    pub compile_commands: Option<PathBuf>,

    /// C++ standard version (default: "c++17").
    #[arg(long, default_value = "c++17")]
    pub cpp_std: String,

    /// Cache file for the whole graph (fast reload).
    #[arg(long)]
    pub cache: Option<PathBuf>,

    /// Directory for incremental per‑file fact caching.
    #[arg(long = "cache-dir")]
    pub cache_dir: Option<PathBuf>,

    /// Port to listen on (default: 8080).
    #[arg(short = 'P', long, default_value = "8080")]
    pub port: u16,

    /// Directory containing static files (frontend).
    #[arg(long, default_value = "web")]
    pub static_dir: PathBuf,

    /// Exclude system header nodes (C/C++ only).
    #[arg(long)]
    pub no_system_headers: bool,

    /// Maximum directory depth when scanning for files.
    #[arg(long)]
    pub max_depth: Option<usize>,
}

/// Main entry point: starts the HTTP server.
///
/// If `--project` is given, a graph is built from that project at startup.
/// Otherwise an **empty graph** is created and projects can be loaded
/// dynamically via the `/api/load` and `/api/upload` endpoints.
#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();
    let args = Cli::parse();

    // Build initial graph (or start empty)
    let graph = if let Some(ref project) = args.project {
        graph_builder::build_or_load_graph(
            project,
            &args.language,
            args.cache.as_ref(),
            args.cache_dir.as_ref(),
            args.no_system_headers,
        )?
    } else {
        log::info!("No initial project; starting with empty graph");
        CodePropertyGraph::new()
    };

    let graph_data = web::Data::new(Mutex::new(graph));

    let static_dir = args
        .static_dir
        .canonicalize()
        .unwrap_or_else(|_| args.static_dir.clone());
    if !static_dir.exists() {
        eprintln!("Warning: static directory {:?} does not exist", static_dir);
    } else {
        println!("Serving static files from {:?}", static_dir);
    }
    println!("Open http://localhost:{}", args.port);

    HttpServer::new(move || {
        App::new()
            .wrap(actix_cors::Cors::permissive())
            .app_data(web::PayloadConfig::new(100 * 1024 * 1024))
            .app_data(graph_data.clone())
            .configure(routes::configure)
            .service(
                Files::new("/", static_dir.clone())
                    .index_file("index.html")
                    .prefer_utf8(true),
            )
    })
    .bind(("127.0.0.1", args.port))?
    .run()
    .await?;

    Ok(())
}
