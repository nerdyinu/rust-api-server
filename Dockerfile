FROM rustlang/rust:nightly as builder

# Install PostgreSQL client libraries for building
RUN apt-get update && \
  apt-get install -y libpq-dev && \
  rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/app
COPY . .
# Do not expand sqlx macros since db connection is not set
ENV SQLX_OFFLINE=true
RUN cargo build --release

FROM debian:bookworm-slim

# Install runtime dependencies for PostgreSQL
RUN apt-get update && \
  apt-get install -y libpq5 openssl ca-certificates && \
  rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/src/app/target/release/api-server /usr/local/bin/api-server

EXPOSE 8080

# Add healthcheck
HEALTHCHECK --interval=30s --timeout=3s \
  CMD curl -f http://localhost:8080/health || exit 1

CMD ["api-server"]
