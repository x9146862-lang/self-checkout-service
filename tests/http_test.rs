use axum::body::Body;
use axum::http::{header, Request, StatusCode};
use axum::Router;
use http_body_util::BodyExt;
use self_checkout_service::app;
use tower::ServiceExt;

/// 发送请求并解析响应为 `(状态码, JSON body)`。
async fn send(req: Request<Body>) -> (StatusCode, serde_json::Value) {
    let router: Router = app();
    let response = router.oneshot(req).await.unwrap();
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let body = serde_json::from_slice(&bytes).unwrap_or(serde_json::Value::Null);
    (status, body)
}

#[tokio::test]
async fn test_health() {
    let req = Request::builder()
        .uri("/health")
        .body(Body::empty())
        .unwrap();
    let (status, body) = send(req).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, serde_json::json!({"status": "ok"}));
}

#[tokio::test]
async fn test_scan_found() {
    let req = Request::builder()
        .method("POST")
        .uri("/scan")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(r#"{"barcode":"6901234567892"}"#))
        .unwrap();
    let (status, body) = send(req).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, serde_json::json!({"name": "乐事薯片", "price": 6.8}));
}

#[tokio::test]
async fn test_scan_not_found() {
    let req = Request::builder()
        .method("POST")
        .uri("/scan")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(r#"{"barcode":"0000000000000"}"#))
        .unwrap();
    let (status, body) = send(req).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body, serde_json::json!({"error": "未找到该条码对应的商品"}));
}

#[tokio::test]
async fn test_scan_invalid_type() {
    let req = Request::builder()
        .method("POST")
        .uri("/scan")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(r#"{"barcode":123}"#))
        .unwrap();
    let (status, body) = send(req).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(
        body,
        serde_json::json!({"error": "请求体 JSON 字段类型不正确"})
    );
}

#[tokio::test]
async fn test_scan_invalid_json() {
    let req = Request::builder()
        .method("POST")
        .uri("/scan")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(""))
        .unwrap();
    let (status, body) = send(req).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body, serde_json::json!({"error": "请求体不是有效的 JSON"}));
}

#[tokio::test]
async fn test_scan_missing_content_type() {
    let req = Request::builder()
        .method("POST")
        .uri("/scan")
        .body(Body::from(r#"{"barcode":"6901234567892"}"#))
        .unwrap();
    let (status, body) = send(req).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(
        body,
        serde_json::json!({"error": "缺少 Content-Type: application/json 请求头"})
    );
}
