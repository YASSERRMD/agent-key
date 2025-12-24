import { create } from 'zustand';
import type { Agent, CreateAgentResponse } from '../types';
import { agentService } from '../services/agentService';

interface AgentState {
    agents: Agent[];
    totalAgents: number;
    currentAgent: Agent | null;
    isLoading: boolean;
    error: string | null;

    fetchAgents: (page?: number, limit?: number) => Promise<void>;
    fetchAgent: (id: string) => Promise<void>;
    createAgent: (data: any) => Promise<CreateAgentResponse>;
    updateAgent: (id: string, data: any) => Promise<void>;
    deleteAgent: (id: string) => Promise<void>;
}

export const useAgentStore = create<AgentState>((set) => ({
    agents: [],
    totalAgents: 0,
    currentAgent: null,
    isLoading: false,
    error: null,

    fetchAgents: async (page = 1, limit = 20) => {
        set({ isLoading: true, error: null });
        try {
            const response = await agentService.getAgents(page, limit);
            set({
                agents: response.data,
                totalAgents: response.total,
                isLoading: false
            });
        } catch (error: any) {
            set({
                error: error.response?.data?.message || 'Failed to fetch agents',
                isLoading: false
            });
        }
    },

    fetchAgent: async (id: string) => {
        set({ isLoading: true, error: null });
        try {
            const agent = await agentService.getAgent(id);
            set({ currentAgent: agent, isLoading: false });
        } catch (error: any) {
            set({
                error: error.response?.data?.message || 'Failed to fetch agent',
                isLoading: false
            });
        }
    },

    createAgent: async (data: any) => {
        set({ isLoading: true, error: null });
        try {
            const response = await agentService.createAgent(data);
            set((state) => ({
                agents: [response.agent, ...state.agents],
                totalAgents: state.totalAgents + 1,
                isLoading: false
            }));
            return response;
        } catch (error: any) {
            set({
                error: error.response?.data?.message || 'Failed to create agent',
                isLoading: false
            });
            throw error;
        }
    },

    updateAgent: async (id: string, data: any) => {
        set({ isLoading: true, error: null });
        try {
            const updatedAgent = await agentService.updateAgent(id, data);
            set((state) => ({
                agents: state.agents.map(a => a.id === id ? updatedAgent : a),
                currentAgent: state.currentAgent?.id === id ? updatedAgent : state.currentAgent,
                isLoading: false
            }));
        } catch (error: any) {
            set({
                error: error.response?.data?.message || 'Failed to update agent',
                isLoading: false
            });
            throw error;
        }
    },

    deleteAgent: async (id: string) => {
        set({ isLoading: true, error: null });
        try {
            await agentService.deleteAgent(id);
            set((state) => ({
                agents: state.agents.filter(a => a.id !== id),
                totalAgents: state.totalAgents - 1,
                currentAgent: state.currentAgent?.id === id ? null : state.currentAgent,
                isLoading: false
            }));
        } catch (error: any) {
            set({
                error: error.response?.data?.message || 'Failed to delete agent',
                isLoading: false
            });
            throw error;
        }
    }
}));
