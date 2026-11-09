FROM rust:1.85 AS builder

WORKDIR /application

COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY resources ./resources

RUN cargo build --locked --release

FROM debian:bookworm-slim

COPY --from=builder /application/target/release/actname /usr/local/bin/actname

ENTRYPOINT ["/usr/local/bin/actname"]
