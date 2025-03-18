ARG DFREMOVER_DEBUG_VERSION=false
# Build stage
FROM rust:1.85.0 AS rust-builder
WORKDIR /usr/src/dfr
COPY . .
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,sharing=private,target=/usr/src/dfr/target \
    cargo install --locked --path .

FROM node:20 AS node-builder
WORKDIR /usr/src/dfr/web
COPY web .
RUN --mount=type=cache,target=/root/.npm \
    npm ci && npm run build

# Final stage
FROM debian:bookworm-slim
# RUN apt-get update && apt-get install -y extra-runtime-dependencies && rm -rf /var/lib/apt/lists/*
# Copy the built web application from the node-builder stage
COPY --from=node-builder /usr/src/dfr/web/dist /app/static
# Copy the dup-file-remover binary from the rust-builder stage
COPY --from=rust-builder /usr/local/cargo/bin/dup-file-remover /app/dup-file-remover
WORKDIR /app
CMD ["/app/dup-file-remover"]