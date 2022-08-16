FROM rust:latest as builder
WORKDIR /api-otel-example
COPY . .
# RUN cargo install --path .
# RUN cargo clean && cargo build -vv --release
RUN cargo build --release
CMD ["./target/release/api-otel-example"]

FROM debian:bullseye-slim
RUN apt-get update && apt-get install -y  build-essential && rm -rf /var/lib/apt/lists/*
COPY --from=builder /api-otel-example/target/release/api-otel-example /usr/local/bin/api-otel-example
EXPOSE 8080
CMD ["api-otel-example"]
