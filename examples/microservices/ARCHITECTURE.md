# Architecture Overview

This document describes the architecture of the microservices example and how the observability components work together.

## System Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                    Client Applications                          │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐            │
│  │   Browser   │  │   cURL      │  │   Postman   │            │
│  └─────────────┘  └─────────────┘  └─────────────┘            │
└─────────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────┐
│                    Microservices Layer                          │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐            │
│  │ Demo Service│  │Users Service│  │Articles Svc │            │
│  │   :8080     │  │   :8081     │  │   :8082     │            │
│  └─────────────┘  └─────────────┘  └─────────────┘            │
│         │                │                │                    │
│         └────────────────┼────────────────┘                    │
│                          │                                     │
│         ┌────────────────▼────────────────┐                    │
│         │        HTTP Communication       │                    │
│         └─────────────────────────────────┘                    │
└─────────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────┐
│                  Observability Layer                            │
│                                                                 │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │              Grafana Alloy (Agent)                      │   │
│  │  ┌─────────────────────────────────────────────────┐   │   │
│  │  │              Log Collection                     │   │   │
│  │  │  • Monitors /app/logs/*.log                     │   │   │
│  │  │  • Extracts trace_id from JSON logs             │   │   │
│  │  │  • Adds app labels (articles/users/demo)        │   │   │
│  │  └─────────────────────────────────────────────────┘   │   │
│  └─────────────────────────────────────────────────────────┘   │
│                                │                               │
│                                ▼                               │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │              OpenTelemetry (OTLP)                      │   │   │
│  │  • Traces sent to http://198.18.0.1:4317              │   │   │
│  │  • Metrics sent to http://198.18.0.1:4317             │   │   │
│  └─────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────┐
│                  Storage & Visualization                        │
│                                                                 │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐            │
│  │    Loki     │  │    Tempo    │  │   Grafana   │            │
│  │   (Logs)    │  │  (Traces)   │  │ (Dashboard) │            │
│  │   :3100     │  │   :3200     │  │   :3000     │            │
│  └─────────────┘  └─────────────┘  └─────────────┘            │
│         │                │                │                    │
│         └────────────────┼────────────────┘                    │
│                          │                                     │
│         ┌────────────────▼────────────────┐                    │
│         │        Unified Query            │                    │
│         │      (Logs + Traces)            │                    │
│         └─────────────────────────────────┘                    │
└─────────────────────────────────────────────────────────────────┘
```

## Component Details

### 1. Microservices

#### Demo Service (:8080)
- **Purpose**: Simple demo application
- **Endpoints**: `/hello`
- **Configuration**: `demo.env`
- **Logs**: `./logs/demo/`

#### Users Service (:8081)
- **Purpose**: User management (CRUD operations)
- **Endpoints**: 
  - `POST /users` - Create user
  - `GET /users/{id}` - Get user
  - `GET /users` - List users
- **Configuration**: `users.env`
- **Logs**: `./logs/users/`

#### Articles Service (:8082)
- **Purpose**: Article management with user dependencies
- **Endpoints**:
  - `POST /articles` - Create article
  - `GET /articles/author/{id}` - Get articles by author
  - `GET /articles/{id}` - Get article
- **Dependencies**: Communicates with Users Service
- **Configuration**: `articles.env`
- **Logs**: `./logs/articles/`

### 2. Observability Components

#### Grafana Alloy
- **Version**: v1.7.5
- **Purpose**: Observability agent
- **Configuration**: `alloy/config.alloy`
- **Functions**:
  - File-based log collection
  - Log processing and enrichment
  - Trace ID extraction
  - Forwarding to Loki

#### Log Collection Pipeline
```
Service Logs → Alloy → Loki → Grafana
     ↓           ↓       ↓       ↓
  JSON logs   Process  Store   Query
  with        & enrich & index & visualize
  trace_id    metadata
```

#### OpenTelemetry Pipeline
```
Service → OTLP Exporter → Tempo → Grafana
   ↓           ↓           ↓        ↓
Traces    HTTP/gRPC    Store    Query &
Spans     protocol     traces   visualize
```

### 3. Data Flow

#### Request Flow
1. **Client Request** → Microservice
2. **Service Processing** → Internal logic + HTTP calls
3. **Logging** → JSON logs with trace context
4. **Tracing** → OTLP export to Tempo
5. **Log Collection** → Alloy processes and forwards to Loki

#### Observability Flow
1. **Log Collection**: Alloy monitors log files
2. **Log Processing**: Extract trace_id and metadata
3. **Log Storage**: Forward to Loki with labels
4. **Trace Collection**: OTLP export to Tempo
5. **Visualization**: Grafana queries both Loki and Tempo

### 4. Configuration Files

#### Environment Files
- `articles.env` - Articles service configuration
- `users.env` - Users service configuration  
- `demo.env` - Demo service configuration

#### Alloy Configuration
- `alloy/config.alloy` - Log collection and processing rules

#### Docker Compose
- `docker-compose.yml` - Service orchestration and networking

### 5. Networking

#### Internal Communication
- Services communicate via Docker network
- Users Service: `http://users-service:8081`
- Articles Service: `http://articles-service:8082`

#### External Access
- Demo Service: `http://localhost:8080`
- Users Service: `http://localhost:8081`
- Articles Service: `http://localhost:8082`

#### Observability Endpoints
- Grafana: `http://localhost:3000`
- Loki: `http://localhost:3100`
- Tempo: `http://localhost:3200`

### 6. Data Persistence

#### Logs
- **Format**: JSON with trace context
- **Location**: `./logs/{service}/`
- **Rotation**: Daily with max 10 files
- **Retention**: Configurable via Alloy

#### Traces
- **Format**: OpenTelemetry protocol
- **Storage**: Tempo (configurable retention)
- **Query**: Via Grafana Tempo data source

### 7. Monitoring & Alerting

#### Built-in Monitoring
- Service health checks
- Log level monitoring
- Trace sampling rates
- Error rate tracking

#### Custom Metrics
- Request duration
- Response status codes
- Service dependencies
- Resource utilization

## Deployment Considerations

### Production Readiness
- **Scaling**: Horizontal scaling via Docker Compose
- **Security**: Environment variable management
- **Monitoring**: Health checks and alerting
- **Backup**: Log and trace data retention policies

### Performance
- **Sampling**: Configurable trace sampling rates
- **Buffering**: Log buffering to prevent data loss
- **Caching**: Grafana query caching
- **Compression**: Log compression for storage efficiency

### Security
- **Network**: Isolated Docker networks
- **Secrets**: Environment variable management
- **Access**: Grafana authentication
- **Audit**: Comprehensive logging and tracing 