import { describe, it, expect, vi, beforeEach } from 'vitest';
import { agentService } from '../../src/services/agentService';
import api from '../../src/services/api';

vi.mock('../../src/services/api');

describe('agentService', () => {
    beforeEach(() => {
        vi.clearAllMocks();
    });

    describe('getAgents', () => {
        it('fetches agents with default pagination', async () => {
            const mockData = { data: [{ id: '1', name: 'Agent 1' }], total: 1, page: 1, limit: 20 };
            vi.mocked(api.get).mockResolvedValue({ data: mockData });

            const result = await agentService.getAgents();

            expect(api.get).toHaveBeenCalledWith('/agents', { params: { page: 1, limit: 20 } });
            expect(result).toEqual(mockData);
        });

        it('fetches agents with custom pagination', async () => {
            const mockData = { data: [], total: 0, page: 2, limit: 10 };
            vi.mocked(api.get).mockResolvedValue({ data: mockData });

            await agentService.getAgents(2, 10);

            expect(api.get).toHaveBeenCalledWith('/agents', { params: { page: 2, limit: 10 } });
        });
    });

    describe('getAgent', () => {
        it('fetches a single agent by id', async () => {
            const mockAgent = { id: '123', name: 'Test Agent' };
            vi.mocked(api.get).mockResolvedValue({ data: mockAgent });

            const result = await agentService.getAgent('123');

            expect(api.get).toHaveBeenCalledWith('/agents/123');
            expect(result).toEqual(mockAgent);
        });
    });

    describe('createAgent', () => {
        it('creates a new agent', async () => {
            const newAgent = { name: 'New Agent', description: 'Description' };
            const mockResponse = { id: '456', ...newAgent };
            vi.mocked(api.post).mockResolvedValue({ data: mockResponse });

            const result = await agentService.createAgent(newAgent);

            expect(api.post).toHaveBeenCalledWith('/agents', newAgent);
            expect(result).toEqual(mockResponse);
        });
    });

    describe('updateAgent', () => {
        it('updates an existing agent', async () => {
            const updateData = { description: 'Updated description' };
            const mockResponse = { id: '123', name: 'Agent', ...updateData };
            vi.mocked(api.patch).mockResolvedValue({ data: mockResponse });

            const result = await agentService.updateAgent('123', updateData);

            expect(api.patch).toHaveBeenCalledWith('/agents/123', updateData);
            expect(result).toEqual(mockResponse);
        });
    });

    describe('deleteAgent', () => {
        it('deletes an agent', async () => {
            vi.mocked(api.delete).mockResolvedValue({ data: {} });

            await agentService.deleteAgent('123');

            expect(api.delete).toHaveBeenCalledWith('/agents/123');
        });
    });
});
