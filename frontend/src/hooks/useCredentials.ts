import { useEffect } from 'react';
import { useCredentialStore } from '../store/credentialStore';

export const useCredentials = (agentId?: string, page: number = 1, limit: number = 20) => {
    const {
        credentials,
        totalCredentials,
        isLoading,
        error,
        fetchCredentials,
        createCredential,
        updateCredential,
        deleteCredential,
        rotateCredential
    } = useCredentialStore();

    useEffect(() => {
        fetchCredentials(agentId, page, limit);
    }, [agentId, page, limit, fetchCredentials]);

    return {
        credentials,
        totalCredentials,
        isLoading,
        error,
        createCredential,
        updateCredential,
        deleteCredential,
        rotateCredential,
        refresh: () => fetchCredentials(agentId, page, limit)
    };
};
