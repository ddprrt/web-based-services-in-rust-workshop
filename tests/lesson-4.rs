use axum::{
    body::Body,
    http::{Request, StatusCode},
};

use tower::Service;
use webservice_rust_workshop::{router, SharedState}; // for `call`

#[ignore]
#[tokio::test]
async fn no_auth() {
    let state = SharedState::default();
    let mut app = router(&state);

    // Add something
    let response = app
        .call(
            Request::builder()
                .uri("/kv/test")
                .method("POST")
                .body("Hello World".into())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Check if it's there
    let response = app
        .call(
            Request::builder()
                .uri("/kv/test")
                .method("GET")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    assert_eq!(&body[..], b"Hello World");

    let response = app
        .call(
            Request::builder()
                .uri("/admin/kv")
                .method("DELETE")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[ignore]
#[tokio::test]
async fn delete_entries() {
    let state = SharedState::default();
    let mut app = router(&state);

    // Add something
    let response = app
        .call(
            Request::builder()
                .uri("/kv/test")
                .method("POST")
                .body("Hello World".into())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Check if it's there
    let response = app
        .call(
            Request::builder()
                .uri("/kv/test")
                .method("GET")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    assert_eq!(&body[..], b"Hello World");

    let response = app
        .call(
            Request::builder()
                .uri("/admin/kv")
                .method("DELETE")
                .header("Authorization", "Bearer secret")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let response = app
        .call(
            Request::builder()
                .uri("/kv/test")
                .method("GET")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[ignore]
#[tokio::test]
async fn delete_keys() {
    let state = SharedState::default();
    let mut app = router(&state);

    // Add something
    let response = app
        .call(
            Request::builder()
                .uri("/kv/test")
                .method("POST")
                .body("Hello World".into())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Check if it's there
    let response = app
        .call(
            Request::builder()
                .uri("/kv/test")
                .method("GET")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    assert_eq!(&body[..], b"Hello World");

    let response = app
        .call(
            Request::builder()
                .uri("/admin/kv/test")
                .method("DELETE")
                .header("Authorization", "Bearer secret")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let response = app
        .call(
            Request::builder()
                .uri("/kv/test")
                .method("GET")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
