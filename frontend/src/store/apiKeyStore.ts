import { create } from 'zustand';
import { apiKeyService } from '../services/apiKeyService';
import type { ApiKey } from '../types';

interface ApiKeyState {
    keys: ApiKey[];
    loading: boolean;
    error: string | null;
    fetchKeys: (agentId: string) => Promise<void>;
    createKey: (agentId: string, data: { expires_in_days?: number }) => Promise<{ id: string; api_key: string } | null>;
    revokeKey: (agentId: string, keyId: string) => Promise<void>;
}

export const useApiKeyStore = create<ApiKeyState>((set, get) => ({
    keys: [],
    loading: false,
    error: null,

    fetchKeys: async (agentId) => {
        set({ loading: true, error: null });
        try {
            const keys = await apiKeyService.listAgentKeys(agentId);
            set({ keys, loading: false });
        } catch (error: any) {
            set({ error: error.response?.data?.message || 'Failed to fetch API keys', loading: false });
        }
    },

    createKey: async (agentId, data) => {
        set({ loading: true, error: null });
        try {
            const result = await apiKeyService.createAgentKey(agentId, {
                name: 'New API Key', // Backend doesn't support name yet, but types have it
                ...data
            });
            await get().fetchKeys(agentId);
            return result;
        } catch (error: any) {
            set({ error: error.response?.data?.message || 'Failed to create API key', loading: false });
            return null;
        }
    },

    revokeKey: async (agentId, keyId) => {
        set({ loading: true, error: null });
        try {
            await apiKeyService.revokeAgentKey(agentId, keyId);
            await get().fetchKeys(agentId);
        } catch (error: any) {
            set({ error: error.response?.data?.message || 'Failed to revoke API key', loading: false });
        }
    },
}));
