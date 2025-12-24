import { describe, it, expect, vi } from 'vitest';
import { agentService } from '../src/services/agentService';
import { apiKeyService } from '../src/services/apiKeyService';
import api from '../src/services/api';

vi.mock('../src/services/api', () => ({
    default: {
        get: vi.fn(),
        post: vi.fn(),
        patch: vi.fn(),
        delete: vi.fn(),
    },
}));

describe('agentService', () => {
    it('should fetch agents list', async () => {
        const mockAgents = [{ id: '1', name: 'Agent 1' }];
        vi.mocked(api.get).mockResolvedValueOnce({ data: { data: mockAgents, meta: { total: 1 } } });

        const result = await agentService.getAgents();

        expect(api.get).toHaveBeenCalledWith('/api/v1/agents', { params: { page: 1, limit: 20 } });
        expect(result.data).toEqual(mockAgents);
    });

    it('should create a new agent', async () => {
        const mockAgent = { id: '1', name: 'New Agent' };
        const mockResponse = {
            agent: mockAgent,
            api_key: 'key123',
            warning: 'save this'
        };
        vi.mocked(api.post).mockResolvedValueOnce({ data: mockResponse });

        const result = await agentService.createAgent({ name: 'New Agent' });

        expect(api.post).toHaveBeenCalledWith('/api/v1/agents', { name: 'New Agent' });
        expect(result).toEqual(mockResponse);
    });
});

describe('apiKeyService', () => {
    it('should fetch api keys for an agent', async () => {
        const mockKeys = [{ id: 'k1', status: 'active' }];
        vi.mocked(api.get).mockResolvedValueOnce({ data: { data: mockKeys } });

        const result = await apiKeyService.listAgentKeys('agent1');

        expect(api.get).toHaveBeenCalledWith('/agents/agent1/keys');
        expect(result).toEqual(mockKeys);
    });

    it('should create an api key', async () => {
        const mockResponse = { id: 'k2', api_key: 'new-key' };
        vi.mocked(api.post).mockResolvedValueOnce({ data: { data: mockResponse } });

        const result = await apiKeyService.createAgentKey('agent1', { name: 'New Key', expires_in_days: 30 });

        expect(api.post).toHaveBeenCalledWith('/agents/agent1/keys', { name: 'New Key', expires_in_days: 30 });
        expect(result).toEqual(mockResponse);
    });

    it('should revoke an api key', async () => {
        vi.mocked(api.delete).mockResolvedValueOnce({ data: {} });

        await apiKeyService.revokeAgentKey('agent1', 'key1');

        expect(api.delete).toHaveBeenCalledWith('/agents/agent1/keys/key1');
    });
});
