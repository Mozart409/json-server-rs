#![forbid(unsafe_code)]
#![warn(clippy::pedantic, clippy::cargo)]
#![allow(clippy::unused_async, clippy::unnecessary_wraps)]

use std::{fs, sync::Arc};

use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
};
use axum_test::TestServer;
use serde::Deserialize;
use serde_json::{Value, json};
use tempfile::TempDir;

struct AppState {
    data_dir: String,
    files: Vec<String>,
}

#[derive(Deserialize)]
struct JsonPathParams {
    file: String,
}

async fn get_apis(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    if state.files.is_empty() {
        (StatusCode::NOT_FOUND, Json(json!({"error": "not found"})))
    } else {
        (StatusCode::OK, Json(json!(state.files)))
    }
}

async fn get_serve_json(
    State(state): State<Arc<AppState>>,
    Path(JsonPathParams { file }): Path<JsonPathParams>,
) -> impl IntoResponse {
    if !state.files.contains(&file) {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "file not found"})),
        );
    }

    let path = format!("{}/{}.json", state.data_dir, file);

    match fs::read_to_string(&path) {
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

async fn health_check() -> impl IntoResponse {
    (StatusCode::OK, "ok")
}

fn create_test_app(data_dir: &str, files: Vec<String>) -> Router {
    let state = Arc::new(AppState {
        data_dir: data_dir.to_string(),
        files,
    });

    Router::new()
        .route("/_health_check", get(health_check))
        .route("/api", get(get_apis))
        .route("/api/{file}", get(get_serve_json))
        .with_state(state)
}

fn setup_test_data() -> (TempDir, String, Vec<String>) {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let data_dir = temp_dir.path().to_str().unwrap().to_string();

    fs::write(
        format!("{data_dir}/articles.json"),
        include_str!("../data/articles.json"),
    )
    .expect("Failed to write articles.json");

    fs::write(
        format!("{data_dir}/starwars.json"),
        include_str!("../data/starwars.json"),
    )
    .expect("Failed to write starwars.json");

    fs::write(
        format!("{data_dir}/nested.json"),
        r#"{
            "data": {
                "nested": {
                    "value": 42
                },
                "array": [1, 2, 3]
            }
        }"#,
    )
    .expect("Failed to write nested.json");

    let files = vec![
        "articles".to_string(),
        "starwars".to_string(),
        "nested".to_string(),
    ];

    (temp_dir, data_dir, files)
}

#[tokio::test]
async fn test_health_check() {
    let (_temp_dir, data_dir, files) = setup_test_data();
    let app = create_test_app(&data_dir, files);
    let server = TestServer::new(app).unwrap();

    let response = server.get("/_health_check").await;
    response.assert_status_ok();
    response.assert_text("ok");
}

#[tokio::test]
async fn test_list_apis() {
    let (_temp_dir, data_dir, files) = setup_test_data();
    let app = create_test_app(&data_dir, files.clone());
    let server = TestServer::new(app).unwrap();

    let response = server.get("/api").await;
    response.assert_status_ok();

    let json: Value = response.json();
    assert!(json.is_array());
    let array = json.as_array().unwrap();
    assert_eq!(array.len(), 3);
    assert!(array.contains(&Value::String("articles".to_string())));
    assert!(array.contains(&Value::String("starwars".to_string())));
    assert!(array.contains(&Value::String("nested".to_string())));
}

#[tokio::test]
async fn test_list_apis_empty() {
    let (_temp_dir, data_dir, _files) = setup_test_data();
    let app = create_test_app(&data_dir, vec![]);
    let server = TestServer::new(app).unwrap();

    let response = server.get("/api").await;
    response.assert_status_not_found();

    let json: Value = response.json();
    assert_eq!(json, json!({"error": "not found"}));
}

#[tokio::test]
async fn test_serve_articles_json() {
    let (_temp_dir, data_dir, files) = setup_test_data();
    let app = create_test_app(&data_dir, files);
    let server = TestServer::new(app).unwrap();

    let response = server.get("/api/articles").await;
    response.assert_status_ok();

    let json: Value = response.json();
    assert!(json.is_object());
    assert!(json.get("data").is_some());
    assert!(json.get("links").is_some());
    assert!(json.get("included").is_some());
}

