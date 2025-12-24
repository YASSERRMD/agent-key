import { describe, it, expect, vi } from 'vitest';
import { authService } from '../src/services/authService';
import api from '../src/services/api';

vi.mock('../src/services/api', () => ({
    default: {
        post: vi.fn(),
    },
}));

describe('authService', () => {
    it('should call login api and return data', async () => {
        const mockData = { user: { id: '1', email: 'test@example.com', name: 'Test' }, token: 'abc' };
        vi.mocked(api.post).mockResolvedValueOnce({ data: mockData });

        const result = await authService.login({ email: 'test@example.com', password: 'password' });

        expect(api.post).toHaveBeenCalledWith('/api/v1/auth/login', { email: 'test@example.com', password: 'password' });
        expect(result).toEqual(mockData);
    });
});
