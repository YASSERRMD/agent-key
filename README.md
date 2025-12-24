# AgentKey Backend

**Agent-Native Credential Management Platform**

A production-ready Rust backend for secure credential management for AI agents.

## Features

- 🔐 **AES-256-GCM Encryption** - Industry-standard encryption for credentials
- 🎫 **JWT Authentication** - Secure token-based authentication
- 🐘 **PostgreSQL** - Reliable database with comprehensive schema
- ⚡ **Redis** - Fast caching layer
- 🐍 **Python SDK** - Native client for easy agent integration
- 🔑 **Ephemeral Tokens** - Short-lived, secure access for agents
- 🦀 **Actix-web** - High-performance async web framework
- ✅ **Comprehensive Tests** - 49+ tests covering all services

## Quick Start

### Prerequisites

- Rust 1.75+
- Docker & Docker Compose
- PostgreSQL 15
- Redis 7

### Setup

1. **Clone and enter the repository**
   ```bash
   git clone https://github.com/YASSERRMD/agent-key.git
   cd agent-key
   ```

2. **Start infrastructure**
   ```bash
   docker-compose up -d
   ```

3. **Configure environment**
   ```bash
   cp .env.example .env
   # Edit .env with your settings
   ```

4. **Run the server**
   ```bash
   cargo run
   ```

5. **Verify health**
   ```bash
   curl http://localhost:8080/health
   ```

## API Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/health` | Liveness probe |
| GET | `/health/ready` | Readiness probe |
| GET | `/api/v1/health/detailed` | Detailed health with latency |

## Testing

```bash
# Run all tests
cargo test

# Run with single thread (for env isolation)
cargo test -- --test-threads=1

# Run only unit tests
cargo test --lib

# Run only integration tests
cargo test --test '*'
```

## Project Structure

```
src/
├── main.rs           # Entry point
├── lib.rs            # Library exports
├── config.rs         # Environment configuration
├── db.rs             # Database connection pool
├── errors.rs         # Centralized error handling
├── server.rs         # Actix-web configuration
├── handlers/
│   ├── mod.rs        # Route configuration
│   └── health.rs     # Health check endpoints
└── services/
    ├── mod.rs        # Service exports
    ├── encryption.rs # AES-256-GCM encryption
    └── jwt.rs        # JWT token service
```

## Configuration

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `DATABASE_URL` | Yes | - | PostgreSQL connection URL |
| `JWT_SECRET` | Yes | - | JWT signing secret (min 32 chars) |
| `ENCRYPTION_KEY` | Yes | - | AES encryption key (min 32 chars) |
| `REDIS_URL` | No | `redis://localhost:6379` | Redis connection URL |
| `SERVER_HOST` | No | `127.0.0.1` | Server bind host |
| `SERVER_PORT` | No | `8080` | Server bind port |
| `ENVIRONMENT` | No | `development` | Environment name |
| `LOG_LEVEL` | No | `info` | Logging level |

## License

MIT
