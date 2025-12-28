"""
Basic AgentKey Usage Example
"""

import os
from agentkey import AgentKey, NotFoundError

def main():
    # Initialize with your agent key
    # In a real app, load this from env var: os.environ.get("AGENT_KEY")
    agent_key = os.environ.get("AGENT_KEY", "ak_example_key")
    
    # Connect to local or production server
    agent = AgentKey(
        api_key=agent_key,
        base_url=os.environ.get("AGENTKEY_URL", "http://localhost:8080")
    )
    
    print(f"Connected as agent: {agent_key[:8]}...")
    
    # 1. Retrieve a secret (auto-cached)
    try:
        secret = agent.get_credential_value("database-password")
        print(f"Retrieved 'database-password': {secret[:3]}***")
    except NotFoundError:
        print("Credential 'database-password' not found.")
        
    # 2. List available credentials
    try:
        creds = agent.list_credentials()
        print(f"\nAvailable credentials ({creds['total']}):")
        for c in creds['data']:
            print(f"- {c['name']} ({c['credential_type']})")
    except Exception as e:
        print(f"Error listing credentials: {e}")

if __name__ == "__main__":
    main()
