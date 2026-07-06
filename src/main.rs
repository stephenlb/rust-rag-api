mod clean;
mod hash;
mod database;
use database::{Database};

use tokio;
use tokio::fs;
use anyhow::Result;
use std::sync::Arc;
use serde_json::{Value, json};
use axum::{
    Json,
    extract::{State, Path},
    routing::post,
    middleware,
    Router,
    body::Body,
    http::{header, Request},
    middleware::Next,
    response::Response,
};

struct RAGState {
    database: Database,
}

#[derive(Debug)] 
struct Prompt {
    text: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let db = Database::new();

    let documents: Vec<String> = load_data("data/text.txt").await?;
    for document in documents {
        let _ = db.add_document(&document).await;
    }
    let state = Arc::new(RAGState {
        database: db,
    });
    let app: Router = Router::new()

        // User prompt
        .route("/", post(root))

        // Upload documents
        // TODO chunck the data
        .route("/doc", post(doc))

        //.layer(Extension(state))
        .with_state(state)
        .layer(middleware::from_fn(force_json_content_type));

    let host = "0.0.0.0:3000";
    let listener = tokio::net::TcpListener::bind(host).await.unwrap();
    println!("Starting Server http://{host}");
    axum::serve(listener, app).await.unwrap();
    Ok(())
}

// TODO instread of this, do it in the .sh file that uses our API
async fn load_data(filename: &str) -> Result<Vec<String>> {
    let contents = fs::read_to_string(filename).await?;
    let lines: Vec<String> = contents
        .lines()
        .map(|line| line.to_string())
        .collect();

    Ok(lines)
}

// Query interface for user prompts
async fn root(
    State(state): State<Arc<RAGState>>,
    Json(body): Json<Value>,
) -> Json<Value> {
    let prompt: &str = body["prompt"].as_str().unwrap_or("");
    let docs = state.database.search(prompt, 2).await;
    let _ = dbg!(docs);

    Json(json!({"text":"Hello!!!!"}))
}
async fn doc(
    State(state): State<Arc<RAGState>>,
    Json(body): Json<Value>,
) -> Json<Value> {
    let document: &str = body["document"].as_str().unwrap_or("");
    let _ = state.database.add_document(document).await;

    Json(json!({"text":"Doc loaded successfully!"}))
}

async fn force_json_content_type(mut req: Request<Body>, next: Next) -> Response {
    req.headers_mut().insert(
        header::CONTENT_TYPE,
        header::HeaderValue::from_static("application/json"),
    );

    next.run(req).await
}
