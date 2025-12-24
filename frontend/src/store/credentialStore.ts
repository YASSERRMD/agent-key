import { create } from 'zustand';
import type { Credential } from '../types';
import { credentialService } from '../services/credentialService';

interface CredentialState {
    credentials: Credential[];
    totalCredentials: number;
    currentCredential: Credential | null;
    isLoading: boolean;
    error: string | null;

    fetchCredentials: (agentId?: string, page?: number, limit?: number) => Promise<void>;
    fetchCredential: (id: string) => Promise<void>;
    createCredential: (data: any) => Promise<Credential>;
    updateCredential: (id: string, data: any) => Promise<void>;
    deleteCredential: (id: string) => Promise<void>;
    rotateCredential: (id: string) => Promise<void>;
}

export const useCredentialStore = create<CredentialState>((set) => ({
    credentials: [],
    totalCredentials: 0,
    currentCredential: null,
    isLoading: false,
    error: null,

    fetchCredentials: async (agentId, page = 1, limit = 20) => {
        set({ isLoading: true, error: null });
        try {
            const response = await credentialService.getCredentials(agentId, page, limit);
            set({
                credentials: response.data,
                totalCredentials: response.total,
                isLoading: false
            });
        } catch (error: any) {
            set({
                error: error.response?.data?.message || 'Failed to fetch credentials',
                isLoading: false
            });
        }
    },

    fetchCredential: async (id: string) => {
        set({ isLoading: true, error: null });
        try {
            const credential = await credentialService.getCredential(id);
            set({ currentCredential: credential, isLoading: false });
        } catch (error: any) {
            set({
                error: error.response?.data?.message || 'Failed to fetch credential',
                isLoading: false
            });
        }
    },

    createCredential: async (data: any) => {
        set({ isLoading: true, error: null });
        try {
            const newCredential = await credentialService.createCredential(data);
            set((state) => ({
                credentials: [newCredential, ...state.credentials],
                totalCredentials: state.totalCredentials + 1,
                isLoading: false
            }));
            return newCredential;
        } catch (error: any) {
            set({
                error: error.response?.data?.message || 'Failed to create credential',
                isLoading: false
            });
            throw error;
        }
    },

    updateCredential: async (id: string, data: any) => {
        set({ isLoading: true, error: null });
        try {
            const updatedCredential = await credentialService.updateCredential(id, data);
            set((state) => ({
                credentials: state.credentials.map(c => c.id === id ? updatedCredential : c),
                currentCredential: state.currentCredential?.id === id ? updatedCredential : state.currentCredential,
                isLoading: false
            }));
        } catch (error: any) {
            set({
                error: error.response?.data?.message || 'Failed to update credential',
                isLoading: false
            });
            throw error;
        }
    },

    deleteCredential: async (id: string) => {
        set({ isLoading: true, error: null });
        try {
            await credentialService.deleteCredential(id);
            set((state) => ({
                credentials: state.credentials.filter(c => c.id !== id),
                totalCredentials: state.totalCredentials - 1,
                currentCredential: state.currentCredential?.id === id ? null : state.currentCredential,
                isLoading: false
            }));
        } catch (error: any) {
            set({
                error: error.response?.data?.message || 'Failed to delete credential',
                isLoading: false
            });
            throw error;
        }
    },

    rotateCredential: async (id: string) => {
        set({ isLoading: true, error: null });
        try {
            const updated = await credentialService.rotateCredential(id);
            set((state) => ({
                credentials: state.credentials.map(c => c.id === id ? updated : c),
                currentCredential: state.currentCredential?.id === id ? updated : state.currentCredential,
                isLoading: false
            }));
        } catch (error: any) {
            set({
                error: error.response?.data?.message || 'Failed to rotate credential',
                isLoading: false
            });
            throw error;
        }
    }
}));
