FROM lukemathwalker/cargo-chef:latest as chef
WORKDIR /app

FROM chef AS planner
COPY ./Cargo.toml ./Cargo.lock ./
COPY ./src ./src
COPY ./tlsn-src ./tlsn-src
RUN cargo chef prepare

FROM chef AS builder
COPY --from=planner /app/recipe.json .
COPY ./tlsn-src ./tlsn-src
ENV GIT_COMMIT_HASH=0000000000000000000000000000000000000000
RUN cargo chef cook --release
COPY . .
RUN cargo build --release
RUN mv ./target/release/Validation_Service ./app

FROM debian:stable-slim AS runtime
RUN apt-get update && apt-get install -y \
    libssl3 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /app/app /usr/local/bin/
ENTRYPOINT ["/usr/local/bin/app"]