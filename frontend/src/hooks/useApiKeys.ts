import { useEffect } from 'react';
import { useApiKeyStore } from '../store/apiKeyStore';

export function useApiKeys(agentId?: string) {
    const { keys, loading, error, fetchKeys, createKey, revokeKey } = useApiKeyStore();

    useEffect(() => {
        if (agentId) {
            fetchKeys(agentId);
        }
    }, [agentId, fetchKeys]);

    return {
        keys,
        loading,
        error,
        createKey: (data: { expires_in_days?: number }) => agentId ? createKey(agentId, data) : Promise.resolve(null),
        revokeKey: (keyId: string) => agentId ? revokeKey(agentId, keyId) : Promise.resolve(),
        refresh: () => agentId ? fetchKeys(agentId) : Promise.resolve(),
    };
}
