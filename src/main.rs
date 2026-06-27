use tokio;
use anyhow::Result;

use std::sync::Arc;

use axum::{
    Json,
    extract::{Extension, Path},
    routing::post,
    Router,
};

struct RAGState {
    // turbovec
    // sqlite
    // embedding
}

struct Prompt {
    _text: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    // TODO embedding impl mod  -> @bonzupii ***Nomic:250m***, Arctic, Granite:30m embedding models
    // TODO turbovec handler
    // TODO sqlite handler
    // TODO llm handler
    // TODO add tests
    let state = Arc::new(RAGState{});
    let _app: Router = Router::new()
        // User Prompt
        .route("/", post(root)
        .layer(Extension(state)));

        /*
        // Add Document to the RAG DBs
        .route("/add", post({
            let _state = Arc::clone(&state);
            //move |body| root(body, state)
        }));
        */

    println!("Hello, world!");
    Ok(())
}

async fn root(
    Extension(state): Extension<Arc<RAGState>>,
    //Json(_body) Json<Prompt>,
) {
}
// TODO /add doc
