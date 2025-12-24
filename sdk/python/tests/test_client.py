"""Tests for AgentKey client."""

import json
import time
from unittest.mock import MagicMock, patch

import pytest
import responses

from agentkey import AgentKey, AuthenticationError, NotFoundError, ServerError
from agentkey._utils import extract_secret_from_token, parse_jwt_payload


class TestClientInitialization:
    """Tests for client initialization."""

    def test_client_initialization(self, api_key: str, base_url: str):
        """Test basic client initialization."""
        client = AgentKey(api_key=api_key, base_url=base_url)
        
        assert client.api_key == api_key
        assert client.base_url == base_url
        assert client.timeout == 30
        assert "X-API-Key" in client.session.headers
        assert client.session.headers["X-API-Key"] == api_key

    def test_client_initialization_empty_api_key(self):
        """Test that empty API key raises ValueError."""
        with pytest.raises(ValueError, match="api_key is required"):
            AgentKey(api_key="")

    def test_client_extracts_agent_id(self, base_url: str):
        """Test agent_id extraction from API key."""
        client = AgentKey(api_key="ak_myagent_suffix", base_url=base_url)
        assert client.agent_id == "myagent"

    def test_client_user_agent_header(self, api_key: str, base_url: str):
        """Test User-Agent header is set correctly."""
        client = AgentKey(api_key=api_key, base_url=base_url)
        assert "agentkey-python/" in client.session.headers["User-Agent"]

    def test_client_repr(self, api_key: str, base_url: str):
        """Test string representation."""
        client = AgentKey(api_key=api_key, base_url=base_url)
        repr_str = repr(client)
        assert "AgentKey" in repr_str
        assert base_url in repr_str


class TestTokenCaching:
    """Tests for token caching functionality."""

    def test_get_credential_token_caching(self, api_key: str, base_url: str, sample_token: str):
        """Test that tokens are cached."""
        client = AgentKey(api_key=api_key, base_url=base_url)
        
        # Manually populate cache
        client._token_cache["test-cred"] = (sample_token, int(time.time()) + 300)
        
        # Should return cached token
        token = client.get_credential_token("test-cred")
        assert token == sample_token

    def test_get_credential_token_cache_expired(self, api_key: str, base_url: str, expired_token: str):
        """Test that expired cached tokens trigger refresh."""
        client = AgentKey(api_key=api_key, base_url=base_url)
        
        # Set expired token in cache
        client._token_cache["test-cred"] = (expired_token, int(time.time()) - 100)
        
        # Should not use expired token (will raise because no mock)
        assert not client._is_token_valid("test-cred")

    def test_is_token_valid_with_buffer(self, api_key: str, base_url: str, sample_token: str):
        """Test token validity check with refresh buffer."""
        client = AgentKey(api_key=api_key, base_url=base_url)
        
        # Token expiring in 20 seconds (less than 30s buffer)
        client._token_cache["test-cred"] = (sample_token, int(time.time()) + 20)
        assert not client._is_token_valid("test-cred")
        
        # Token expiring in 60 seconds (more than 30s buffer)
        client._token_cache["test-cred"] = (sample_token, int(time.time()) + 60)
        assert client._is_token_valid("test-cred")

    def test_clear_cache_specific(self, api_key: str, base_url: str, sample_token: str):
        """Test clearing specific credential from cache."""
        client = AgentKey(api_key=api_key, base_url=base_url)
        
        client._token_cache["cred1"] = (sample_token, int(time.time()) + 300)
        client._token_cache["cred2"] = (sample_token, int(time.time()) + 300)
        
        client.clear_cache("cred1")
        
        assert "cred1" not in client._token_cache
        assert "cred2" in client._token_cache

    def test_clear_cache_all(self, api_key: str, base_url: str, sample_token: str):
        """Test clearing all cache."""
        client = AgentKey(api_key=api_key, base_url=base_url)
        
        client._token_cache["cred1"] = (sample_token, int(time.time()) + 300)
        client._token_cache["cred2"] = (sample_token, int(time.time()) + 300)
        
        client.clear_cache()
        
        assert len(client._token_cache) == 0


