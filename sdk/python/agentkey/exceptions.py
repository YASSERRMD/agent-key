"""AgentKey SDK exceptions."""


class AgentKeyError(Exception):
    """Base exception for AgentKey errors."""

    def __init__(self, message: str, status_code: int = 0):
        self.message = message
        self.status_code = status_code
        super().__init__(self.message)


class AuthenticationError(AgentKeyError):
    """Raised when API key is invalid or missing."""

    def __init__(self, message: str = "Authentication failed"):
        super().__init__(message, status_code=401)


class NotFoundError(AgentKeyError):
    """Raised when credential or agent not found."""

    def __init__(self, message: str = "Resource not found"):
        super().__init__(message, status_code=404)


class ServerError(AgentKeyError):
    """Raised when AgentKey service returns 5xx error."""

    def __init__(self, message: str = "Server error"):
        super().__init__(message, status_code=500)


class TokenExpiredError(AgentKeyError):
    """Raised when token is expired."""

    def __init__(self, message: str = "Token has expired"):
        super().__init__(message, status_code=401)


class RateLimitError(AgentKeyError):
    """Raised when rate limit is exceeded."""

    def __init__(self, message: str = "Rate limit exceeded", retry_after: int = 0):
        super().__init__(message, status_code=429)
        self.retry_after = retry_after
