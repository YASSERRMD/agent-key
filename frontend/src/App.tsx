import { useEffect } from 'react';
import { BrowserRouter as Router, Routes, Route } from 'react-router-dom';
import { useAuthStore } from './store/authStore';
import { authService } from './services/authService';
import ProtectedRoute from './components/ProtectedRoute';

// Pages
import AuthPage from '@/pages/AuthPage';
import DashboardPage from '@/pages/DashboardPage';
import AgentsPage from '@/pages/AgentsPage';
import AgentDetailPage from '@/pages/AgentDetailPage';
import CredentialsPage from '@/pages/CredentialsPage';
import SettingsPage from '@/pages/SettingsPage';

const NotFoundPage = () => <div className="p-8"><h1>404 Not Found</h1></div>;

export default function App() {
  const { setUser, setToken, setLoading } = useAuthStore();

  // Check auth on mount
  useEffect(() => {
    const checkAuth = async () => {
      const token = localStorage.getItem('auth_token');
      if (token) {
        try {
          setLoading(true);
          const user = await authService.getCurrentUser();
          setUser(user);
          setToken(token);
        } catch (error) {
          console.error('Initial auth check failed:', error);
          localStorage.removeItem('auth_token');
          setToken(null);
          setUser(null);
        } finally {
          setLoading(false);
        }
      }
    };

    checkAuth();
  }, [setUser, setToken, setLoading]);

  return (
    <Router>
      <Routes>
        {/* Public Routes */}
        <Route path="/login" element={<AuthPage />} />
        <Route path="/signup" element={<AuthPage />} />

        {/* Protected Routes */}
        <Route
          path="/"
          element={
            <ProtectedRoute>
              <DashboardPage />
            </ProtectedRoute>
          }
        />
        <Route
          path="/agents"
          element={
            <ProtectedRoute>
              <AgentsPage />
            </ProtectedRoute>
          }
        />
        <Route
          path="/agents/:id"
          element={
            <ProtectedRoute>
              <AgentDetailPage />
            </ProtectedRoute>
          }
        />
        <Route
          path="/credentials"
          element={
            <ProtectedRoute>
              <CredentialsPage />
            </ProtectedRoute>
          }
        />
        <Route
          path="/settings"
          element={
            <ProtectedRoute>
              <SettingsPage />
            </ProtectedRoute>
          }
        />

        {/* Catch-all */}
        <Route path="*" element={<NotFoundPage />} />
      </Routes>
    </Router>
  );
}
