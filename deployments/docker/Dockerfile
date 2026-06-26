FROM rust:1-bookworm AS builder
WORKDIR /src
COPY . .
RUN cargo build --release -p sdkwork-agents-api-server

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /src/target/release/sdkwork-agents-api-server /usr/local/bin/sdkwork-agents-api-server
COPY --from=builder /src/target/release/sdkwork-agents-standalone-gateway /usr/local/bin/sdkwork-agents-standalone-gateway
COPY --from=builder /src/database /app/database
ENV SDKWORK_AGENTS_APP_ROOT=/app
ENV SDKWORK_AGENTS_ENVIRONMENT=production
ENV SDKWORK_AGENTS_CONFIG_PROFILE=production
ENV SDKWORK_AGENTS_APPLICATION_PUBLIC_INGRESS_BIND=0.0.0.0:8095
EXPOSE 8095
CMD ["sdkwork-agents-standalone-gateway"]
