use tokio;
use anyhow::Result;
use std::sync::Arc;
use serde_json::{Value, json};
use axum::{
    Json,
    extract::{Extension, Path},
    routing::post,
    middleware,
    Router,
    body::Body,
    http::{header, Request},
    middleware::Next,
    response::Response,
};

async fn force_json_content_type(mut req: Request<Body>, next: Next) -> Response {
    req.headers_mut().insert(
        header::CONTENT_TYPE,
        header::HeaderValue::from_static("application/json"),
    );
    next.run(req).await
}

#[derive(Debug)] 
struct RAGState {
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
    // TODO embedding impl mod  -> @bonzupii ***Nomic:250m***, Arctic, Granite:30m embedding models
    // TODO turbovec handler
    // TODO sqlite handler
    // TODO llm handler
    // TODO add tests
    let state = Arc::new(RAGState{});
    let app: Router = Router::new()
        // User Prompt
        .route("/", post(root)
        .layer(Extension(state)))
        .layer(middleware::from_fn(force_json_content_type));

        /*
        // Add Document to the RAG DBs
        .route("/add", post({
            let _state = Arc::clone(&state);
            //move |body| root(body, state)
        }));
        */

    let host = "0.0.0.0:3000";
    let listener = tokio::net::TcpListener::bind(host).await.unwrap();
    println!("Starting Server http://{host}");
    axum::serve(listener, app).await.unwrap();
    Ok(())
}

async fn root(
    Extension(state): Extension<Arc<RAGState>>,
    Json(body): Json<Value>,
) -> Json<Value> {
    // TODO --- This is today's main goal
    println!("posted data");
    println!("{}", body);

    Json(json!({"text":"Hello!!!!"}))
}
// TODO /add doc
