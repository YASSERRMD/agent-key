import { useEffect } from 'react';
import { BrowserRouter as Router, Routes, Route } from 'react-router-dom';
import { useAuthStore } from './store/authStore';
import { authService } from './services/authService';
import ProtectedRoute from './components/ProtectedRoute';
import { ToastProvider } from './components/common/Toast';

// Pages
import AuthPage from '@/pages/AuthPage';
import DashboardPage from '@/pages/DashboardPage';
import AgentsPage from '@/pages/AgentsPage';
import AgentDetailPage from '@/pages/AgentDetailPage';
import CredentialsPage from '@/pages/CredentialsPage';
import CredentialDetailPage from '@/pages/CredentialDetailPage';
import SettingsPage from '@/pages/SettingsPage';
import AuditLogPage from '@/pages/AuditLogPage';
import ResetPasswordForm from '@/components/auth/ResetPasswordForm';

const NotFoundPage = () => (
  <div className="min-h-screen bg-gray-50 flex items-center justify-center">
    <div className="text-center">
      <h1 className="text-6xl font-bold text-gray-300">404</h1>
      <p className="text-xl text-gray-600 mt-4">Page not found</p>
      <a href="/" className="mt-6 inline-block text-teal-600 hover:text-teal-500">
        Go back home
      </a>
    </div>
  </div>
);

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
    <ToastProvider>
      <Router>
        <Routes>
          {/* Public Routes */}
          <Route path="/login" element={<AuthPage />} />
          <Route path="/signup" element={<AuthPage />} />
          <Route path="/reset-password" element={<ResetPasswordForm />} />

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
            path="/credentials/:id"
            element={
              <ProtectedRoute>
                <CredentialDetailPage />
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
          <Route
            path="/settings/:tab"
            element={
              <ProtectedRoute>
                <SettingsPage />
              </ProtectedRoute>
            }
          />
          <Route
            path="/audit"
            element={
              <ProtectedRoute>
                <AuditLogPage />
              </ProtectedRoute>
            }
          />

          {/* Catch-all */}
          <Route path="*" element={<NotFoundPage />} />
        </Routes>
      </Router>
    </ToastProvider>
  );
}
