FROM rust:1-slim AS chef
USER root
RUN cargo install cargo-chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json --bin duplexscan

FROM chef AS builder
# Performance-oriented flags
ENV CARGO_INCREMENTAL=1
ARG RUSTFLAGS="-C target-cpu=native -C opt-level=3"
ENV RUSTFLAGS=${RUSTFLAGS}


COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json --bin duplexscan

COPY . .
RUN cargo build --release --bin duplexscan

FROM scratch
COPY --from=builder /app/target/release/duplexscan /