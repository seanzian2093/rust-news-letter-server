# Builder stage
FROM rust:1.84.0 AS builder
# Switch working directory to `app` (equivalent to `cd app`)
# The `app` folder will be created for us by Docker in case it does not
# exist already.
WORKDIR /app
# Copy all files from our working environment to our Docker image
COPY . .
# force sqlx to look at saved metadata instead of trying to query a live database
ENV SQLX_OFFLINE true
# Use the release profile to make it faaaast
RUN cargo build --release

# Runtime stage
# FROM rust:1.84.0-slim AS runtime
# Use slim version to reduce image size
FROM debian:bullseye-slim AS runtime
WORKDIR /app
RUN apt-get update \ 
    && apt-get install -y --no-install-recommends openssl \
    && apt-get autoremove -y \
    && apt-get clean -y \
    && rm -rf /var/lib/apt/lists/*
# Only copy binary
COPY --from=builder /app/target/release/rust-new-letter-server rust-new-letter-server
COPY configuration configuration
ENV APP_ENVIRONMENT production
ENTRYPOINT ["./rust-new-letter-server"]