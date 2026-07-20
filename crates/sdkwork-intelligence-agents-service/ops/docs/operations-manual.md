# SDKWork Intelligence Agents Service - Operations Manual

## Overview

This document provides operational procedures for the SDKWork Intelligence Agents Service,
covering deployment, monitoring, maintenance, and troubleshooting procedures.

## Architecture

### Components

- **Agents Service**: Core business logic for agent management
- **PostgreSQL Store**: Persistent storage for agent configurations and audit events
- **Prometheus Metrics**: Observability endpoint at `/metrics`
- **Grafana Dashboard**: Real-time monitoring and alerting

### Data Flow

```
Gateway → Agents Service → PostgreSQL Store
                ↓
         Audit Events Sink
                ↓
         Kernel Event Store
```

## Deployment

### Prerequisites

1. PostgreSQL 13+ with the SDKWork Agents `3.1.0` baseline applied by the lifecycle orchestrator
2. Kubernetes cluster with an ingress/controller appropriate for the deployment
3. Prometheus-compatible metrics collection for `/metrics/agents`

### Kubernetes Deployment

```bash
# Apply the application deployment, service, HPA and disruption budget
kubectl apply -f deployments/kubernetes/standalone-gateway-deployment.yaml
kubectl apply -f deployments/kubernetes/standalone-gateway-service.yaml
kubectl apply -f deployments/kubernetes/standalone-gateway-hpa.yaml
kubectl apply -f deployments/kubernetes/standalone-gateway-pdb.yaml

# Verify deployment status
kubectl get deployment sdkwork-api-agents-standalone-gateway
kubectl get pods -l app.kubernetes.io/name=sdkwork-agents
kubectl get hpa sdkwork-api-agents-standalone-gateway
```

### Health Check Endpoints

- **Liveness**: `GET /health` (port 8095)
- **Readiness**: `GET /ready` (port 8095)
- **Metrics**: `GET /metrics/agents` (port 8095)

### Environment Variables

| Variable | Required | Description |
|----------|----------|-------------|
| `SDKWORK_DEPLOYMENT_ENV` | Yes | Must be `production` for prod deployments |
| `SDKWORK_AGENTS_DEV_AUTH_BYPASS` | Yes | Must be `false` in production |
| `SDKWORK_AGENTS_STORE_DATABASE_URL` | Yes | PostgreSQL connection string |
| `SDKWORK_AGENTS_SERVICE_WORKER_LIMIT` | Yes | Maximum concurrent synchronous service workers; production profile default `128` |
| `SDKWORK_AGENTS_PROVIDER_WORKER_LIMIT` | Yes | Maximum concurrent provider executions; production profile default `32` |
| `SDKWORK_TENANT_ID` | No | Default tenant for bootstrap |
| `SDKWORK_ORGANIZATION_ID` | No | Default organization |

## Monitoring

### Prometheus Metrics

Access metrics at `http://agents.sdkwork.io/metrics`

Key metrics:
- `sdkwork_agents_total` - Total number of agents
- `sdkwork_agents_active` - Active agents count
- `sdkwork_agents_deleted` - Soft-deleted agents
- `sdkwork_agents_requests_total` - Request count by operation
- `sdkwork_agents_errors_total` - Error count by operation
- `sdkwork_agents_audit_events_total` - Total audit events

### Grafana Dashboard

Import dashboard from `ops/grafana/dashboard-agents.json`

Dashboard panels:
1. Agent Count Overview (gauge)
2. Request Rate by Operation (time series)
3. Error Rate (percentage gauge)
4. Active Provider Bindings
5. Total Audit Events
6. Errors by Operation (hourly bar chart)

### Alerting Rules

Recommended Prometheus alerts:

```yaml
groups:
  - name: sdkwork-agents
    rules:
      - alert: HighErrorRate
        expr: sum(rate(sdkwork_agents_errors_total[5m])) / sum(rate(sdkwork_agents_requests_total[5m])) > 0.05
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "High error rate in agents service"

      - alert: NoActiveAgents
        expr: sdkwork_agents_active == 0
        for: 10m
        labels:
          severity: critical
        annotations:
          summary: "No active agents in system"

      - alert: DatabaseConnectionPoolExhausted
        expr: sdkwork_db_pool_utilization > 0.9
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "Database connection pool near exhaustion"
```

