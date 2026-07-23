# SDKWork Intelligence Agents Service - Troubleshooting Guide

## Overview

This guide covers common issues, diagnostic procedures, and resolution steps for the 
SDKWork Intelligence Agents Service in production environments.

## Diagnostic Tools

### Log Analysis

```bash
# View recent logs
kubectl logs -l app=sdkwork-intelligence-agents -n sdkwork --tail=200

# Stream logs in real-time
kubectl logs -f -l app=sdkwork-intelligence-agents -n sdkwork

# Search for errors
kubectl logs -l app=sdkwork-intelligence-agents -n sdkwork | grep -i error

# Search for specific request ID
kubectl logs -l app=sdkwork-intelligence-agents -n sdkwork | grep "request_id=abc123"
```

### Database Diagnostics

```bash
# Connect to database
psql $SDKWORK_AGENTS_DATABASE_URL

# Check table sizes
SELECT 
    schemaname, 
    tablename, 
    pg_size_pretty(pg_total_relation_size(schemaname||'.'||tablename)) as size
FROM pg_tables 
WHERE schemaname = 'public'
ORDER BY pg_total_relation_size(schemaname||'.'||tablename) DESC;

# Check active connections
SELECT count(*) FROM pg_stat_activity WHERE datname = 'agents_store';

# Check for long-running queries
SELECT pid, query, state, query_start 
FROM pg_stat_activity 
WHERE state = 'active' AND query_start < NOW() - INTERVAL '30 seconds';
```

### Metrics Queries

```bash
# Query Prometheus metrics
curl -s "http://prometheus:9090/api/v1/query?query=sdkwork_agents_errors_total" | jq

# Check error rate
curl -s "http://prometheus:9090/api/v1/query?query=rate(sdkwork_agents_errors_total[5m])" | jq

# Check request latency percentiles (if available)
curl -s "http://prometheus:9090/api/v1/query?query=histogram_quantile(0.99,sdkwork_agents_request_duration_seconds_bucket)" | jq
```

## Common Issues

### 1. Service Crash on Startup

**Symptoms**:
- Pod crashes immediately after startup
- Crash loop backoff status
- No logs or very short logs

**Diagnostic Steps**:
```bash
# Check pod events
kubectl describe pod <pod-name> -n sdkwork

# Check previous container logs
kubectl logs <pod-name> -n sdkwork --previous
```

**Common Causes**:

#### A. DEV_AUTH_BYPASS in Production

```
agents security bootstrap rejected SDKWORK_AGENTS_DEV_AUTH_BYPASS in a production-like environment
```

**Resolution**:
```bash
# Fix environment variable
kubectl set env deployment/sdkwork-intelligence-agents \
  SDKWORK_AGENTS_DEV_AUTH_BYPASS=false -n sdkwork

# Verify fix
kubectl rollout status deployment/sdkwork-intelligence-agents -n sdkwork
```

#### B. Database Connection Failure

```
database pool: connection refused
```

**Resolution**:
1. Verify database URL is correct
2. Check network connectivity from pods to database
3. Verify database credentials in secret
4. Check if database is accepting connections

```bash
# Test database connectivity from pod
kubectl exec -it <pod-name> -n sdkwork -- nc -zv <db-host> 5432

# Verify secret
kubectl get secret sdkwork-agents-db-credentials -n sdkwork -o yaml
```

#### C. Schema Not Initialized

```
relation "ai_agent" does not exist
```

**Resolution**:
```bash
# Run schema migration
psql $SDKWORK_AGENTS_DATABASE_URL -f database/ddl/baseline/postgres/0001_agents_baseline.sql
```

### 2. High Error Rate

**Symptoms**:
- Dashboard shows error rate > 5%
- Client applications receiving 5xx errors
- Alert triggered

**Diagnostic Steps**:
```bash
# Check error counts by operation
curl -s "http://prometheus:9090/api/v1/query?query=sum+by+(operation)(rate(sdkwork_agents_errors_total[5m]))" | jq

# Check recent error logs
kubectl logs -l app=sdkwork-intelligence-agents -n sdkwork | grep -i error | tail -50
```

**Common Causes**:

#### A. Database Connection Pool Exhaustion

```
pool exhausted, waiting for connection
```

**Resolution**:
1. Check pool metrics
2. Increase pool size in configuration
3. Scale horizontally via HPA

```yaml
# Increase pool size in ConfigMap
database:
  pool:
    min_size: 10
    max_size: 40
```

#### B. Rate Limiting

```
rate limit exceeded
```

**Resolution**:
1. Check ingress rate limit configuration
2. Verify legitimate traffic patterns
3. Adjust rate limit if needed

#### C. Invalid Request Data

```
validation error: invalid agent_id format
```

**Resolution**:
1. Check client request format
2. Verify API version compatibility
3. Update client application if needed

### 3. Slow Response Times

**Symptoms**:
- High latency in request processing
- Client timeouts
- Dashboard shows elevated latency

