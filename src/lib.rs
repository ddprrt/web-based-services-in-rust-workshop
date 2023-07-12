use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use axum::{
    body::Bytes,
    extract::{Query, State},
    response::Html,
    routing::get,
    Router,
};

use serde::Deserialize;

/// Custom type for a shared state
pub type SharedState = Arc<RwLock<AppState>>;
#[derive(Default)]
pub struct AppState {
    db: HashMap<String, Bytes>,
}

pub fn router(state: &SharedState) -> Router<SharedState> {
    Router::with_state(Arc::clone(state))
        .route("/", get(hello_axum))
        .route("/hello", get(handler_hello))
}

#[derive(Deserialize)]
struct VisitorParams {
    name: Option<String>,
}

async fn handler_hello(Query(visitor_params): Query<VisitorParams>) -> Html<String> {
    Html(format!(
        "<h1>Hello {}</h1>",
        visitor_params.name.unwrap_or("Unknown Visitor".to_owned())
    ))
}

async fn hello_axum() -> &'static str {
    "<h1>Hello Axum</h1>"
}

async fn handler_kv_get(State(state): State<SharedState>) {}
