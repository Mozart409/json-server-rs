use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Html,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::{fs, net::SocketAddr};
use tower::ServiceBuilder;

use tower_http::compression::CompressionLayer;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "json-server-rs=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(root))
        // `POST /users` goes to `create_user`
        .route("/users", post(create_user))
        // `GET /api` goes to `get_json_files`
        .route("/api", get(get_apis))
        .route("/api/", get(get_apis))
        .route("/api/:file", get(get_serve_json))
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(CompressionLayer::new()),
        );

    // add a fallback service for handling routes to unknown paths
    let app = app.fallback(handler_404);

    // run it
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("listening on http://{}", addr);
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

async fn handler_404() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "nothing to see here")
}

async fn create_user(
    // this argument tells axum to parse the request body
    // as JSON into a `CreateUser` type
    Json(payload): Json<CreateUser>,
) -> impl IntoResponse {
    // insert your application logic here
    let user = User {
        id: 1337,
        username: payload.username,
    };

    // this will be converted into a JSON response
    // with a status code of `201 Created`
    (StatusCode::CREATED, Json(user))
}

// the input to our `create_user` handler
#[derive(Deserialize)]
struct CreateUser {
    username: String,
}

// the output to our `create_user` handler
#[derive(Serialize)]
struct User {
    id: u64,
    username: String,
}

async fn get_apis() -> impl IntoResponse {
    let json_files = get_json_files();
    let json_files = Json(json_files);
    (StatusCode::OK, json_files)
}

#[derive(Deserialize)]
struct JsonPathParams {
    file: String,
}

async fn get_serve_json(Path(JsonPathParams { file }): Path<JsonPathParams>) -> impl IntoResponse {
    // load the json file and return it
    let path = format!("data/{}.json", file);
    let str = fs::read_to_string(path).unwrap();
    // convert the string to a json object  and return it
    let json: serde_json::Value = serde_json::from_str(&str).unwrap();
    (StatusCode::OK, Json(json))
}

// create a function that finds all .json files in the directory data
// and returns a vector of the file names
fn get_json_files() -> Vec<String> {
    let mut json_files = Vec::new();
    for entry in fs::read_dir("data").unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.extension().unwrap() == "json" {
            // trim of the .json extension
            let file_name = path.file_name().unwrap().to_str().unwrap().to_string();
            let file_name = file_name.replace(".json", "");
            json_files.push(file_name);
        }
    }
    json_files
}
