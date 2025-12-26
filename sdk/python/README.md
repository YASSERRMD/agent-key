# AgentKey Python SDK

Python SDK for AgentKey - a secure credential management platform for AI agents.

## Installation

```bash
pip install agentkey
```

Or install from source:

```bash
cd sdk/python
pip install -e .
```

## Quick Start

```python
from agentkey import AgentKey

# Initialize with your agent's API key (from AgentKey dashboard)
agent = AgentKey(
    api_key="ak_your_agent_api_key",
    base_url="http://localhost:8080"  # Your AgentKey server
)

# Get a credential value
openai_key = agent.get_credential_value("openai-key")
print(f"Retrieved key: {openai_key[:10]}...")

# Use it in your application
import openai
openai.api_key = openai_key
```

## Features

- **Simple API**: Just a few methods for all common operations
- **Token Caching**: Automatic caching of ephemeral tokens
- **Auto-Refresh**: Tokens refresh automatically before expiration
- **Full CRUD**: Create, read, update, delete credentials programmatically
- **Type Hints**: Full type hint coverage for IDE support
- **Python 3.8+**: Compatible with Python 3.8 and above

## API Reference

### Initialization

```python
from agentkey import AgentKey

agent = AgentKey(
    api_key="ak_...",           # Required: Agent API key
    base_url="http://...",      # Optional: API base URL (default: https://api.agentkey.io)
    timeout=30                  # Optional: Request timeout in seconds
)
```

### Admin Access

You can also use a **Team Admin Key** (from Settings > API Keys) to initialize the client. Admin keys have universal access:

```python
# Initialize with Admin Key
admin = AgentKey(api_key="ak_team_admin_key")

# Admin can access ANY credential in the team
# Useful for debugging, migration scripts, or super-agent orchestration
any_cred = admin.get_credential_value("other-agent-secret")
```

### Get Credential Value

Get the decrypted secret value of a credential.

```python
# Get credential by name
password = agent.get_credential_value("database-password")
api_key = agent.get_credential_value("openai-key")

# Use in your application
conn = psycopg2.connect(host="db.example.com", password=password)
```

### List Credentials

List all credentials available to this agent.

```python
credentials = agent.list_credentials()

for cred in credentials["data"]:
    print(f"- {cred['name']} ({cred['credential_type']})")

# Pagination
page2 = agent.list_credentials(page=2, limit=10)
```

### Create Credential

Create a new credential for this agent.

```python
# Basic credential
agent.create_credential(
    name="new-api-key",
    secret="sk-abc123..."
)

# With type and description
agent.create_credential(
    name="database-password",
    secret="super-secret-password",
    credential_type="database",
    description="Production DB password"
)
```

**Credential Types:**
- `generic` (default)
- `api_key`
- `aws`
- `openai`
- `database`
- `oauth`

### Update Credential

Update an existing credential's secret value.

```python
# Update secret
agent.update_credential(
    credential_name="openai-key",
    new_secret="sk-new-key-value"
)

# Update with description
agent.update_credential(
    credential_name="openai-key",
    new_secret="sk-new-key-value",
    description="Rotated on 2024-12-26"
)
```

### Delete Credential

Delete a credential.

```python
agent.delete_credential("old-credential")
```

### Get Ephemeral Token

Get a short-lived JWT token containing the credential.

```python
token = agent.get_credential_token("database-password")

# Token is cached and auto-refreshed (30s buffer before expiration)
```

### Revoke Token

Revoke a token before its natural expiration.

```python
from agentkey._utils import parse_jwt_payload

token = agent.get_credential_token("secret")
payload = parse_jwt_payload(token)
agent.revoke_token(payload["jti"])
```

### Health Check

Check if the AgentKey server is available.

```python
if agent.health_check():
    print("AgentKey is healthy!")
else:
    print("AgentKey is unavailable")
```

### Cache Management

The SDK automatically caches tokens. You can manage the cache manually:

```python
# Clear cache for specific credential
agent.clear_cache("database-password")

# Clear all cached tokens
agent.clear_cache()

# Force refresh a token
token = agent.get_credential_token("secret", force_refresh=True)
```

## Error Handling

```python
from agentkey import (
    AgentKey,
    AgentKeyError,
    AuthenticationError,
    NotFoundError,
    RateLimitError,
    ServerError
)

agent = AgentKey(api_key="ak_...")

try:
    password = agent.get_credential_value("db-password")
except AuthenticationError:
    print("Invalid API key")
except NotFoundError:
    print("Credential not found")
except RateLimitError:
    print("Too many requests, slow down")
except ServerError:
    print("AgentKey service unavailable")
except AgentKeyError as e:
    print(f"Unexpected error: {e}")
```

## Complete Example

```python
from agentkey import AgentKey, NotFoundError
import openai

def main():
    # Initialize SDK
    agent = AgentKey(
        api_key="ak_your_agent_api_key",
        base_url="http://localhost:8080"
    )
    
    # Check health
    if not agent.health_check():
        raise Exception("AgentKey is not available")
    
    # List available credentials
    print("Available credentials:")
    for cred in agent.list_credentials()["data"]:
        print(f"  - {cred['name']} ({cred['credential_type']})")
    
    # Get OpenAI key
    try:
        openai_key = agent.get_credential_value("openai-key")
        openai.api_key = openai_key
        print(f"✓ OpenAI key loaded: {openai_key[:10]}...")
    except NotFoundError:
        print("✗ OpenAI key not found, creating...")
        agent.create_credential(
            name="openai-key",
            secret="sk-your-openai-key",
            credential_type="openai"
        )
    
    # Your AI agent logic here...
    print("Agent ready!")

if __name__ == "__main__":
    main()
```

## Best Practices

1. **Use `get_credential_value()`**: For most cases, this is all you need
2. **Don't log secrets**: Never log the returned credential values
3. **Handle errors**: Always wrap calls in try/except
4. **Single instance**: Create one `AgentKey` instance and reuse it
5. **Rotate regularly**: Use `update_credential()` to rotate secrets

## Environment Variables

You can also configure the SDK using environment variables:

```bash
export AGENTKEY_API_KEY="ak_your_key"
export AGENTKEY_BASE_URL="http://localhost:8080"
```

```python
import os
from agentkey import AgentKey

agent = AgentKey(
    api_key=os.environ.get("AGENTKEY_API_KEY"),
    base_url=os.environ.get("AGENTKEY_BASE_URL", "http://localhost:8080")
)
```

## Testing

Run the SDK tests:

```bash
cd sdk/python
python -m pytest tests/ -v
```

See `tests/` directory for test results and examples.

## License

This SDK is part of the AgentKey project and is subject to the AgentKey Source Available License.