#[tokio::test]
async fn test_serve_starwars_json() {
    let (_temp_dir, data_dir, files) = setup_test_data();
    let app = create_test_app(&data_dir, files);
    let server = TestServer::new(app).unwrap();

    let response = server.get("/api/starwars").await;
    response.assert_status_ok();

    let json: Value = response.json();
    assert!(json.is_object());
    assert_eq!(
        json.get("name"),
        Some(&Value::String("Luke Skywalker".to_string()))
    );
    assert_eq!(json.get("height"), Some(&Value::String("172".to_string())));
    assert_eq!(json.get("mass"), Some(&Value::String("77".to_string())));
}

#[tokio::test]
async fn test_serve_nested_json() {
    let (_temp_dir, data_dir, files) = setup_test_data();
    let app = create_test_app(&data_dir, files);
    let server = TestServer::new(app).unwrap();

    let response = server.get("/api/nested").await;
    response.assert_status_ok();

    let json: Value = response.json();
    assert!(json.is_object());

    let data = json.get("data").unwrap();
    let nested = data.get("nested").unwrap();
    assert_eq!(nested.get("value"), Some(&Value::Number(42.into())));

    let array = data.get("array").unwrap();
    assert_eq!(
        array,
        &Value::Array(vec![
            Value::Number(1.into()),
            Value::Number(2.into()),
            Value::Number(3.into())
        ])
    );
}

#[tokio::test]
async fn test_serve_nonexistent_file() {
    let (_temp_dir, data_dir, files) = setup_test_data();
    let app = create_test_app(&data_dir, files);
    let server = TestServer::new(app).unwrap();

    let response = server.get("/api/nonexistent").await;
    response.assert_status_not_found();

    let json: Value = response.json();
    assert_eq!(json, json!({"error": "file not found"}));
}

#[tokio::test]
async fn test_invalid_json_file() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let data_dir = temp_dir.path().to_str().unwrap().to_string();

    fs::write(format!("{data_dir}/invalid.json"), "{ invalid json }")
        .expect("Failed to write invalid.json");

    let files = vec!["invalid".to_string()];
    let app = create_test_app(&data_dir, files);
    let server = TestServer::new(app).unwrap();

    let response = server.get("/api/invalid").await;
    response.assert_status_internal_server_error();

    let json: Value = response.json();
    assert!(json.get("error").is_some());
    assert!(json.get("error").unwrap().as_str().is_some());
}

#[tokio::test]
async fn test_large_json_file() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let data_dir = temp_dir.path().to_str().unwrap().to_string();

    let numbers: Vec<i32> = (1..=1000).collect();
    let large_json = serde_json::to_string(&json!({"numbers": numbers}))
        .expect("Failed to serialize large json");

    fs::write(format!("{data_dir}/large.json"), large_json).expect("Failed to write large.json");

    let files = vec!["large".to_string()];
    let app = create_test_app(&data_dir, files);
    let server = TestServer::new(app).unwrap();

    let response = server.get("/api/large").await;
    response.assert_status_ok();

    let json: Value = response.json();
    let numbers_array = json.get("numbers").unwrap().as_array().unwrap();
    assert_eq!(numbers_array.len(), 1000);
    assert_eq!(numbers_array[0], Value::Number(1.into()));
    assert_eq!(numbers_array[999], Value::Number(1000.into()));
}

#[tokio::test]
async fn test_unicode_in_json() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let data_dir = temp_dir.path().to_str().unwrap().to_string();

    let unicode_data = json!({
        "name": "测试",
        "emoji": "🦀",
        "mixed": "Hello 世界! 👋",
        "nested": {
            "unicode": "Привет"
        }
    });

    fs::write(
        format!("{data_dir}/unicode.json"),
        serde_json::to_string(&unicode_data).unwrap(),
    )
    .expect("Failed to write unicode.json");

    let files = vec!["unicode".to_string()];
    let app = create_test_app(&data_dir, files);
    let server = TestServer::new(app).unwrap();

    let response = server.get("/api/unicode").await;
    response.assert_status_ok();

    let json: Value = response.json();
    assert_eq!(json["name"], Value::String("测试".to_string()));
    assert_eq!(json["emoji"], Value::String("🦀".to_string()));
    assert_eq!(json["mixed"], Value::String("Hello 世界! 👋".to_string()));
    assert_eq!(
        json["nested"]["unicode"],
        Value::String("Привет".to_string())
    );
}
