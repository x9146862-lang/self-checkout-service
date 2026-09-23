# self-checkout-service

> 一个基于 **axum** 的简易自助结账 HTTP 服务（练手项目）。
> 商品数据写死在内存中的 `HashMap` 里，目前**未接入数据库**。

提供的接口：

- `GET /health` —— 健康检查
- `POST /scan` —— 根据商品条码查找商品

内置 `tracing` 结构化日志、`tower-http` CORS 支持，以及统一的 JSON 错误响应格式。

## 项目结构

```
.
├── Cargo.toml        # 项目配置（axum / tokio / serde / tracing / tower-http）
├── src/
│   ├── main.rs       # 程序入口：初始化 tracing、绑定端口并启动服务
│   └── lib.rs        # 库入口：路由、处理器、商品目录、ValidJson 提取器、日志中间件、app() 工厂
└── .gitignore
```

## 环境要求

- Rust 工具链（rustc + cargo），推荐通过 [rustup](https://rustup.rs/) 安装

## 运行

```bash
# 编译并启动服务（默认监听 http://127.0.0.1:3000）
cargo run
```

监听地址可通过环境变量 `ADDR` 覆盖（例如 `ADDR=0.0.0.0:8080 cargo run`）。

> **CORS 说明**：当前配置允许**所有来源**的跨域请求（`CorsLayer::permissive()`），仅适用于开发环境；生产部署前请收紧为具体来源白名单。

## 接口

### GET /health

健康检查，用于探活。

```bash
curl http://127.0.0.1:3000/health
# {"status":"ok"}
```

**成功响应**（`200 OK`）

```json
{"status": "ok"}
```

### POST /scan

根据商品条码查找商品。

**请求体**（`application/json`）

| 字段 | 类型 | 说明 |
| ---- | ---- | ---- |
| `barcode` | 字符串 | 商品条码 |

```bash
curl -X POST http://127.0.0.1:3000/scan \
  -H 'Content-Type: application/json' \
  -d '{"barcode": "6901234567892"}'
```

**成功响应**（`200 OK`）

```json
{"name": "乐事薯片", "price": 6.8}
```

**商品目录**（写死在内存中）

| 条码 | 商品名 | 价格 |
| ---- | ---- | ---- |
| 6901234567890 | 可口可乐 | 3.5 |
| 6901234567891 | 农夫山泉 | 2.0 |
| 6901234567892 | 乐事薯片 | 6.8 |
| 6901234567893 | 康师傅方便面 | 4.5 |
| 6901234567894 | 奥利奥饼干 | 8.9 |
| 6901234567895 | 蒙牛纯牛奶 | 5.2 |

**错误响应**

- 条码不存在 → `404 Not Found`：

```json
{"error": "未找到该条码对应的商品"}
```

- 请求体不是合法 JSON、字段类型错误或缺少 `Content-Type` → `400 Bad Request`：

```json
{"error": "请求体 JSON 字段类型不正确"}
```

## 统一错误处理

所有错误接口统一返回 `{"error": "..."}` 的 JSON 格式，便于调用方解析。

- 业务错误（条码找不到）返回 `404`，错误信息来自 `ApiError::not_found`。
- 请求体解析失败由自定义提取器 `ValidJson<T>` 捕获 axum 默认的 `JsonRejection`，统一转换为 `400` + 友好提示，避免向客户端暴露 serde 的内部实现细节；详细原因仅写入服务端日志（`WARN` 级别）。

## tracing 日志

服务启动时初始化 `tracing-subscriber`，默认以 `INFO` 级别输出到控制台。每个 HTTP 请求由日志中间件记录：请求方法、路径、状态码、处理耗时。

通过环境变量 `RUST_LOG` 可调整日志级别：

```bash
RUST_LOG=debug cargo run   # 输出 DEBUG 及以上级别
RUST_LOG=off cargo run     # 关闭日志
```

## 质量检查

```bash
# 编译
cargo build

# 静态检查（把 warning 视为错误）
cargo clippy --all-targets -- -D warnings

# 格式化检查
cargo fmt --check
```

## AI 辅助生成声明

本项目部分代码与文档由 AI 辅助生成，但均经过人工验证和修改（包括编译、静态检查与接口实测）。
