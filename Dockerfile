FROM rust:1.97 AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
# 先构建一个空 main 以缓存依赖，避免每次改代码都重新编译依赖
RUN mkdir src && echo "fn main() {}" > src/main.rs && cargo build --release
COPY src ./src
RUN cargo build --release

FROM debian:bookworm-slim
WORKDIR /app
COPY --from=builder /app/target/release/self-checkout-service /usr/local/bin/
ENV ADDR=0.0.0.0:3000
EXPOSE 3000
CMD ["self-checkout-service"]
