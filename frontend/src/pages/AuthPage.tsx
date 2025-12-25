import LoginForm from '../components/auth/LoginForm';
import SignupForm from '../components/auth/SignupForm';
import { useLocation } from 'react-router-dom';

export default function AuthPage() {
    const location = useLocation();
    const isSignup = location.pathname === '/signup';

    // Forms now have their own complete layouts, just render them directly
    if (isSignup) {
        return <SignupForm />;
    }

    return <LoginForm />;
}
