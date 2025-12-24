import LoginForm from '../components/auth/LoginForm';
import SignupForm from '../components/auth/SignupForm';
import AuthLayout from '../components/auth/AuthLayout';
import { useLocation } from 'react-router-dom';

export default function AuthPage() {
    // Determine mode based on URL or local state toggle?
    // Current requirement implies separate routes /login and /signup or toggle.
    // Prompt says "src/pages/AuthPage.tsx: Login/signup toggle"
    // So likely one page component handles both logic.

    // Use state to toggle or better yet, read from router if URL is /login vs /signup
    // But since I'm implementing one page component `AuthPage`, I'll rely on URL or props.
    // Let's check App.tsx routing plan. 
    // "/login" -> AuthPage
    // "/signup" -> AuthPage? Not in router plan.
    // Plan: "- /login (public) - /signup (public)"
    // So AuthPage should detect mode.

    const location = useLocation();
    const isSignup = location.pathname === '/signup';

    if (isSignup) {
        return (
            <AuthLayout
                title="Create your account"
                subtitle="Already have an account?"
                linkText="Sign in"
                linkTo="/login"
            >
                <SignupForm />
            </AuthLayout>
        );
    }

    return (
        <AuthLayout
            title="Sign in to your account"
            subtitle="Or"
            linkText="create a new account"
            linkTo="/signup"
        >
            <LoginForm />
        </AuthLayout>
    );
}
