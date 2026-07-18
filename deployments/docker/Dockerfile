FROM rust:1-bookworm AS builder
WORKDIR /src
COPY . .
RUN cargo build --release -p sdkwork-agents-standalone-gateway

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /src/target/release/sdkwork-agents-standalone-gateway /usr/local/bin/sdkwork-agents-standalone-gateway
COPY --from=builder /src/database /app/database
COPY --from=builder /src/etc /app/etc
COPY deployments/docker/agents-container-entrypoint.sh /usr/local/bin/sdkwork-agents-container-entrypoint
RUN chmod 0755 /usr/local/bin/sdkwork-agents-container-entrypoint
ENV SDKWORK_AGENTS_APP_ROOT=/app
EXPOSE 8095
ENTRYPOINT ["sdkwork-agents-container-entrypoint"]
CMD ["sdkwork-agents-standalone-gateway"]
