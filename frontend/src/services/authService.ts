import api from './api';
import type { User, LoginFormData, SignupFormData } from '../types';

export const authService = {
    async login(data: LoginFormData): Promise<{ user: User; token: string }> {
        const response = await api.post('/api/v1/auth/login', {
            email: data.email,
            password: data.password,
        });
        // Backend returns { success: true, user: ..., token: ... } or similar structure?
        // Based on backend handlers/auth.rs:
        // AuthResponse { token, user: UserProfile }
        return response.data;
    },

    async signup(data: SignupFormData): Promise<{ user: User; token: string }> {
        const response = await api.post('/api/v1/auth/register', {
            email: data.email,
            password: data.password,
            team_name: data.team_name,
        });
        return response.data;
    },

    async logout(): Promise<void> {
        // Logout endpoint? Checked handlers/auth.rs -> No explicit logout endpoint usually for JWT unless blocklist.
        // But implementation plan says "Implement logout".
        // If backend doesn't support it, we just clear client side.
        // However, Prompt says "logout()" service call.
        // I will call it, but if 404, we ignore.
        try {
            await api.post('/api/v1/auth/logout');
        } catch (e) {
            // Ignore if endpoint missing
        }
        localStorage.removeItem('auth_token');
    },

    async getCurrentUser(): Promise<User> {
        const response = await api.get('/api/v1/auth/me');
        return response.data;
    },

    async refreshToken(): Promise<string> {
        const response = await api.post('/api/v1/auth/refresh');
        const token = response.data.token;
        localStorage.setItem('auth_token', token);
        return token;
    },
};
