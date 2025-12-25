import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { BrowserRouter } from 'react-router-dom';
import SignupForm from '../../src/components/auth/SignupForm';

// Mock useAuth hook
vi.mock('../../src/hooks/useAuth', () => ({
    useAuth: () => ({
        signup: vi.fn(),
        isLoading: false,
    }),
}));

describe('SignupForm', () => {
    const renderSignupForm = () => {
        render(
            <BrowserRouter>
                <SignupForm />
            </BrowserRouter>
        );
    };

    beforeEach(() => {
        vi.clearAllMocks();
    });

    it('renders signup form with all fields', () => {
        renderSignupForm();

        expect(screen.getByLabelText(/team name/i)).toBeInTheDocument();
        expect(screen.getByLabelText(/email/i)).toBeInTheDocument();
        expect(screen.getByLabelText(/^password$/i)).toBeInTheDocument();
        expect(screen.getByLabelText(/confirm password/i)).toBeInTheDocument();
        expect(screen.getByRole('button', { name: /create account/i })).toBeInTheDocument();
    });

    it('shows validation error for short password', async () => {
        renderSignupForm();

        const passwordInput = screen.getByLabelText(/^password$/i);
        fireEvent.change(passwordInput, { target: { value: 'short' } });

        const submitButton = screen.getByRole('button', { name: /create account/i });
        fireEvent.click(submitButton);

        await waitFor(() => {
            expect(screen.getByText(/at least 12 characters/i)).toBeInTheDocument();
        });
    });

    it('shows validation error for password mismatch', async () => {
        renderSignupForm();

        const teamInput = screen.getByLabelText(/team name/i);
        const emailInput = screen.getByLabelText(/email/i);
        const passwordInput = screen.getByLabelText(/^password$/i);
        const confirmInput = screen.getByLabelText(/confirm password/i);

        fireEvent.change(teamInput, { target: { value: 'My Team' } });
        fireEvent.change(emailInput, { target: { value: 'test@example.com' } });
        fireEvent.change(passwordInput, { target: { value: 'ValidPass123!' } });
        fireEvent.change(confirmInput, { target: { value: 'DifferentPass123!' } });

        const submitButton = screen.getByRole('button', { name: /create account/i });
        fireEvent.click(submitButton);

        await waitFor(() => {
            expect(screen.getByText(/passwords don't match/i)).toBeInTheDocument();
        });
    });

    it('has link to login page', () => {
        renderSignupForm();

        const loginLink = screen.getByText(/sign in/i);
        expect(loginLink).toBeInTheDocument();
        expect(loginLink).toHaveAttribute('href', '/login');
    });

    it('displays terms and privacy links', () => {
        renderSignupForm();

        expect(screen.getByText(/terms of service/i)).toBeInTheDocument();
        expect(screen.getByText(/privacy policy/i)).toBeInTheDocument();
    });
});
