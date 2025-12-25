import api from './api';
import type { User, LoginFormData, SignupFormData } from '../types';

export const authService = {
    async login(data: LoginFormData): Promise<{ user: User; token: string }> {
        const response = await api.post('/auth/login', {
            email: data.email,
            password: data.password,
        });
        return response.data;
    },

    async signup(data: SignupFormData): Promise<{ user: User; token: string }> {
        const response = await api.post('/auth/register', {
            email: data.email,
            password: data.password,
            team_name: data.team_name,
        });
        return response.data;
    },

    async logout(): Promise<void> {
        try {
            await api.post('/auth/logout');
        } catch (e) {
            // Ignore if endpoint missing
        }
        localStorage.removeItem('auth_token');
    },

    async getCurrentUser(): Promise<User> {
        const response = await api.get('/auth/me');
        return response.data;
    },

    async refreshToken(): Promise<string> {
        const response = await api.post('/auth/refresh');
        const token = response.data.token;
        localStorage.setItem('auth_token', token);
        return token;
    },

    async resetPassword(email: string): Promise<void> {
        await api.post('/auth/reset-password', { email });
    },

    async confirmResetPassword(token: string, newPassword: string): Promise<void> {
        await api.post('/auth/reset-password/confirm', { token, password: newPassword });
    },
};

export default authService;
