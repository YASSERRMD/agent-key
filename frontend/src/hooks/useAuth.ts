import { useCallback } from 'react';
import { useNavigate } from 'react-router-dom';
import { useAuthStore } from '../store/authStore';
import { authService } from '../services/authService';
import type { LoginFormData, SignupFormData } from '../types';

export const useAuth = () => {
    const navigate = useNavigate();
    const { user, token, setUser, setToken, logout: storeLogout, setLoading, setError, isLoading, error } =
        useAuthStore();

    const login = useCallback(
        async (data: LoginFormData) => {
            setLoading(true);
            setError(null);
            try {
                const result = await authService.login(data);
                setUser(result.user);
                setToken(result.token);
                navigate('/');
            } catch (err: any) {
                const message = err.response?.data?.error || err.response?.data?.message || 'Login failed';
                setError(message);
                throw new Error(message);
            } finally {
                setLoading(false);
            }
        },
        [setUser, setToken, setLoading, setError, navigate]
    );

    const signup = useCallback(
        async (data: SignupFormData) => {
            setLoading(true);
            setError(null);
            try {
                const result = await authService.signup(data);
                setUser(result.user);
                setToken(result.token);
                navigate('/');
            } catch (err: any) {
                const message = err.response?.data?.error || err.response?.data?.message || 'Signup failed';
                setError(message);
                throw new Error(message);
            } finally {
                setLoading(false);
            }
        },
        [setUser, setToken, setLoading, setError, navigate]
    );

    const logout = useCallback(async () => {
        try {
            await authService.logout();
        } catch (error) {
            console.error('Logout error:', error);
        } finally {
            storeLogout();
            navigate('/login');
        }
    }, [storeLogout, navigate]);

    const isAuthenticated = !!token;

    return {
        user,
        token,
        isAuthenticated,
        isLoading,
        error,
        login,
        signup,
        logout,
    };
};