**Diagnostic Steps**:
```bash
# Check database query performance
SELECT query, calls, total_time/calls as avg_time_ms 
FROM pg_stat_statements 
ORDER BY avg_time_ms DESC 
LIMIT 10;

# Check for missing indexes
SELECT schemaname, tablename, attname, n_distinct, correlation 
FROM pg_stats 
WHERE schemaname = 'public' AND tablename LIKE 'ai_agent%';
```

**Common Causes**:

#### A. Missing Database Index

**Resolution**:
```sql
-- Add index on frequently queried columns
CREATE INDEX CONCURRENTLY idx_ai_agent_status ON ai_agent(status);
CREATE INDEX CONCURRENTLY idx_ai_agent_updated_at ON ai_agent(updated_at DESC);
```

#### B. Large Result Sets

**Resolution**:
1. Verify pagination is being used
2. Check `page_size` parameter
3. Add additional filters to queries

#### C. Network Latency

**Resolution**:
1. Check network policies
2. Verify database proximity to application
3. Consider read replicas for read-heavy workloads

### 4. Authentication Failures

**Symptoms**:
- 401/403 responses to valid requests
- "access denied" errors

**Diagnostic Steps**:
```bash
# Check request headers
kubectl logs -l app=sdkwork-intelligence-agents -n sdkwork | grep "subject_id"

# Verify IAM integration
kubectl exec -it <pod-name> -n sdkwork -- curl -s http://localhost:8095/health
```

**Common Causes**:

#### A. Missing Gateway Headers

```
missing required header: x-subject-id
```

**Resolution**:
1. Verify gateway configuration
2. Check header forwarding rules
3. Ensure IAM token validation is enabled

#### B. Insufficient Permissions

```
access denied: ai.agents.manage permission required
```

**Resolution**:
1. Check user roles in IAM
2. Verify permission grants
3. Update policy if needed

### 5. Memory Issues

**Symptoms**:
- OOMKilled pods
- Increasing memory usage over time
- Memory-related alerts

**Diagnostic Steps**:
```bash
# Check memory usage
kubectl top pods -l app=sdkwork-intelligence-agents -n sdkwork

# Get memory metrics from Prometheus
curl -s "http://prometheus:9090/api/v1/query?query=container_memory_working_set_bytes{pod=~\"sdkwork-intelligence-agents.*\"}" | jq
```

**Resolution**:
1. Check for memory leaks in application
2. Increase memory limits if justified
3. Verify connection pool sizing
4. Check for unbounded data structures

```yaml
# Increase memory limits
resources:
  limits:
    memory: "1Gi"
```

## Performance Tuning

### Database Optimization

```sql
-- Analyze table statistics
ANALYZE ai_agent;

-- Vacuum tables to reclaim space
VACUUM ANALYZE ai_agent;

-- Check index usage
SELECT indexrelname, idx_scan, idx_tup_read, idx_tup_fetch 
FROM pg_stat_user_indexes 
WHERE schemaname = 'public'
ORDER BY idx_scan ASC;
```

### Connection Pool Tuning

```yaml
# Optimal pool sizing formula: connections = (core_count * 2) + effective_spindle_count
database:
  pool:
    min_size: 10
    max_size: 30
    idle_timeout: 300
    max_lifetime: 1800
```

### Query Optimization

1. Always use pagination
2. Filter by tenant_id first
3. Use indexes on frequently filtered columns
4. Avoid SELECT * in queries

## Incident Response

### Severity Levels

| Level | Description | Response Time | Examples |
|-------|-------------|---------------|----------|
| P1 | Critical - Service down | 15 minutes | Database unreachable, auth bypass rejected |
| P2 | High - Degraded service | 30 minutes | High error rate, slow responses |
| P3 | Medium - Limited impact | 2 hours | Non-critical feature broken |
| P4 | Low - Minor issue | 1 business day | Documentation error, minor UI bug |

### Escalation Path

1. **L1 Support**: Initial triage, known issue resolution
2. **L2 Support**: Complex troubleshooting, configuration changes
3. **L3 Support**: Database issues, security incidents
4. **Platform Team**: Infrastructure issues, major incidents

### Post-Incident Checklist

- [ ] Root cause identified
- [ ] Fix implemented and verified
- [ ] Monitoring/alerts updated if needed
- [ ] Documentation updated
- [ ] Post-mortem scheduled (P1/P2 only)

## Reference Commands

```bash
# Full pod status
kubectl get all -l app=sdkwork-intelligence-agents -n sdkwork

# Resource usage
kubectl top pods -l app=sdkwork-intelligence-agents -n sdkwork

# Events
kubectl get events -n sdkwork --sort-by='.lastTimestamp'

# Describe deployment
kubectl describe deployment sdkwork-intelligence-agents -n sdkwork

# Check ingress
kubectl get ingress sdkwork-intelligence-agents-ingress -n sdkwork

# Port forward for local testing
kubectl port-forward deployment/sdkwork-intelligence-agents 8095:8095 -n sdkwork
```