class TestGetCredentialValue:
    """Tests for get_credential_value method."""

    def test_get_credential_value(self, api_key: str, base_url: str, sample_token: str):
        """Test getting credential value from cached token."""
        client = AgentKey(api_key=api_key, base_url=base_url)
        
        # Pre-populate cache
        client._token_cache["test-credential"] = (sample_token, int(time.time()) + 300)
        
        value = client.get_credential_value("test-credential")
        assert value == "test-secret-value"


class TestErrorHandling:
    """Tests for error handling."""

    @responses.activate
    def test_authentication_error(self, api_key: str, base_url: str):
        """Test 401 response raises AuthenticationError."""
        client = AgentKey(api_key=api_key, base_url=base_url)
        
        responses.add(
            responses.POST,
            f"{base_url}/api/v1/agents/test123/credentials/test/token",
            json={"error": "Unauthorized"},
            status=401
        )
        
        with pytest.raises(AuthenticationError):
            client.get_credential_token("test")

    @responses.activate
    def test_not_found_error(self, api_key: str, base_url: str):
        """Test 404 response raises NotFoundError."""
        client = AgentKey(api_key=api_key, base_url=base_url)
        
        responses.add(
            responses.POST,
            f"{base_url}/api/v1/agents/test123/credentials/missing/token",
            json={"message": "Credential not found"},
            status=404
        )
        
        with pytest.raises(NotFoundError):
            client.get_credential_token("missing")

    @responses.activate
    def test_server_error(self, api_key: str, base_url: str):
        """Test 5xx response raises ServerError."""
        client = AgentKey(api_key=api_key, base_url=base_url)
        
        responses.add(
            responses.POST,
            f"{base_url}/api/v1/agents/test123/credentials/test/token",
            json={"error": "Internal error"},
            status=500
        )
        
        with pytest.raises(ServerError):
            client.get_credential_token("test")


class TestHealthCheck:
    """Tests for health check functionality."""

    @responses.activate
    def test_health_check_success(self, api_key: str, base_url: str):
        """Test successful health check."""
        client = AgentKey(api_key=api_key, base_url=base_url)
        
        responses.add(
            responses.GET,
            f"{base_url}/api/v1/health",
            json={"status": "healthy"},
            status=200
        )
        
        assert client.health_check() is True

    @responses.activate
    def test_health_check_failure(self, api_key: str, base_url: str):
        """Test failed health check."""
        client = AgentKey(api_key=api_key, base_url=base_url)
        
        responses.add(
            responses.GET,
            f"{base_url}/api/v1/health",
            json={"error": "Unhealthy"},
            status=503
        )
        
        assert client.health_check() is False


class TestUtilityFunctions:
    """Tests for utility functions."""

    def test_parse_jwt_payload(self, sample_token: str):
        """Test JWT payload parsing."""
        payload = parse_jwt_payload(sample_token)
        
        assert payload["sub"] == "cred-123"
        assert payload["agent_id"] == "agent-456"
        assert payload["secret"] == "test-secret-value"
        assert payload["token_type"] == "ephemeral"

    def test_parse_jwt_payload_invalid(self):
        """Test invalid JWT raises ValueError."""
        with pytest.raises(ValueError, match="Invalid JWT format"):
            parse_jwt_payload("not.a.valid.jwt.token")

    def test_extract_secret_from_token(self, sample_token: str):
        """Test secret extraction from token."""
        secret = extract_secret_from_token(sample_token)
        assert secret == "test-secret-value"

    def test_extract_secret_missing(self):
        """Test missing secret raises ValueError."""
        import base64
        import json
        
        header = base64.urlsafe_b64encode(json.dumps({"alg": "HS256"}).encode()).decode().rstrip("=")
        payload = base64.urlsafe_b64encode(json.dumps({"sub": "test"}).encode()).decode().rstrip("=")
        token = f"{header}.{payload}.sig"
        
        with pytest.raises(ValueError, match="does not contain 'secret'"):
            extract_secret_from_token(token)
