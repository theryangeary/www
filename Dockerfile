# use chef for faster rust builds/better caching
FROM lukemathwalker/cargo-chef:latest-rust-1.87 AS chef
WORKDIR /app

# generate chef plan
FROM chef AS planner

COPY Cargo.toml Cargo.lock build.rs ./
COPY src ./src

RUN cargo chef prepare --recipe-path recipe.json

# build rust bins
FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json

RUN cargo chef cook --release --recipe-path recipe.json

COPY . .

RUN apt-get update && apt-get install -y curl
RUN curl -sLO https://github.com/tailwindlabs/tailwindcss/releases/latest/download/tailwindcss-linux-arm64 \
    && chmod +x tailwindcss-linux-arm64 \
    && mv tailwindcss-linux-arm64 tailwindcss

RUN cargo build --release

FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/www /www

EXPOSE 3000

ENTRYPOINT ["/www"]

