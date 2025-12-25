import api from './api';
import type { Credential, CreateCredentialData, UpdateCredentialData, PaginatedResponse } from '../types';

export interface CredentialVersion {
    id: string;
    version: number;
    created_at: string;
    status: string;
}

export interface DecryptedCredential {
    secret: string;
}

export const credentialService = {
    async getCredentials(agentId?: string, page: number = 1, limit: number = 20): Promise<PaginatedResponse<Credential>> {
        const url = agentId
            ? `/agents/${agentId}/credentials`
            : '/credentials';

        const response = await api.get(url, {
            params: { page, limit },
        });
        return response.data;
    },

    async getCredential(id: string): Promise<Credential> {
        const response = await api.get(`/credentials/${id}`);
        return response.data;
    },

    async createCredential(data: CreateCredentialData): Promise<Credential> {
        if (!data.agent_id) {
            throw new Error("Agent ID is required to create a credential");
        }
        const response = await api.post(`/agents/${data.agent_id}/credentials`, data);
        return response.data;
    },

    async updateCredential(id: string, data: UpdateCredentialData): Promise<Credential> {
        const response = await api.patch(`/credentials/${id}`, data);
        return response.data;
    },

    async deleteCredential(id: string): Promise<void> {
        await api.delete(`/credentials/${id}`);
    },

    async rotateCredential(id: string): Promise<Credential> {
        const response = await api.post(`/credentials/${id}/rotate`);
        return response.data;
    },

    async getVersions(id: string): Promise<CredentialVersion[]> {
        const response = await api.get<CredentialVersion[]>(`/credentials/${id}/versions`);
        return response.data;
    },

    async decryptCredential(id: string): Promise<DecryptedCredential> {
        const response = await api.get<DecryptedCredential>(`/credentials/${id}/decrypt`);
        return response.data;
    },

    async getCredentialToken(credentialName: string, agentApiKey: string): Promise<{ token: string; expires_in: number }> {
        const response = await api.get<{ token: string; expires_in: number }>(`/tokens/${credentialName}`, {
            headers: { 'X-API-Key': agentApiKey },
        });
        return response.data;
    },
};

export default credentialService;
