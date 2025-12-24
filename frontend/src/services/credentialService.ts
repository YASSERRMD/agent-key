import api from './api';
import type { Credential, CreateCredentialData, UpdateCredentialData, PaginatedResponse } from '../types';

export const credentialService = {
    async getCredentials(agentId?: string, page: number = 1, limit: number = 20): Promise<PaginatedResponse<Credential>> {
        const response = await api.get('/api/v1/credentials', {
            params: { agent_id: agentId, page, limit },
        });
        return response.data;
    },

    async getCredential(id: string): Promise<Credential> {
        const response = await api.get(`/api/v1/credentials/${id}`);
        return response.data;
    },

    async createCredential(data: CreateCredentialData): Promise<Credential> {
        const response = await api.post('/api/v1/credentials', data);
        return response.data;
    },

    async updateCredential(id: string, data: UpdateCredentialData): Promise<Credential> {
        const response = await api.patch(`/api/v1/credentials/${id}`, data);
        return response.data;
    },

    async deleteCredential(id: string): Promise<void> {
        await api.delete(`/api/v1/credentials/${id}`);
    },

    async rotateCredential(id: string): Promise<Credential> {
        const response = await api.post(`/api/v1/credentials/${id}/rotate`);
        return response.data;
    }
};
