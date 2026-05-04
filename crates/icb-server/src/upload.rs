use actix_multipart::Multipart;
use actix_web::{web, HttpResponse};
use futures_util::StreamExt;
use icb_graph::graph::CodePropertyGraph;
use std::io::Cursor;
use std::sync::Mutex;
use tempfile::tempdir;
use zip::ZipArchive;

use crate::graph_builder;

pub async fn handle_upload(
    data: web::Data<Mutex<CodePropertyGraph>>,
    mut payload: Multipart,
) -> HttpResponse {
    let tmp = match tempdir() {
        Ok(dir) => dir,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };

    let mut zip_bytes: Vec<u8> = Vec::new();

    while let Some(item) = payload.next().await {
        let mut field = match item {
            Ok(f) => f,
            Err(e) => return HttpResponse::BadRequest().body(e.to_string()),
        };
        while let Some(chunk) = field.next().await {
            let chunk = match chunk {
                Ok(c) => c,
                Err(_) => break,
            };
            zip_bytes.extend_from_slice(&chunk);
        }
    }

    if zip_bytes.is_empty() {
        return HttpResponse::BadRequest().body("No ZIP file received");
    }

    let reader = Cursor::new(zip_bytes);
    let mut archive = match ZipArchive::new(reader) {
        Ok(a) => a,
        Err(e) => return HttpResponse::BadRequest().body(format!("Invalid ZIP: {}", e)),
    };

    for i in 0..archive.len() {
        let mut file = match archive.by_index(i) {
            Ok(f) => f,
            Err(_) => continue,
        };

        let name = file.name().to_string();
        let path = tmp.path().join(&name);

        if file.is_dir() {
            std::fs::create_dir_all(&path).ok();
        } else {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent).ok();
            }
            let mut out = match std::fs::File::create(&path) {
                Ok(f) => f,
                Err(_) => continue,
            };
            std::io::copy(&mut file, &mut out).ok();
        }
    }

    let graph_result = graph_builder::build_or_load_graph(tmp.path(), "auto", None);

    drop(tmp);

    match graph_result {
        Ok(new_graph) => {
            let nodes = new_graph.graph.node_count();
            let edges = new_graph.graph.edge_count();
            if let Ok(mut locked) = data.lock() {
                *locked = new_graph;
            }
            HttpResponse::Ok().json(serde_json::json!({
                "status": "ok",
                "nodes": nodes,
                "edges": edges,
            }))
        }
        Err(e) => HttpResponse::BadRequest().body(e.to_string()),
    }
}
