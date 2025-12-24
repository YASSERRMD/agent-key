import api from './api';
import type { Agent, CreateAgentData, UpdateAgentData, PaginatedResponse } from '../types';

export const agentService = {
    async getAgents(page: number = 1, limit: number = 20): Promise<PaginatedResponse<Agent>> {
        const response = await api.get('/api/v1/agents', {
            params: { page, limit },
        });
        // The backend might return { data: Agent[], total, ... } or { data: { data: Agent[], ... } }
        // standardizing to match PaginatedResponse<Agent>
        return response.data;
    },

    async getAgent(id: string): Promise<Agent> {
        const response = await api.get(`/api/v1/agents/${id}`);
        return response.data;
    },

    async createAgent(data: CreateAgentData): Promise<Agent> {
        const response = await api.post('/api/v1/agents', data);
        return response.data;
    },

    async updateAgent(id: string, data: UpdateAgentData): Promise<Agent> {
        const response = await api.patch(`/api/v1/agents/${id}`, data);
        return response.data;
    },

    async deleteAgent(id: string): Promise<void> {
        await api.delete(`/api/v1/agents/${id}`);
    },
};
