import api from './api';

export interface ActivityLog {
    id: number;
    description: string;
    timestamp: string;
    status: string;
    ip_address?: string;
}

export interface DashboardStats {
    total_agents: number;
    total_credentials: number;
    api_access_count: number;
    success_rate: number;
    recent_activity: ActivityLog[];
}

export const dashboardService = {
    getStats: async (): Promise<DashboardStats> => {
        const response = await api.get<DashboardStats>('/dashboard/stats');
        return response.data;
    },
};
