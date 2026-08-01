# Stage 1: Cargo Build Environment
FROM rust:1.80-slim-bookworm AS builder

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    clang \
    cmake \
    libpdfium-dev \
    libtesseract-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY . .
RUN cargo build --release --bin api

# Stage 2: Production Cloud Run Runtime
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libtesseract5 \
    tesseract-ocr-eng \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/release/api /app/convertflow-api

ENV PORT=8080
ENV RUST_LOG=info
EXPOSE 8080

CMD ["/app/convertflow-api"]
