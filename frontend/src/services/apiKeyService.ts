import api from './api';
import type { ApiKey, ApiResponse, CreateApiKeyData } from '../types';

export const apiKeyService = {
    async listAgentKeys(agentId: string): Promise<ApiKey[]> {
        const response = await api.get<ApiResponse<ApiKey[]>>(`/agents/${agentId}/keys`);
        return response.data.data;
    },

    async createAgentKey(agentId: string, data: CreateApiKeyData): Promise<{ id: string; api_key: string }> {
        const response = await api.post<ApiResponse<{ id: string; api_key: string }>>(`/agents/${agentId}/keys`, data);
        return response.data.data;
    },

    async revokeAgentKey(agentId: string, keyId: string): Promise<void> {
        await api.delete(`/agents/${agentId}/keys/${keyId}`);
    },
};
