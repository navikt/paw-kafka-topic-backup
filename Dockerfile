FROM clux/muslrust:stable as builder
WORKDIR /build
COPY . .
ENV RUSTFLAGS='-C target-feature=+crt-static'
RUN cargo build --release

FROM gcr.io/distroless/static-debian11:nonroot
WORKDIR /app
COPY --from=builder /build/target/x86_64-unknown-linux-musl/release/paw-kafka-topic-backup /app/paw-kafka-topic-backup
EXPOSE 8080
CMD ["RUST_BACKTRACE=1 /app/paw-kafka-topic-backup"]