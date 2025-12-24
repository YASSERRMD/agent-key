"""Pytest configuration and fixtures."""

import pytest


@pytest.fixture
def api_key() -> str:
    """Test API key."""
    return "ak_test123_randomsuffix"


@pytest.fixture
def base_url() -> str:
    """Test base URL."""
    return "https://api.test.agentkey.io"


@pytest.fixture
def sample_token() -> str:
    """Sample JWT token for testing."""
    # This is a valid JWT structure (header.payload.signature)
    # Payload: {"sub": "cred-123", "agent_id": "agent-456", "secret": "test-secret", "exp": 9999999999}
    import base64
    import json
    
    header = base64.urlsafe_b64encode(json.dumps({"alg": "HS256", "typ": "JWT"}).encode()).decode().rstrip("=")
    payload = base64.urlsafe_b64encode(json.dumps({
        "sub": "cred-123",
        "agent_id": "agent-456",
        "team_id": "team-789",
        "secret": "test-secret-value",
        "credential_type": "password",
        "credential_name": "test-credential",
        "exp": 9999999999,
        "iat": 1234567890,
        "jti": "jti-test-123",
        "token_type": "ephemeral"
    }).encode()).decode().rstrip("=")
    signature = "fake_signature_for_testing"
    
    return f"{header}.{payload}.{signature}"


@pytest.fixture
def expired_token() -> str:
    """Expired JWT token for testing."""
    import base64
    import json
    
    header = base64.urlsafe_b64encode(json.dumps({"alg": "HS256", "typ": "JWT"}).encode()).decode().rstrip("=")
    payload = base64.urlsafe_b64encode(json.dumps({
        "sub": "cred-123",
        "agent_id": "agent-456",
        "secret": "test-secret-value",
        "exp": 1000000000,  # Expired
        "jti": "jti-expired-123"
    }).encode()).decode().rstrip("=")
    signature = "fake_signature"
    
    return f"{header}.{payload}.{signature}"
