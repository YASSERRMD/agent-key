import api from './api';

export interface AuditEvent {
    id: number;
    team_id: string;
    user_id?: string;
    event_type: string;
    resource_type?: string;
    resource_id?: string;
    details?: string;
    ip_address?: string;
    created_at: string;
}

export interface AuditLogParams {
    page?: number;
    limit?: number;
    event_type?: string;
    resource_type?: string;
    start_date?: string;
    end_date?: string;
}

export const auditService = {
    getAuditLogs: async (params: AuditLogParams = {}): Promise<{ data: AuditEvent[]; total: number }> => {
        const response = await api.get<{ data: AuditEvent[]; total: number }>('/audit', { params });
        return response.data;
    },

    getAuditEvent: async (id: number): Promise<AuditEvent> => {
        const response = await api.get<AuditEvent>(`/audit/${id}`);
        return response.data;
    },
};

export default auditService;
