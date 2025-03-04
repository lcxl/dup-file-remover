ARG DFREMOVER_DEBUG_VERSION=false
# Build stage
FROM rust:1.84.1 as rust-builder
WORKDIR /usr/src/dfr
COPY . .
RUN cargo install --locked --path .

FROM node:20 as node-builder
WORKDIR /usr/src/dfr/web
COPY web .
RUN npm ci && npm run build

# Final stage
FROM debian:bullseye-slim
RUN apt-get update && apt-get install -y extra-runtime-dependencies && rm -rf /var/lib/apt/lists/*
# Copy the built web application from the node-builder stage
COPY --from=node-builder /usr/src/dfr/web/dist /usr/local/bin/dup-file-remover/static
# Copy the dup-file-remover binary from the rust-builder stage
COPY --from=rust-builder /usr/local/cargo/bin/dup-file-remover /usr/local/bin/dup-file-remover
CMD ["/usr/local/bin/dup-file-remover"]