## Maintenance

### Scaling

Horizontal Pod Autoscaler scales based on:
- CPU utilization (target: 70%)
- Memory utilization (target: 75%)

Manual scaling:
```bash
kubectl scale deployment sdkwork-api-agents-standalone-gateway --replicas=5
```

### Database Maintenance

#### Connection Pool Monitoring

Check pool metrics via Prometheus:
```
sdkwork_db_pool_size
sdkwork_db_pool_idle_connections
sdkwork_db_pool_utilization
```

#### Schema Migration

Run schema migrations before deployment:
```bash
psql -d agents_store -f specs/sql/agents_managed_store_postgres.sql
```

### Backup Procedures

1. **Database Backup**:
   ```bash
   pg_dump -Fc agents_store > agents_store_backup_$(date +%Y%m%d).dump
   ```

2. **Audit Event Archive**:
   ```sql
   -- Archive audit events older than 90 days
   INSERT INTO audit_events_archive 
   SELECT * FROM audit_events WHERE occurred_at < NOW() - INTERVAL '90 days';
   DELETE FROM audit_events WHERE occurred_at < NOW() - INTERVAL '90 days';
   ```

## Security

### Production Security Checks

The service enforces fail-closed security:

1. **DEV_AUTH_BYPASS Check**: Production bootstrap rejects
   `SDKWORK_AGENTS_DEV_AUTH_BYPASS=true` in production-like environments, and
   dev-only static policy construction falls back to deny-all if misconfigured.
   This prevents accidental deployment of development-only authentication bypass.

2. **Tenant Isolation**: All queries enforce `tenant_id` filtering via parameterized SQL.

3. **SQL Injection Prevention**: All queries use parameterized statements ($1, $2, etc.).

4. **Audit Trail**: All operations emit structured audit events with JSON payloads.

### Security Verification

```bash
# Verify security configuration
kubectl exec -it deployment/sdkwork-intelligence-agents -n sdkwork -- env | grep SDKWORK

# Expected output:
# SDKWORK_DEPLOYMENT_ENV=production
# SDKWORK_AGENTS_DEV_AUTH_BYPASS=false
```

## Troubleshooting

See `ops/docs/troubleshooting.md` for detailed troubleshooting procedures.

### Quick Diagnostics

```bash
# Check pod logs
kubectl logs -l app.kubernetes.io/name=sdkwork-agents --tail=100

# Check pod status
kubectl describe pod -l app.kubernetes.io/name=sdkwork-agents

# Check service endpoints
kubectl get endpoints sdkwork-api-agents-standalone-gateway

# Check HPA status
kubectl describe hpa sdkwork-api-agents-standalone-gateway
```

### Common Issues

| Issue | Symptom | Resolution |
|-------|---------|------------|
| Auth Bypass Rejected | Pod fails readiness or dev-only policy denies requests | Set `SDKWORK_AGENTS_DEV_AUTH_BYPASS=false` |
| Database Connection Error | 5xx errors on all requests | Verify database URL and network connectivity |
| HPA Not Scaling | Pods stay at min replicas | Check resource metrics and the configured worker limits |
| High Error Rate | Dashboard shows >5% errors | Check downstream provider status |

## Upgrade Procedure

1. **Pre-upgrade**:
   - Backup database
   - Verify migration scripts
   - Check compatibility matrix

2. **Upgrade**:
   ```bash
   # Update image version
   kubectl set image deployment/sdkwork-intelligence-agents \
     agents-service=sdkwork/intelligence-agents-service:v2.0.0 -n sdkwork
   
   # Monitor rollout
   kubectl rollout status deployment/sdkwork-intelligence-agents -n sdkwork
   ```

3. **Post-upgrade**:
   - Verify health endpoints
   - Check metrics collection
   - Run smoke tests

## Rollback Procedure

```bash
# Rollback to previous version
kubectl rollout undo deployment/sdkwork-intelligence-agents -n sdkwork

# Verify rollback
kubectl rollout history deployment/sdkwork-intelligence-agents -n sdkwork
```

## Support

For production issues, contact:
- Platform Team: platform-team@sdkwork.io
- On-call: Check PagerDuty schedule
- Escalation: See incident response playbook
