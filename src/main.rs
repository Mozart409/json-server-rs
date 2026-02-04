#![forbid(unsafe_code)]
#![warn(clippy::pedantic, clippy::cargo)]
#![allow(clippy::multiple_crate_versions)]
#![allow(clippy::unused_async, clippy::unnecessary_wraps)]

use axum::extract::State;
use axum::{
    Json, Router, extract::Path, http::StatusCode, response::Html, response::IntoResponse,
    routing::get,
};
use clap::Parser;
use color_eyre::eyre::{self, Context, ContextCompat};
use serde::Deserialize;
use serde_json::{Value, json};
use std::fs::{self};
use std::net::SocketAddr;
use std::path::Path as fsPath;
use std::sync::Arc;
use tower_http::{compression::CompressionLayer, trace::TraceLayer};
use tracing::log;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

struct AppState {
    data_dir: String,
    files: Vec<String>,
}

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Port to listen on
    #[arg(short, long, default_value_t = 3333)]
    port: u16,

    /// Path to the folder
    #[arg(short, long, default_value_t = format!("./data"))]
    data_dir: String,
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    color_eyre::install()?;

    let args = Args::parse();

    // get the data_dir from the command line
    let data_dir = fsPath::new(&args.data_dir)
        .to_str()
        .context("Invalid data directory path")?
        .to_string();
    println!("data_dir: {data_dir}");

    // check if the folder exists
    let exists = fs::metadata(&data_dir).is_ok();

    if exists {
        tracing::debug!("data_dir exists: {data_dir}");
    } else {
        tracing::warn!("data_dir does not exist: {data_dir}");
        log::warn!("data_dir does not exist: {data_dir}");
        println!("data_dir does not exist: {data_dir}");
        std::process::exit(1);
    }

    // check if the folder contains .json files
    let files =
        get_json_files(&data_dir).context("Failed to get JSON files from data directory")?;

    if files.is_empty() {
        tracing::warn!("data_dir does not contain any json files");
        log::warn!("data_dir does not contain any json files");
        println!("data_dir does not contain any json files");
        std::process::exit(1);
    } else {
        tracing::debug!("data_dir contains json files: {files:?}");
    }

    // When the data_dir ends with a /, remove it
    let data_dir = if data_dir.ends_with('/') {
        data_dir.replace('/', "")
    } else {
        data_dir
    };

    let shared_state = Arc::new(AppState { data_dir, files });

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "json-server-rs=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // build our application with a route
    let app = Router::new()
        .route("/", get(root))
        .route("/_health_check", get(health_check))
        .route("/api", get(get_apis))
        .route("/api/", get(get_apis))
        .route("/api/{file}", get(get_serve_json))
        .layer(TraceLayer::new_for_http())
        .layer(CompressionLayer::new())
        .with_state(shared_state);

    // add a fallback service for handling routes to unknown paths
    let app = app.fallback(handler_404);

    // run it
    let addr = SocketAddr::from(([127, 0, 0, 1], args.port));
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .context("Failed to bind to address")?;
    println!("listening on http://{addr}");
    tracing::debug!("listening on http://{addr}");

    axum::serve(listener, app)
        .await
        .context("Failed to start server")?;

    Ok(())
}
// basic handler that responds with a static string
async fn root() -> Html<&'static str> {
    Html("<h1>Hello, World!</h1>")
}

async fn health_check() -> impl IntoResponse {
    (StatusCode::OK, "ok")
}

async fn handler_404() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "nothing to see here")
}

async fn get_apis(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    if state.files.is_empty() {
        (
            StatusCode::NOT_FOUND,
            axum::Json(json!({"error": "not found"})),
        )
    } else {
        (StatusCode::OK, axum::Json(json!(state.files)))
    }
}

#[derive(Deserialize)]
struct JsonPathParams {
    file: String,
}

async fn get_serve_json(
    State(state): State<Arc<AppState>>,
    Path(JsonPathParams { file }): Path<JsonPathParams>,
) -> impl IntoResponse {
    // check if the file from the endpoint is in the vector of state.files
    // so we can return a 404 if the file is not found
    if !state.files.contains(&file) {
        return (
            StatusCode::NOT_FOUND,
            axum::Json(json!({"error": "file not found"})),
        );
    }

    let path = format!("{}/{}.json", state.data_dir, file);

    tracing::debug!("path: {}", path);
    match fs::read_to_string(&path).with_context(|| format!("Failed to read file: {path}")) {
        Ok(contents) => match serde_json::from_str::<Value>(&contents) {
            Ok(v) => (StatusCode::OK, Json(v)),
            Err(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()})),
            ),
        },
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "Failed to read file"})),
        ),
    }
}

// create a function that finds all .json files in the directory data
// and returns a vector of the file names
fn get_json_files(data_dir: &str) -> eyre::Result<Vec<String>> {
    let mut json_files = Vec::new();
    let entries =
        fs::read_dir(data_dir).with_context(|| format!("Failed to read directory: {data_dir}"))?;

    for entry in entries {
        let entry = entry.context("Failed to read directory entry")?;
        let path = entry.path();

        if let Some(extension) = path.extension()
            && extension == "json"
        {
            let file_name = path
                .file_stem()
                .context("Failed to get file stem")?
                .to_str()
                .context("Invalid filename encoding")?
                .to_string();
            json_files.push(file_name);
        }
    }
    Ok(json_files)
}
