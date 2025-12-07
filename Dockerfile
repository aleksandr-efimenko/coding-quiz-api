FROM rust:alpine as builder

WORKDIR /usr/src/app

# Install build dependencies
# We need musl-dev/gcc for compiling C deps if any, and curl/unzip for swagger ui
RUN apk add --no-cache pkgconfig openssl-dev libc-dev curl unzip musl-dev make gcc

# Copy manifests
COPY Cargo.toml Cargo.lock ./
COPY .sqlx ./.sqlx
ENV SQLX_OFFLINE=true

# Copy source code
COPY src ./src
COPY scripts ./scripts
COPY seed ./seed
COPY migrations ./migrations

# Build binaries
RUN cargo build --release --bin coding-quiz-api
RUN cargo build --release --bin migrate
RUN cargo build --release --bin seed

# Runtime stage
FROM alpine:latest

WORKDIR /usr/local/bin

# Install runtime dependencies
RUN apk add --no-cache \
    ca-certificates \
    libssl3 \
    postgresql-client \
    bash

# Copy binaries
COPY --from=builder /usr/src/app/target/release/coding-quiz-api .
COPY --from=builder /usr/src/app/target/release/migrate .
COPY --from=builder /usr/src/app/target/release/seed ./seed-cli

# Copy data
COPY --from=builder /usr/src/app/migrations ./migrations
COPY --from=builder /usr/src/app/seed ./seed

# Copy entrypoint
COPY entrypoint.sh .
RUN chmod +x entrypoint.sh

# Expose API port
EXPOSE 8080

ENTRYPOINT ["./entrypoint.sh"]
