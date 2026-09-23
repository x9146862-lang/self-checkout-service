use self_checkout_service::app;

#[tokio::main]
async fn main() {
    // 初始化 tracing：默认 INFO 级别输出到控制台，可用 RUST_LOG 覆盖
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();

    // 监听地址可通过环境变量 ADDR 覆盖
    let addr = std::env::var("ADDR").unwrap_or_else(|_| "127.0.0.1:3000".to_string());

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("端口绑定失败");
    tracing::info!(addr = %addr, "self-checkout 服务已启动");

    axum::serve(listener, app()).await.expect("服务运行失败");
}
