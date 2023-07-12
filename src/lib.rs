use std::{
    collections::HashMap,
    sync::{Arc, PoisonError, RwLock},
};

use axum::{
    body::Bytes,
    extract::{DefaultBodyLimit, Path, Query, State},
    handler::Handler,
    response::{Html, IntoResponse},
    routing::get,
    Router,
};

use hyper::{Body, Request, StatusCode};
use serde::Deserialize;
use tower::{Layer, Service, ServiceBuilder};

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
        .route(
            "/kv/:key",
            get(handler_kv_get).post_service(
                ServiceBuilder::new()
                    .layer(DefaultBodyLimit::disable())
                    .service(handler_kv_post.with_state(Arc::clone(state))),
            ),
        )
        .layer(LoggerMiddleware::new())
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

async fn handler_kv_post(
    Path(key): Path<String>,
    State(state): State<SharedState>,
    bytes: Bytes,
) -> Result<&'static str, (StatusCode, &'static str)> {
    let mut state = match state.write() {
        Ok(state) => state,
        Err(err) => return Err((StatusCode::INTERNAL_SERVER_ERROR, "Database corrupted")),
    };

    state.db.insert(key, bytes);

    Ok("Inserted key")
}

struct DbError(StatusCode, &'static str);

impl<T> From<PoisonError<T>> for DbError {
    fn from(_: PoisonError<T>) -> Self {
        Self(StatusCode::INTERNAL_SERVER_ERROR, "Database corrupted")
    }
}

impl IntoResponse for DbError {
    fn into_response(self) -> axum::response::Response {
        (self.0, self.1).into_response()
    }
}

struct Point<T, U> {
    x: T,
    y: U,
}

impl<T> Point<T, T> {
    fn new(x: T, y: T) -> Self {
        Self { x, y }
    }
}

impl Point<i32, i32> {
    fn sum(&self) -> i32 {
        self.x + self.y
    }
}

fn foo() {
    let int_point = Point::new(1, 2);
    int_point.sum();
    let str_point = Point::new("a", "b");
}

async fn handler_kv_get(
    Path(key): Path<String>,
    State(state): State<SharedState>,
) -> Result<Bytes, DbError> {
    match state.read()?.db.get(&key) {
        Some(val) => Ok(val.clone()),
        None => Err(DbError(StatusCode::NOT_FOUND, "Key not found")),
    }
}

#[derive(Clone, Copy)]
struct Logger<S> {
    inner: S,
}

impl<S> Logger<S> {
    fn new(inner: S) -> Self {
        Self { inner }
    }
}

impl<IS> Service<Request<Body>> for Logger<IS>
where
    IS: Service<Request<Body>>,
{
    type Response = IS::Response;
    type Error = IS::Error;
    type Future = IS::Future;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        println!("{} {}", req.method(), req.uri());
        self.inner.call(req)
    }
}

struct LoggerMiddleware;

impl LoggerMiddleware {
    fn new() -> Self {
        Self
    }
}

impl<S> Layer<S> for LoggerMiddleware {
    type Service = Logger<S>;

    fn layer(&self, inner: S) -> Self::Service {
        Logger::new(inner)
    }
}
