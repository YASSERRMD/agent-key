import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { BrowserRouter } from 'react-router-dom';
import LoginForm from '../../src/components/auth/LoginForm';

// Mock useAuth hook
vi.mock('../../src/hooks/useAuth', () => ({
    useAuth: () => ({
        login: vi.fn(),
        isLoading: false,
    }),
}));

describe('LoginForm', () => {
    const renderLoginForm = () => {
        render(
            <BrowserRouter>
                <LoginForm />
            </BrowserRouter>
        );
    };

    beforeEach(() => {
        vi.clearAllMocks();
    });

    it('renders login form with all fields', () => {
        renderLoginForm();

        expect(screen.getByLabelText(/email/i)).toBeInTheDocument();
        expect(screen.getByLabelText(/password/i)).toBeInTheDocument();
        expect(screen.getByRole('button', { name: /sign in/i })).toBeInTheDocument();
    });

    it('shows validation errors for empty fields', async () => {
        renderLoginForm();

        const submitButton = screen.getByRole('button', { name: /sign in/i });
        fireEvent.click(submitButton);

        await waitFor(() => {
            expect(screen.getByText(/invalid email/i)).toBeInTheDocument();
        });
    });

    it('shows validation error for invalid email', async () => {
        renderLoginForm();

        const emailInput = screen.getByLabelText(/email/i);
        fireEvent.change(emailInput, { target: { value: 'invalid-email' } });

        const submitButton = screen.getByRole('button', { name: /sign in/i });
        fireEvent.click(submitButton);

        await waitFor(() => {
            expect(screen.getByText(/invalid email/i)).toBeInTheDocument();
        });
    });

    it('has link to signup page', () => {
        renderLoginForm();

        const signupLink = screen.getByText(/create one/i);
        expect(signupLink).toBeInTheDocument();
        expect(signupLink).toHaveAttribute('href', '/signup');
    });

    it('has forgot password link', () => {
        renderLoginForm();

        const forgotLink = screen.getByText(/forgot password/i);
        expect(forgotLink).toBeInTheDocument();
    });

    it('displays logo', () => {
        renderLoginForm();

        const logo = screen.getByAltText(/agentkey/i);
        expect(logo).toBeInTheDocument();
    });
});
