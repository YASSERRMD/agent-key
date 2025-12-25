import api from './api';
import type { User } from '../types';

export interface UpdateProfileData {
    name?: string;
    email?: string;
}

export interface ChangePasswordData {
    current_password: string;
    new_password: string;
}

export const userService = {
    getProfile: async (): Promise<User> => {
        const response = await api.get<User>('/users/me');
        return response.data;
    },

    updateProfile: async (data: UpdateProfileData): Promise<User> => {
        const response = await api.patch<User>('/users/me', data);
        return response.data;
    },

    changePassword: async (data: ChangePasswordData): Promise<void> => {
        await api.post('/users/me/password', data);
    },

    updateAvatar: async (file: File): Promise<{ avatar_url: string }> => {
        const formData = new FormData();
        formData.append('avatar', file);
        const response = await api.post<{ avatar_url: string }>('/users/me/avatar', formData, {
            headers: { 'Content-Type': 'multipart/form-data' },
        });
        return response.data;
    },

    deleteAccount: async (): Promise<void> => {
        await api.delete('/users/me');
    },
};

export default userService;
