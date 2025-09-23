FROM rust:latest as builder
WORKDIR /build
COPY . .
ENV RUSTFLAGS='-C target-feature=+crt-static'
RUN cargo build --release

FROM cgr.dev/chainguard/static:latest
WORKDIR /app
COPY --from=builder /build/target/x86_64-unknown-linux-musl/release/paw-kafka-topic-backup /app/paw-kafka-topic-backup
EXPOSE 8080
ENV RUST_BACKTRACE=1
CMD ["/app/paw-kafka-topic-backup"]