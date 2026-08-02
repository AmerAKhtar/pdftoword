# Stage 1: Cargo Build Environment
FROM rust:1.80-slim-bookworm AS builder

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    clang \
    cmake \
    libtesseract-dev \
    curl \
    tar \
    && rm -rf /var/lib/apt/lists/*

RUN mkdir -p /tmp/pdfium \
    && curl -fL -s https://github.com/bblanchon/pdfium-binaries/releases/download/chromium/7961/pdfium-linux-x64.tgz | tar -xz -C /tmp/pdfium \
    && cp /tmp/pdfium/lib/libpdfium.so /usr/lib/ \
    && rm -rf /tmp/pdfium

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

COPY --from=builder /usr/lib/libpdfium.so /usr/lib/libpdfium.so

WORKDIR /app
COPY --from=builder /app/target/release/api /app/convertflow-api

ENV PORT=8080
ENV RUST_LOG=info
EXPOSE 8080

CMD ["/app/convertflow-api"]
