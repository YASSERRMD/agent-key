import { describe, it, expect, vi } from 'vitest';
import { credentialService } from '../src/services/credentialService';
import api from '../src/services/api';

vi.mock('../src/services/api', () => ({
    default: {
        get: vi.fn(),
        post: vi.fn(),
        patch: vi.fn(),
        delete: vi.fn(),
    },
}));

describe('credentialService', () => {
    it('should fetch credentials for an agent', async () => {
        const mockCredentials = [{ id: 'c1', name: 'DB Password' }];
        vi.mocked(api.get).mockResolvedValueOnce({ data: { data: mockCredentials, total: 1 } });

        const result = await credentialService.getCredentials('agent1');

        expect(api.get).toHaveBeenCalledWith('/api/v1/credentials', {
            params: { agent_id: 'agent1', page: 1, limit: 20 },
        });
        expect(result.data).toEqual(mockCredentials);
    });

    it('should create a credential', async () => {
        const mockCredential = { id: 'c2', name: 'API key 2' };
        vi.mocked(api.post).mockResolvedValueOnce({ data: mockCredential });

        const result = await credentialService.createCredential({
            name: 'API key 2',
            credential_type: 'api_key',
            secret: 'secret123'
        });

        expect(api.post).toHaveBeenCalledWith('/api/v1/credentials', {
            name: 'API key 2',
            credential_type: 'api_key',
            secret: 'secret123'
        });
        expect(result).toEqual(mockCredential);
    });

    it('should rotate a credential', async () => {
        const mockCredential = { id: 'c2', name: 'API key 2', last_rotated: '2025-01-01' };
        vi.mocked(api.post).mockResolvedValueOnce({ data: mockCredential });

        const result = await credentialService.rotateCredential('c2');

        expect(api.post).toHaveBeenCalledWith('/api/v1/credentials/c2/rotate');
        expect(result).toEqual(mockCredential);
    });
});
