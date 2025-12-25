# AgentKey

<p align="center">
  <img src="frontend/public/logo.png" alt="AgentKey Logo" width="150" height="150">
</p>


## Overview
AgentKey is a centralized credential management platform specifically designed for AI agents. It provides a secure, production-ready environment for storing, rotating, and accessing credentials using industry-standard encryption and short-lived ephemeral tokens.

The platform consists of a high-performance Rust backend and a modern React frontend dashboard for seamless management of agents and credentials.

## Core Features
- **Secure Credential Storage**: All sensitive data is encrypted using AES-256-GCM.
- **Agent Management**: Dedicated identities for AI agents with granular access controls.
- **API Key Management**: Support for multiple API keys per agent with expiration and revocation capabilities.
- **Credential Rotation**: Native support for rotating API keys, OAuth tokens, and secrets with version history.
- **Ephemeral Access**: Generation of short-lived tokens for secure agent-to-service communication.
- **Comprehensive Audit Logs**: Detailed tracking of credential access and management actions.
- **React Dashboard**: Intuitive user interface for managing agents, teams, and security settings.

## Technical Architecture

### Backend (Rust)
- **Framework**: Actix-web for high-performance asynchronous request handling.
- **Database**: PostgreSQL for persistent storage with a structured schema.
- **Caching**: Redis for session management and token validation.
- **Security**: AES-256-GCM for data encryption and JWT for user authentication.

### Frontend (React)
- **Build Tool**: Vite for fast development and optimized production builds.
- **State Management**: Zustand for efficient global state handling.
- **Styling**: Tailwind CSS for a responsive and consistent design system.
- **Navigation**: React Router for seamless page transitions.
- **Icons**: Lucide React for consistent visual language.

## Getting Started

### Prerequisites
- Rust 1.75 or higher
- Node.js 18 or higher
- Docker and Docker Compose
- PostgreSQL 15
- Redis 7

### Installation

1. **Clone the Repository**
   ```bash
   git clone https://github.com/YASSERRMD/agent-key.git
   cd agent-key
   ```

2. **Backend Configuration**
   ```bash
   cp .env.example .env
   # Update .env with your local database and secret keys
   ```

3. **Frontend Configuration**
   ```bash
   cd frontend
   npm install
   ```

4. **Initialize Infrastructure**
   ```bash
   docker-compose up -d
   ```

5. **Run the Application**
   - **Backend**: `cargo run` (from the root directory)
   - **Frontend**: `npm run dev` (from the frontend directory)

## API Reference
The backend exposes several critical endpoints for management and agent interaction.

| Category | Endpoint | Method | Description |
|----------|----------|--------|-------------|
| Auth | `/auth/login` | POST | User authentication |
| Agents | `/agents` | GET/POST | List and create agents |
| API Keys | `/agents/{id}/keys` | GET/POST | Manage agent API keys |
| Tokens | `/tokens/generate` | POST | Generate ephemeral tokens |
| Health | `/health` | GET | System health check |

## Project Structure
```text
agent-key/
├── src/                # Rust backend source code
│   ├── handlers/       # HTTP request handlers
│   ├── services/       # Core business logic and security services
│   ├── models/         # Database models and DTOs
│   └── middleware/     # Auth and security middleware
├── frontend/           # React dashboard source code
│   ├── src/
│   │   ├── components/ # Reusable UI components
│   │   ├── pages/      # View layouts
│   │   └── store/      # Global state stores
├── tests/              # Integration and unit tests
└── docker-compose.yml  # Infrastructure orchestration
```

## Development and Testing

### Backend Tests
```bash
cargo test
```

### Frontend Tests
```bash
cd frontend
npm test
```

## License
This project is licensed under the MIT License.
