import api from './api';
import type { Credential, CreateCredentialData, UpdateCredentialData, PaginatedResponse } from '../types';

export const credentialService = {
    async getCredentials(agentId?: string, page: number = 1, limit: number = 20): Promise<PaginatedResponse<Credential>> {
        const url = agentId
            ? `/api/v1/agents/${agentId}/credentials`
            : '/api/v1/credentials'; // Fallback if supported, but typically needs agent context

        const response = await api.get(url, {
            params: { page, limit },
        });
        return response.data;
    },

    async getCredential(id: string): Promise<Credential> {
        // This endpoint might be fine if ID is unique globally, 
        // but typically nested resources are accessed via parent.
        // Assuming /api/v1/credentials/{id} exists for direct access or we need to search.
        // Based on backend handlers, it might be scoped. 
        // Let's assume global access by ID is allowed for read/update/delete if implemented,
        // otherwise we might need agentId here too.
        // Sticking to previous implementation for ID-based ops unless proven wrong.
        const response = await api.get(`/api/v1/credentials/${id}`);
        return response.data;
    },

    async createCredential(data: CreateCredentialData): Promise<Credential> {
        if (!data.agent_id) {
            throw new Error("Agent ID is required to create a credential");
        }
        const response = await api.post(`/api/v1/agents/${data.agent_id}/credentials`, data);
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
