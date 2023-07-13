use std::{
    collections::HashMap,
    sync::{Arc, PoisonError, RwLock},
    time::Duration,
};

use axum::{
    body::Bytes,
    error_handling::HandleErrorLayer,
    extract::{DefaultBodyLimit, Path, Query, State},
    handler::Handler,
    response::{Html, IntoResponse},
    routing::{delete, get},
    BoxError, Router,
};

use hyper::{Body, Request, StatusCode};
use serde::Deserialize;
use tower::{timeout::TimeoutLayer, Layer, Service, ServiceBuilder};
use tower_http::auth::RequireAuthorizationLayer;

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
        .nest("/admin", admin_routes(state))
        .layer(
            ServiceBuilder::new()
                .layer(HandleErrorLayer::new(handle_error))
                .layer(TimeoutLayer::new(Duration::from_secs(5))),
        )
        .layer(LoggerMiddleware::new())
}

fn admin_routes(state: &SharedState) -> Router<SharedState> {
    Router::with_state(Arc::clone(state))
        .route("/kv", delete(admin_handle_delete))
        .route("/kv/:key", delete(admin_handle_delete_key))
        .layer(RequireAuthorizationLayer::bearer("secret"))
}

async fn admin_handle_delete_key(
    Path(key): Path<String>,
    State(state): State<SharedState>,
) -> (StatusCode, &'static str) {
    let mut state = match state.write() {
        Ok(state) => state,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Database corrupted"),
    };

    let _ = state.db.remove(&key);

    (StatusCode::OK, "Deleted entry")
}

async fn admin_handle_delete(State(state): State<SharedState>) -> (StatusCode, &'static str) {
    let mut state = match state.write() {
        Ok(state) => state,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Database corrupted"),
    };

    state.db.clear();

    (StatusCode::OK, "Deleted all entries")
}

async fn handle_error(err: BoxError) -> (StatusCode, &'static str) {
    if err.is::<tower::timeout::error::Elapsed>() {
        eprintln!("Request timed out: {}", err);
        return (StatusCode::REQUEST_TIMEOUT, "Request timed out");
    } else if err.is::<std::io::Error>() {
        eprintln!("IO Error: {}", err);
        return (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error");
    }
    (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error")
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
    //tokio::time::sleep(Duration::from_secs(6)).await;
    "<h1>Hello Axum</h1>"
}

async fn handler_kv_post(
    Path(key): Path<String>,
    State(state): State<SharedState>,
    bytes: Bytes,
) -> Result<&'static str, (StatusCode, &'static str)> {
    let mut state = match state.write() {
        Ok(state) => state,
        Err(_) => return Err((StatusCode::INTERNAL_SERVER_ERROR, "Database corrupted")),
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

fn _foo() {
    let int_point = Point::new(1, 2);
    int_point.sum();
    let _str_point = Point::new("a", "b");
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
