mod database;
use database::{Database};

use tokio;
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

#[derive(Debug)] 
struct RAGState {
    database: Database,
    // turbovec
    // sqlite
    // embedding
}

#[derive(Debug)] 
struct Prompt {
    text: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    // TODO tomorrow Monday
    // TODO sqlite handler module
    // TODO Schema
    // TODO Tuesday
    // TODO embedding impl mod  -> @bonzupii ***Nomic:250m***, Arctic, Granite:30m embedding models
    // TODO turbovec handler
    // TODO LLM handler
    // TODO ✅ sqlite in state
    // TODO ✅ add tests
    // TODO ✅ web server routes
    let db = Database::new();
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

// Query interface for user prompts
async fn root(
    State(state): State<Arc<RAGState>>,
    Json(body): Json<Value>,
) -> Json<Value> {
    let prompt: &str = body["prompt"].as_str().unwrap_or("");
    //println!("posted data");
    //println!("User prompt: {}", prompt);
    let docs = state.database.search(prompt, 2).await;
    dbg!(docs);
    //dbg!(s);
    /*
    let _ = match state.database.search(prompt, 5).await {
        Ok(rows) => rows,
        Err(err) => dbg!(err),
    };
    */

    // Response
    Json(json!({"text":"Hello!!!!"}))
}
async fn doc(
    State(state): State<Arc<RAGState>>,
    Json(body): Json<Value>,
) -> Json<Value> {
    let document: &str = body["document"].as_str().unwrap_or("");
    let _ = state.database.insert(document).await;
    //dbg!(s);
    Json(json!({"text":"Doc loaded successfully!"}))
}

async fn force_json_content_type(mut req: Request<Body>, next: Next) -> Response {
    req.headers_mut().insert(
        header::CONTENT_TYPE,
        header::HeaderValue::from_static("application/json"),
    );
    next.run(req).await
}
