import { useEffect } from 'react';
import { useAgentStore } from '../store/agentStore';

export const useAgents = (page: number = 1, limit: number = 20) => {
    const {
        agents,
        totalAgents,
        isLoading,
        error,
        fetchAgents,
        createAgent,
        updateAgent,
        deleteAgent
    } = useAgentStore();

    useEffect(() => {
        fetchAgents(page, limit);
    }, [page, limit, fetchAgents]);

    return {
        agents,
        totalAgents,
        isLoading,
        error,
        createAgent,
        updateAgent,
        deleteAgent,
        refresh: () => fetchAgents(page, limit)
    };
};
