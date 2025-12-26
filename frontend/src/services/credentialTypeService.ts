import api from './api';

export interface CredentialType {
    id: string;
    team_id: string;
    name: string;
    display_name: string;
    description?: string;
    icon?: string;
    color?: string;
    is_system: boolean;
    created_at: string;
}

export interface CreateCredentialTypeData {
    name: string;
    display_name: string;
    description?: string;
    icon?: string;
    color?: string;
}

export interface UpdateCredentialTypeData {
    display_name?: string;
    description?: string;
    icon?: string;
    color?: string;
}

export const credentialTypeService = {
    async getCredentialTypes(): Promise<CredentialType[]> {
        const response = await api.get<CredentialType[]>('/credential-types');
        return response.data;
    },

    async createCredentialType(data: CreateCredentialTypeData): Promise<CredentialType> {
        const response = await api.post<CredentialType>('/credential-types', data);
        return response.data;
    },

    async updateCredentialType(id: string, data: UpdateCredentialTypeData): Promise<CredentialType> {
        const response = await api.patch<CredentialType>(`/credential-types/${id}`, data);
        return response.data;
    },

    async deleteCredentialType(id: string): Promise<void> {
        await api.delete(`/credential-types/${id}`);
    },
};

export default credentialTypeService;
