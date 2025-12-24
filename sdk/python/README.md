# AgentKey Python SDK

Python SDK for AgentKey agent credential management with ephemeral token support.

## Installation

```bash
pip install agentkey
```

## Quick Start

```python
from agentkey import AgentKey

# Initialize client with your agent API key
agent = AgentKey(api_key="ak_your_agent_key_here")

# Get credential value directly (recommended)
password = agent.get_credential_value("db-password")

# Use in your application
import psycopg2
conn = psycopg2.connect(
    host="db.example.com",
    user="app_user",
    password=password
)
```

## Features

- **Simple API**: Just 3-5 methods for all common operations
- **Token Caching**: Automatic caching of ephemeral tokens
- **Auto-Refresh**: Tokens refresh automatically before expiration (30s buffer)
- **Type Hints**: Full type hint coverage for IDE support
- **Python 3.8+**: Compatible with Python 3.8 and above

## API Reference

### `AgentKey(api_key, base_url, timeout)`

Initialize the AgentKey client.

**Parameters:**
- `api_key` (str): Agent API key from AgentKey dashboard
- `base_url` (str, optional): API base URL. Default: `https://api.agentkey.io`
- `timeout` (int, optional): Request timeout in seconds. Default: `30`

### `get_credential_value(credential_name) -> str`

Get decrypted credential value. This is the recommended method for most use cases.

```python
password = agent.get_credential_value("db-password")
api_key = agent.get_credential_value("openai-api-key")
```

### `get_credential_token(credential_name, force_refresh) -> str`

Get ephemeral JWT token for credential. Useful when you need the full token.

```python
token = agent.get_credential_token("db-password")
# Token is cached and auto-refreshed
```

### `list_credentials(page, limit) -> dict`

List available credentials.

```python
creds = agent.list_credentials()
for cred in creds["data"]:
    print(f"{cred['name']}: {cred['type']}")
```

### `revoke_token(jti) -> None`

Revoke a token before expiration.

```python
from agentkey._utils import parse_jwt_payload

token = agent.get_credential_token("secret")
payload = parse_jwt_payload(token)
agent.revoke_token(payload["jti"])
```

### `health_check() -> bool`

Check if AgentKey service is available.

```python
if agent.health_check():
    print("AgentKey is healthy!")
```

## Error Handling

```python
from agentkey import AgentKey, AuthenticationError, NotFoundError, ServerError

agent = AgentKey(api_key="ak_...")

try:
    password = agent.get_credential_value("db-password")
except AuthenticationError:
    print("Invalid API key")
except NotFoundError:
    print("Credential not found")
except ServerError:
    print("AgentKey service unavailable")
```

## Token Caching

The SDK automatically caches tokens and refreshes them 30 seconds before expiration:

```python
# First call: fetches new token
token1 = agent.get_credential_token("db-password")  # HTTP request

# Second call: uses cached token (no HTTP request)
token2 = agent.get_credential_token("db-password")  # From cache

# Force refresh if needed
token3 = agent.get_credential_token("db-password", force_refresh=True)

# Clear cache manually
agent.clear_cache("db-password")  # Clear specific credential
agent.clear_cache()  # Clear all
```

## Best Practices

1. **Use `get_credential_value()`**: For most cases, this is all you need
2. **Don't log secrets**: Never log the returned credential values
3. **Handle errors**: Always wrap calls in try/except
4. **Single instance**: Create one `AgentKey` instance and reuse it

## License

MIT License
