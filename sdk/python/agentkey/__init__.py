"""AgentKey Python SDK.

A simple SDK for agent credential management with ephemeral token support.

Example:
    >>> from agentkey import AgentKey
    >>> agent = AgentKey(api_key="ak_...")
    >>> password = agent.get_credential_value("db-password")
"""

from .client import AgentKey
from .exceptions import (
    AgentKeyError,
    AuthenticationError,
    NotFoundError,
    ServerError,
    TokenExpiredError,
)
from ._version import __version__

__all__ = [
    "AgentKey",
    "AgentKeyError",
    "AuthenticationError",
    "NotFoundError",
    "ServerError",
    "TokenExpiredError",
    "__version__",
]
