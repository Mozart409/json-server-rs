#![forbid(unsafe_code)]
#![warn(clippy::pedantic, clippy::cargo)]
#![allow(clippy::unused_async)]

use axum::extract::State;
use axum::Error;
use axum::{
    extract::Path, http::StatusCode, response::Html, response::IntoResponse, routing::get, Json,
    Router,
};
use clap::{arg, command, Parser};
use serde::Deserialize;
use serde_json::{json, Value};
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
    #[arg(short, long, default_value_t = 3000)]
    port: u16,

    /// Path to the folder
    #[arg(short, long, default_value_t = format!("./data"))]
    data_dir: String,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    // get the data_dir from the command line
    let data_dir = fsPath::new(&args.data_dir).to_str().unwrap().to_string();
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
    let files = get_json_files(data_dir.clone()).expect("Can't get json files");

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
        .route("/api/:file", get(get_serve_json))
        .layer(TraceLayer::new_for_http())
        .layer(CompressionLayer::new())
        .with_state(shared_state);

    // add a fallback service for handling routes to unknown paths
    let app = app.fallback(handler_404);

    // run it
    let addr = SocketAddr::from(([127, 0, 0, 1], args.port));
    println!("listening on http://{addr}");
    tracing::debug!("listening on http://{}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
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
    // check if the let file is in the vector of state.files
    if !state.files.contains(&file) {
        return (
            StatusCode::NOT_FOUND,
            axum::Json(json!({"error": "file not found"})),
        );
    }

    let path = format!("{}/{}.json", state.data_dir, file);

    tracing::debug!("path: {}", path);
    let str = fs::read_to_string(path).expect("Unable to read file");
    match serde_json::from_str::<Value>(&str) {
        Ok(v) => (StatusCode::OK, Json(v)),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        ),
    }
}

// create a function that finds all .json files in the directory data
// and returns a vector of the file names
fn get_json_files(data_dir: String) -> Result<Vec<String>, Error> {
    let mut json_files = Vec::new();
    for entry in fs::read_dir(data_dir).expect("read_dir call failed") {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.extension().unwrap() == "json" {
            // trim of the .json extension
            let file_name = path.file_name().unwrap().to_str().unwrap().to_string();
            let file_name = file_name.replace(".json", "");
            json_files.push(file_name);
        }
    }
    Ok(json_files)
}
