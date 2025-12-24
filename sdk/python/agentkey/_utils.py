"""Utility functions for AgentKey SDK."""

import base64
import json
from typing import Any, Dict, Optional


def parse_jwt_payload(token: str) -> Dict[str, Any]:
    """
    Parse JWT payload without verification.
    
    Args:
        token: JWT token string
        
    Returns:
        Decoded payload as dictionary
        
    Raises:
        ValueError: If token format is invalid
    """
    parts = token.split(".")
    if len(parts) != 3:
        raise ValueError("Invalid JWT format: expected 3 parts")
    
    payload_b64 = parts[1]
    
    # Add padding if needed
    padding = 4 - len(payload_b64) % 4
    if padding != 4:
        payload_b64 += "=" * padding
    
    try:
        payload_bytes = base64.urlsafe_b64decode(payload_b64)
        return json.loads(payload_bytes.decode("utf-8"))
    except Exception as e:
        raise ValueError(f"Failed to decode JWT payload: {e}")


def extract_secret_from_token(token: str) -> str:
    """
    Extract secret from ephemeral token payload.
    
    Args:
        token: Ephemeral JWT token
        
    Returns:
        Decrypted secret string
        
    Raises:
        ValueError: If token format is invalid or secret is missing
    """
    payload = parse_jwt_payload(token)
    secret = payload.get("secret")
    if secret is None:
        raise ValueError("Token does not contain 'secret' field")
    return secret


def get_token_expiration(token: str) -> Optional[int]:
    """
    Get token expiration timestamp.
    
    Args:
        token: JWT token
        
    Returns:
        Expiration Unix timestamp or None
    """
    try:
        payload = parse_jwt_payload(token)
        return payload.get("exp")
    except ValueError:
        return None
