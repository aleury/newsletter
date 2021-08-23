############### Builder stage ###############
FROM rust:1.54.0 AS builder
WORKDIR /app
RUN cargo install --locked --branch master \
    --git https://github.com/aleury/cargo-build-deps
COPY Cargo.toml Cargo.lock ./
RUN cargo build-deps --release
COPY . .
ENV SQLX_OFFLINE true
RUN cargo build --release

############### Runtime stage ###############
FROM debian:buster-slim AS runtime
WORKDIR /app
RUN apt-get update -y \
    && apt-get install -y --no-install-recommends openssl \
    # Clean up
    && apt-get autoremove -y \
    && apt-get clean -y \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/newsletter newsletter
COPY config config
ENV APP_ENVIRONMENT production
ENTRYPOINT ["./newsletter"]