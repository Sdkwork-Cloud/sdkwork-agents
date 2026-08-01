FROM rust:1-bookworm AS builder
WORKDIR /workspace
COPY sdkwork-agents ./sdkwork-agents
COPY sdkwork-database ./sdkwork-database
COPY sdkwork-iam ./sdkwork-iam
COPY sdkwork-id ./sdkwork-id
COPY sdkwork-kernel ./sdkwork-kernel
COPY sdkwork-prompts ./sdkwork-prompts
COPY sdkwork-sandbox ./sdkwork-sandbox
COPY sdkwork-utils ./sdkwork-utils
COPY sdkwork-web-framework ./sdkwork-web-framework
WORKDIR /workspace/sdkwork-agents
RUN cargo build --locked --release \
    -p sdkwork-api-agents-standalone-gateway \
    -p sdkwork-intelligence-agents-worker

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /workspace/sdkwork-agents/target/release/sdkwork-api-agents-standalone-gateway /usr/local/bin/sdkwork-api-agents-standalone-gateway
COPY --from=builder /workspace/sdkwork-agents/target/release/sdkwork-intelligence-agents-worker /usr/local/bin/sdkwork-intelligence-agents-worker
COPY --chown=65532:65532 --from=builder /workspace/sdkwork-agents/database /app/database
COPY --chown=65532:65532 --from=builder /workspace/sdkwork-agents/etc /app/etc
COPY sdkwork-agents/deployments/docker/agents-container-entrypoint.sh /usr/local/bin/sdkwork-agents-container-entrypoint
RUN chmod 0755 /usr/local/bin/sdkwork-agents-container-entrypoint
ENV SDKWORK_AGENTS_APP_ROOT=/app
EXPOSE 8095 8096
USER 65532:65532
STOPSIGNAL SIGTERM
ENTRYPOINT ["sdkwork-agents-container-entrypoint"]
CMD ["sdkwork-api-agents-standalone-gateway"]
