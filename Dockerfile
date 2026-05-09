# Stage 1: Build WASM
FROM rust:1.95-slim AS builder

RUN apt-get update && apt-get install -y pkg-config && rm -rf /var/lib/apt/lists/*
RUN rustup target add wasm32-unknown-unknown
RUN cargo install wasm-bindgen-cli --version 0.2.121

WORKDIR /build
COPY Cargo.toml Cargo.lock ./
COPY src/ src/

RUN cargo build --target wasm32-unknown-unknown --release
RUN wasm-bindgen target/wasm32-unknown-unknown/release/ecosystem_sim.wasm --out-dir pkg --web

# Stage 2: Serve with nginx
FROM nginx:alpine

COPY --from=builder /build/pkg/ /usr/share/nginx/pkg/
COPY index.html /usr/share/nginx/index.html
COPY nginx.conf /etc/nginx/conf.d/default.conf

EXPOSE 8080
CMD ["nginx", "-g", "daemon off;"]
