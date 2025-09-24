FROM rust:bookworm as builder
WORKDIR /build
RUN apt-get update && apt-get install -y --no-install-recommends cmake
COPY . .
#ENV RUSTFLAGS='-C target-feature=+crt-static'
RUN cargo build --release

#FROM cgr.dev/chainguard/static:latest
FROM rust:1.89.0-slim
WORKDIR /app
COPY --from=builder /build/target/release/paw-kafka-topic-backup /app/paw-kafka-topic-backup
EXPOSE 8080
ENTRYPOINT ["/app/paw-kafka-topic-backup"]