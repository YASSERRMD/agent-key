import api from './api';
import type { ApiKey, CreateApiKeyData } from '../types';

export interface CreateApiKeyRequest {
    name: string;
    expires_in_days?: number;
}

export type { ApiKey };

export const apiKeyService = {
    // Agent-specific keys
    async listAgentKeys(agentId: string): Promise<ApiKey[]> {
        const response = await api.get<ApiKey[]>(`/agents/${agentId}/keys`);
        return response.data;
    },

    async createAgentKey(agentId: string, data: CreateApiKeyData): Promise<{ id: string; api_key: string }> {
        const response = await api.post<{ id: string; api_key: string }>(`/agents/${agentId}/keys`, data);
        return response.data;
    },

    async revokeAgentKey(agentId: string, keyId: string): Promise<void> {
        await api.delete(`/agents/${agentId}/keys/${keyId}`);
    },

    // User/Team API keys for settings
    async getApiKeys(): Promise<ApiKey[]> {
        const response = await api.get<ApiKey[]>('/api-keys');
        return response.data;
    },

    async createApiKey(data: CreateApiKeyRequest): Promise<{ id: string; api_key: string }> {
        const response = await api.post<{ id: string; api_key: string }>('/api-keys', data);
        return response.data;
    },

    async regenerateApiKey(id: string): Promise<{ api_key: string }> {
        const response = await api.post<{ api_key: string }>(`/api-keys/${id}/regenerate`);
        return response.data;
    },

    async revokeApiKey(id: string): Promise<void> {
        await api.delete(`/api-keys/${id}`);
    },
};

export default apiKeyService;
