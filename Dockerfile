FROM clux/muslrust:stable as builder
WORKDIR /build
COPY . .
ENV RUSTFLAGS='-C target-feature=+crt-static'
RUN cargo build --target x86_64-unknown-linux-musl --release

FROM cgr.dev/chainguard/static:latest
WORKDIR /app
COPY --from=builder /build/target/x86_64-unknown-linux-musl/release/paw-kafka-topic-backup /app/paw-kafka-topic-backup
EXPOSE 8080
ENTRYPOINT ["/app/paw-kafka-topic-backup"]