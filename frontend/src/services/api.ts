import axios from 'axios';
import type { AxiosInstance, AxiosError } from 'axios';

// Create axios instance with /api/v1 base path
const apiBaseURL = import.meta.env.VITE_API_URL || 'http://localhost:8080/api/v1';

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

            try {
                const response = await axios.post(`${apiBaseURL}/auth/refresh`, {}, { withCredentials: true });
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

        // Extract error message from response
        const errorMessage = (error.response?.data as any)?.message ||
            (error.response?.data as any)?.error ||
            error.message ||
            'An error occurred';

        return Promise.reject(new Error(errorMessage));
    }
);

export default api;
