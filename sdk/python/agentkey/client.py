"""AgentKey client for agent credential management."""

import logging
import time
from typing import Any, Dict, List, Optional, Tuple

import requests

from ._utils import extract_secret_from_token, get_token_expiration, parse_jwt_payload
from ._version import __version__
from .exceptions import (
    AgentKeyError,
    AuthenticationError,
    NotFoundError,
    RateLimitError,
    ServerError,
    TokenExpiredError,
)

logger = logging.getLogger(__name__)

# Token refresh buffer (seconds before expiration to refresh)
TOKEN_REFRESH_BUFFER = 30


class AgentKey:
    """
    AgentKey client for agent credential management.
    
    Provides methods to retrieve ephemeral tokens and credential values
    with automatic token caching and refresh.
    
    Example:
        >>> agent = AgentKey(api_key="ak_...")
        >>> password = agent.get_credential_value("db-password")
        >>> token = agent.get_credential_token("api-key")
    """

    def __init__(
        self,
        api_key: str,
        base_url: str = "https://api.agentkey.io",
        timeout: int = 30,
    ):
        """
        Initialize AgentKey client.
        
        Args:
            api_key: Agent API key from AgentKey dashboard
            base_url: AgentKey API base URL
            timeout: Request timeout in seconds
        """
        if not api_key:
            raise ValueError("api_key is required")
        
        self.api_key = api_key
        self.base_url = base_url.rstrip("/")
        self.timeout = timeout
        
        # Extract agent_id from API key if possible (format: ak_{agent_id}_{random})
        self._agent_id: Optional[str] = None
        if api_key.startswith("ak_"):
            parts = api_key.split("_")
            if len(parts) >= 2:
                self._agent_id = parts[1]
        
        # Create session with default headers
        self.session = requests.Session()
        self.session.headers.update({
            "X-API-Key": api_key,
            "User-Agent": f"agentkey-python/{__version__}",
            "Content-Type": "application/json",
            "Accept": "application/json",
        })
        
        # Token cache: {credential_name: (token, expires_at_unix)}
        self._token_cache: Dict[str, Tuple[str, int]] = {}

    @property
    def agent_id(self) -> Optional[str]:
        """Get agent ID extracted from API key."""
        return self._agent_id

    def _handle_response(self, response: requests.Response) -> Dict[str, Any]:
        """
        Handle API response and raise appropriate exceptions.
        
        Args:
            response: Requests response object
            
        Returns:
            Parsed JSON response
            
        Raises:
            AuthenticationError: For 401 responses
            NotFoundError: For 404 responses
            RateLimitError: For 429 responses
            ServerError: For 5xx responses
            AgentKeyError: For other errors
        """
        if response.status_code == 401:
            raise AuthenticationError("Invalid API key or unauthorized")
        
        if response.status_code == 403:
            raise AuthenticationError("Access forbidden")
        
        if response.status_code == 404:
            try:
                data = response.json()
                message = data.get("message", "Resource not found")
            except Exception:
                message = "Resource not found"
            raise NotFoundError(message)
        
        if response.status_code == 429:
            retry_after = int(response.headers.get("Retry-After", 60))
            raise RateLimitError("Rate limit exceeded", retry_after=retry_after)
        
        if response.status_code >= 500:
            raise ServerError(f"Server error: {response.status_code}")
        
        if response.status_code >= 400:
            try:
                data = response.json()
                message = data.get("message", f"Request failed: {response.status_code}")
            except Exception:
                message = f"Request failed: {response.status_code}"
            raise AgentKeyError(message, status_code=response.status_code)
        
        if response.status_code == 204:
            return {}
        
        try:
            return response.json()
        except Exception:
            return {}

    def _is_token_valid(self, credential_name: str) -> bool:
        """
        Check if cached token is still valid.
        
        Args:
            credential_name: Name of credential
            
        Returns:
            True if token exists and has more than TOKEN_REFRESH_BUFFER seconds left
        """
        if credential_name not in self._token_cache:
            return False
        
        _, expires_at = self._token_cache[credential_name]
        now = int(time.time())
        
        # Invalid if less than buffer time remaining
        return (expires_at - now) > TOKEN_REFRESH_BUFFER

    def get_credential_token(self, credential_name: str, force_refresh: bool = False) -> str:
        """
        Get ephemeral token for credential.
        
        Uses cached token if still valid (more than 30s before expiration).
        Automatically refreshes expired or soon-to-expire tokens.
        
        Args:
            credential_name: Name of credential (e.g., "db-password")
            force_refresh: Force token refresh even if cached token is valid
            
        Returns:
            JWT token string
            
        Raises:
            AuthenticationError: If API key is invalid
            NotFoundError: If credential not found
            ServerError: For server errors
        """
        # Check cache first (unless force refresh)
        if not force_refresh and self._is_token_valid(credential_name):
            token, _ = self._token_cache[credential_name]
            logger.debug(f"Using cached token for '{credential_name}'")
            return token
        
        # Request new token
        if not self._agent_id:
            raise AgentKeyError("Cannot determine agent_id from API key")
        
        url = f"{self.base_url}/api/v1/agents/{self._agent_id}/credentials/{credential_name}/token"
        
        try:
            response = self.session.post(url, timeout=self.timeout)
            data = self._handle_response(response)
        except requests.RequestException as e:
            raise ServerError(f"Network error: {e}")
        
        token = data.get("token")
        if not token:
            raise AgentKeyError("Invalid response: missing token")
        
        # Cache token
        expires_at = get_token_expiration(token)
        if expires_at:
            self._token_cache[credential_name] = (token, expires_at)
            logger.debug(f"Cached new token for '{credential_name}', expires at {expires_at}")
        
        return token

    def get_credential_value(self, credential_name: str) -> str:
        """
        Get decrypted credential value.
        
        Retrieves an ephemeral token and extracts the secret from it.
        
        Args:
            credential_name: Name of credential
            
        Returns:
            Decrypted secret value
            
        Raises:
            AuthenticationError: If API key is invalid
            NotFoundError: If credential not found
            ServerError: For server errors
        """
        token = self.get_credential_token(credential_name)
        return extract_secret_from_token(token)

    def list_credentials(
        self,
        page: int = 1,
        limit: int = 20,
    ) -> Dict[str, Any]:
        """
        List available credentials.
        
        Args:
            page: Page number (1-indexed)
            limit: Items per page (max 100)
            
        Returns:
            Dict with keys: data, total, page, limit, pages
            
        Raises:
            AuthenticationError: If API key is invalid
            ServerError: For server errors
        """
        if not self._agent_id:
            raise AgentKeyError("Cannot determine agent_id from API key")
        
        url = f"{self.base_url}/api/v1/agents/{self._agent_id}/credentials"
        params = {"page": page, "limit": min(limit, 100)}
        
        try:
            response = self.session.get(url, params=params, timeout=self.timeout)
            return self._handle_response(response)
        except requests.RequestException as e:
            raise ServerError(f"Network error: {e}")

    def update_credential(
        self,
        credential_name: str,
        new_secret: str,
        description: Optional[str] = None,
    ) -> Dict[str, Any]:
        """
        Update a credential's secret value.
        
        This allows programmatic rotation of secrets. The agent can update
        its own credentials with new values.
        
        Args:
            credential_name: Name of the credential to update
            new_secret: The new secret value
            description: Optional new description
            
        Returns:
            Updated credential metadata (without the secret)
            
        Raises:
            AuthenticationError: If API key is invalid
            NotFoundError: If credential not found
            ServerError: For server errors
            
        Example:
            >>> agent.update_credential("openai-key", "sk-new-key-value")
        """
        if not self._agent_id:
            raise AgentKeyError("Cannot determine agent_id from API key")
        
        # First, find the credential by name to get its ID
        credentials = self.list_credentials(limit=100)
        credential_id = None
        for cred in credentials.get("data", []):
            if cred.get("name") == credential_name:
                credential_id = cred.get("id")
                break
        
        if not credential_id:
            raise NotFoundError(f"Credential '{credential_name}' not found")
        
        url = f"{self.base_url}/api/v1/credentials/{credential_id}"
        payload: Dict[str, Any] = {"secret": new_secret}
        if description is not None:
            payload["description"] = description
        
        try:
            response = self.session.patch(url, json=payload, timeout=self.timeout)
            return self._handle_response(response)
        except requests.RequestException as e:
            raise ServerError(f"Network error: {e}")

    def create_credential(
        self,
        name: str,
        secret: str,
        credential_type: str = "generic",
        description: Optional[str] = None,
    ) -> Dict[str, Any]:
        """
        Create a new credential for this agent.
        
        Args:
            name: Unique name for the credential
            secret: The secret value to store
            credential_type: Type of credential (generic, aws, openai, database, api_key)
            description: Optional description
            
        Returns:
            Created credential metadata
            
        Raises:
            AuthenticationError: If API key is invalid
            ServerError: For server errors
            
        Example:
            >>> agent.create_credential("new-api-key", "sk-xxx", credential_type="openai")
        """
        if not self._agent_id:
            raise AgentKeyError("Cannot determine agent_id from API key")
        
        url = f"{self.base_url}/api/v1/agents/{self._agent_id}/credentials"
        payload = {
            "name": name,
            "secret": secret,
            "credential_type": credential_type,
        }
        if description:
            payload["description"] = description
        
        try:
            response = self.session.post(url, json=payload, timeout=self.timeout)
            return self._handle_response(response)
        except requests.RequestException as e:
            raise ServerError(f"Network error: {e}")

    def delete_credential(self, credential_name: str) -> None:
        """
        Delete a credential.
        
        Args:
            credential_name: Name of the credential to delete
            
        Raises:
            NotFoundError: If credential not found
            ServerError: For server errors
        """
        if not self._agent_id:
            raise AgentKeyError("Cannot determine agent_id from API key")
        
        # Find credential by name
        credentials = self.list_credentials(limit=100)
        credential_id = None
        for cred in credentials.get("data", []):
            if cred.get("name") == credential_name:
                credential_id = cred.get("id")
                break
        
        if not credential_id:
            raise NotFoundError(f"Credential '{credential_name}' not found")
        
        url = f"{self.base_url}/api/v1/credentials/{credential_id}"
        
        try:
            response = self.session.delete(url, timeout=self.timeout)
            self._handle_response(response)
        except requests.RequestException as e:
            raise ServerError(f"Network error: {e}")

    def revoke_token(self, jti: str) -> None:
        """
        Revoke a token before expiration.
        
        Args:
            jti: Token JTI from JWT payload
            
        Raises:
            NotFoundError: If token not found
            ServerError: For server errors
        """
        url = f"{self.base_url}/api/v1/tokens/revoke"
        
        try:
            response = self.session.post(
                url,
                json={"jti": jti},
                timeout=self.timeout,
            )
            self._handle_response(response)
        except requests.RequestException as e:
            raise ServerError(f"Network error: {e}")

    def get_token_status(self, jti: str) -> Dict[str, Any]:
        """
        Get token status.
        
        Args:
            jti: Token JTI from JWT payload
            
        Returns:
            Dict with keys: jti, status, expires_at, created_at
            
        Raises:
            NotFoundError: If token not found
            ServerError: For server errors
        """
        url = f"{self.base_url}/api/v1/tokens/{jti}/status"
        
        try:
            response = self.session.get(url, timeout=self.timeout)
            return self._handle_response(response)
        except requests.RequestException as e:
            raise ServerError(f"Network error: {e}")

    def health_check(self) -> bool:
        """
        Check if AgentKey service is available.
        
        Returns:
            True if healthy
            
        Raises:
            ServerError: If health check fails
        """
        url = f"{self.base_url}/api/v1/health"
        
        try:
            response = self.session.get(url, timeout=self.timeout)
            return response.status_code == 200
        except requests.RequestException:
            return False

    def clear_cache(self, credential_name: Optional[str] = None) -> None:
        """
        Clear token cache.
        
        Args:
            credential_name: Clear specific credential, or all if None
        """
        if credential_name:
            self._token_cache.pop(credential_name, None)
        else:
            self._token_cache.clear()

    def __repr__(self) -> str:
        """String representation."""
        return f"AgentKey(base_url='{self.base_url}', agent_id='{self._agent_id}')"
