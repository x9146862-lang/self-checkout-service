use axum::{
    extract::{rejection::JsonRejection, FromRequest, Request},
    http::StatusCode,
    middleware::{self, Next},
    response::{IntoResponse, Json, Response},
    routing::{get, post},
    Router,
};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::LazyLock;
use std::time::Instant;
use tower_http::cors::CorsLayer;
use tracing::{info, warn};

/// `GET /health` 的响应体。
#[derive(Debug, Serialize)]
struct HealthResponse {
    /// 服务状态
    status: String,
}

/// 商品信息。
#[derive(Debug, Clone, Serialize)]
struct Product {
    /// 商品名称
    name: String,
    /// 商品价格
    price: f64,
}

/// POST /scan 的请求体。
#[derive(Debug, Deserialize)]
struct ScanRequest {
    /// 商品条码
    barcode: String,
}

/// 统一错误响应结构：所有错误接口都返回 `{"error": "..."}` 格式。
#[derive(Debug, Serialize)]
struct ApiError {
    /// 错误描述
    error: String,
    /// 该错误对应的 HTTP 状态码（不序列化进响应体）
    #[serde(skip)]
    status: StatusCode,
}

impl ApiError {
    /// 构造 400 Bad Request 错误
    fn bad_request(msg: impl Into<String>) -> Self {
        Self {
            error: msg.into(),
            status: StatusCode::BAD_REQUEST,
        }
    }

    /// 构造 404 Not Found 错误
    fn not_found(msg: impl Into<String>) -> Self {
        Self {
            error: msg.into(),
            status: StatusCode::NOT_FOUND,
        }
    }
}

/// 让 ApiError 可以直接作为处理器返回值：以对应状态码返回 JSON。
impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (self.status, Json(self)).into_response()
    }
}

/// 自定义 JSON 提取器：把 axum 默认的反序列化失败统一转换成 [`ApiError`]，
/// 保证所有错误都返回统一的 `{"error": "..."}` JSON 格式，并避免向客户端
/// 暴露 serde 的内部实现细节。
struct ValidJson<T>(T);

impl<S, T> FromRequest<S> for ValidJson<T>
where
    T: DeserializeOwned,
    S: Send + Sync,
    Json<T>: FromRequest<S, Rejection = JsonRejection>,
{
    type Rejection = ApiError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        match Json::<T>::from_request(req, state).await {
            Ok(Json(value)) => Ok(ValidJson(value)),
            Err(rejection) => {
                // 详细原因只写进日志，返回给客户端的是一句统一、友好的提示
                warn!(
                    error = %rejection.body_text(),
                    status = rejection.status().as_u16(),
                    "JSON 请求体解析失败"
                );
                let message = match rejection {
                    JsonRejection::MissingJsonContentType(_) => {
                        "缺少 Content-Type: application/json 请求头"
                    }
                    JsonRejection::JsonSyntaxError(_) => "请求体不是有效的 JSON",
                    JsonRejection::JsonDataError(_) => "请求体 JSON 字段类型不正确",
                    _ => "请求体格式错误",
                };
                Err(ApiError::bad_request(message))
            }
        }
    }
}

/// 写死在内存里的商品目录（条码 -> 商品）。
static CATALOG: LazyLock<HashMap<String, Product>> = LazyLock::new(|| {
    HashMap::from([
        (
            "6901234567890".to_string(),
            Product {
                name: "可口可乐".to_string(),
                price: 3.5,
            },
        ),
        (
            "6901234567891".to_string(),
            Product {
                name: "农夫山泉".to_string(),
                price: 2.0,
            },
        ),
        (
            "6901234567892".to_string(),
            Product {
                name: "乐事薯片".to_string(),
                price: 6.8,
            },
        ),
        (
            "6901234567893".to_string(),
            Product {
                name: "康师傅方便面".to_string(),
                price: 4.5,
            },
        ),
        (
            "6901234567894".to_string(),
            Product {
                name: "奥利奥饼干".to_string(),
                price: 8.9,
            },
        ),
        (
            "6901234567895".to_string(),
            Product {
                name: "蒙牛纯牛奶".to_string(),
                price: 5.2,
            },
        ),
    ])
});

/// GET /health —— 健康检查。
async fn health_handler() -> Json<HealthResponse> {
    info!(path = "/health", "健康检查");
    Json(HealthResponse {
        status: "ok".to_string(),
    })
}

/// POST /scan —— 根据条码查找商品，找不到返回 404。
async fn scan_handler(
    ValidJson(payload): ValidJson<ScanRequest>,
) -> Result<Json<Product>, ApiError> {
    match CATALOG.get(&payload.barcode) {
        Some(product) => {
            info!(barcode = %payload.barcode, "扫描到商品");
            Ok(Json(product.clone()))
        }
        None => {
            info!(barcode = %payload.barcode, "商品未找到");
            Err(ApiError::not_found("未找到该条码对应的商品"))
        }
    }
}

/// 日志中间件：记录每个请求的方法、路径、状态码与处理耗时。
async fn logging_middleware(req: Request, next: Next) -> Response {
    let method = req.method().clone();
    let path = req.uri().path().to_string();
    let start = Instant::now();
    let response = next.run(req).await;
    let elapsed = start.elapsed();
    info!(
        method = %method,
        path = %path,
        status = response.status().as_u16(),
        elapsed_ms = elapsed.as_millis() as u64,
        "HTTP 请求处理完成"
    );
    response
}

/// 构建应用路由（含 CORS 与日志中间件），供 `main` 复用。
pub fn app() -> Router {
    Router::new()
        .route("/health", get(health_handler))
        .route("/scan", post(scan_handler))
        // 开发环境允许所有来源的跨域请求
        .layer(CorsLayer::permissive())
        .layer(middleware::from_fn(logging_middleware))
}
