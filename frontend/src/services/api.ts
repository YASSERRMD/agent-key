import axios from 'axios';
import type { AxiosInstance, AxiosError } from 'axios';

// Create axios instance
const apiBaseURL = import.meta.env.VITE_API_URL || 'http://localhost:8080';

const api: AxiosInstance = axios.create({
    baseURL: apiBaseURL,
    headers: {
        'Content-Type': 'application/json',
    },
});

// Request interceptor (add auth token)
api.interceptors.request.use(
    (config) => {
        const token = localStorage.getItem('auth_token');
        if (token) {
            config.headers.Authorization = `Bearer ${token}`;
        }
        return config;
    },
    (error) => Promise.reject(error)
);

// Response interceptor (handle 401, refresh token)
api.interceptors.response.use(
    (response) => response,
    async (error: AxiosError) => {
        const originalRequest = error.config;

        // Check if error is 401 and we haven't retried yet
        // @ts-ignore
        if (error.response?.status === 401 && !originalRequest._retry) {
            // @ts-ignore
            originalRequest._retry = true;

            // Try to refresh token
            try {
                // We need separate instance or careful call to avoid infinite loop
                // Assuming refresh endpoint needs REFRESH token which might be in HttpOnly cookie?
                // Prompt says "RefreshToken()" service method.
                // If refresh token is in cookie, we just call post.
                // If refresh token is in localStorage (bad practice but possible), we send it.
                // Assuming cookie for refresh token as per typical secure setup, or we send 'Authorization' with old token?
                // Actually, let's assume standard flow: Access Token expired.

                const response = await axios.post(`${apiBaseURL}/api/v1/auth/refresh`, {}, { withCredentials: true });
                const newToken = response.data.token;

                if (newToken) {
                    localStorage.setItem('auth_token', newToken);

                    if (originalRequest) {
                        originalRequest.headers.Authorization = `Bearer ${newToken}`;
                        return api(originalRequest);
                    }
                }
            } catch (refreshError) {
                // Refresh failed, clear auth and redirect to login
                localStorage.removeItem('auth_token');
                window.location.href = '/login';
            }
        }

        return Promise.reject(error);
    }
);

export default api;
