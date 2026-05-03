//! # icb-server
//!
//! Web server for ICB graph visualization and analytics.
//!
//! Serves static files from `web/` and exposes a REST API to explore
//! the Code Property Graph, retrieve node details, collect project
//! metrics, and compare two project snapshots.

use actix_files::Files;
use actix_web::{web, App, HttpServer};
use clap::Parser;
use std::path::PathBuf;
use std::sync::Mutex;

mod analytics;
mod diff;
mod display_name;
mod graph_builder;
mod routes;

#[derive(Parser)]
#[command(name = "icb-server")]
#[command(about = "ICB web server for graph visualization")]
pub struct Cli {
    #[arg(short, long)]
    pub project: PathBuf,

    #[arg(short, long, default_value = "python")]
    pub language: String,

    #[arg(long)]
    pub compile_commands: Option<PathBuf>,

    #[arg(long, default_value = "c++17")]
    pub cpp_std: String,

    #[arg(long)]
    pub cache: Option<PathBuf>,

    #[arg(short = 'P', long, default_value = "8080")]
    pub port: u16,

    #[arg(long, default_value = "web")]
    pub static_dir: PathBuf,

    #[arg(long)]
    pub no_system_headers: bool,

    #[arg(long)]
    pub max_depth: Option<usize>,
}

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();
    let args = Cli::parse();

    let graph = graph_builder::build_or_load_graph(
        &args.project,
        &args.language,
        args.compile_commands.as_ref(),
        &args.cpp_std,
        args.cache.as_ref(),
        args.no_system_headers,
        args.max_depth,
    )?;

    let graph_data = web::Data::new(Mutex::new(graph));
    let static_dir = args.static_dir.canonicalize().unwrap_or(args.static_dir);
    if !static_dir.exists() {
        eprintln!("Warning: static directory {:?} does not exist", static_dir);
    }
    println!("Serving static files from {:?}", static_dir);
    println!("Open http://localhost:{}", args.port);

    HttpServer::new(move || {
        App::new()
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
