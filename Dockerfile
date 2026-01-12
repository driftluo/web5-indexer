FROM rust:1.92.0-slim as build
WORKDIR /usr/src/web5-indexer

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    ca-certificates \
    build-essential \
    && rm -rf /var/lib/apt/lists/*

COPY . .
RUN cd /usr/src/web5-indexer && cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    libssl3 \
    libssl-dev \
    ca-certificates && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=build /usr/src/web5-indexer/target/release/web5-indexer /app/web5-indexer
CMD ["/app/web5-indexer"]
