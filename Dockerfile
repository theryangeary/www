# use chef for faster rust builds/better caching
FROM lukemathwalker/cargo-chef:latest-rust-1.88 AS chef
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
ARG TARGETARCH
ARG TARGETOS
RUN case "${TARGETARCH}" in \
      amd64) TW_ARCH="x64" ;; \
      arm64) TW_ARCH="arm64" ;; \
      *) echo "Unsupported arch: ${TARGETARCH}"; exit 1 ;; \
    esac \
    && curl -sLO https://github.com/tailwindlabs/tailwindcss/releases/latest/download/tailwindcss-${TARGETOS}-${TW_ARCH} \
    && chmod +x tailwindcss-${TARGETOS}-${TW_ARCH} \
    && mv tailwindcss-${TARGETOS}-${TW_ARCH} tailwindcss

RUN cargo build --release

FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

RUN groupadd --gid 1001 appgroup && \
    useradd \
        --uid 1001 \
        --gid appgroup \
        --shell /bin/bash \
        --no-create-home \
        --no-log-init \
        appuser

WORKDIR /app

COPY --from=builder /app/target/release/www /www

COPY --chown=appuser:appgroup . .

USER appuser

EXPOSE 3000

ENTRYPOINT ["/www"]

