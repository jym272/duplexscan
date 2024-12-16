FROM clux/muslrust@sha256:30938544fe77218583d8fcf96f5e6c0c8a29e87abc0a5f1a6943e3bba607f8ae AS chef
USER root

RUN cargo install cargo-chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json --bin duplexscan

FROM chef AS builder

ENV CARGO_INCREMENTAL=0

# https://github.com/johnthagen/min-sized-rust?tab=readme-ov-file#compress-the-binary
ARG RUSTFLAGS="-C target-feature=+crt-static"
ENV RUSTFLAGS=${RUSTFLAGS}
COPY --from=planner /app/recipe.json recipe.json

# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --target x86_64-unknown-linux-musl --recipe-path recipe.json --bin duplexscan
# Build application
COPY . .
RUN cargo build --release --target x86_64-unknown-linux-musl --bin duplexscan

# Stage: Install UPX and compress the binary if needed!
#FROM alpine:latest AS compressor
#
#WORKDIR /root
#
## Download and extract UPX
#RUN wget https://github.com/upx/upx/releases/download/v4.2.4/upx-4.2.4-amd64_linux.tar.xz \
#    && tar -xf upx-4.2.4-amd64_linux.tar.xz \
#    && mv upx-4.2.4-amd64_linux/upx /usr/local/bin \
#    && rm -rf upx-4.2.4-amd64_linux.tar.xz upx-4.2.4-amd64_linux
#
## Copy the binary from the builder stage
#COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/duplexscan .
#
## Compress the binary
#RUN upx --best --lzma duplexscan

FROM scratch
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/duplexscan /
