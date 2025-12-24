import { useState } from 'react';
import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import { z } from 'zod';
import { useAuth } from '../../hooks/useAuth';
import type { LoginFormData } from '../../types';
import Button from '../common/Button';
import Input from '../common/Input';
import Alert from '../common/Alert';

const loginSchema = z.object({
    email: z.string().email('Invalid email address'),
    password: z.string().min(6, 'Password must be at least 6 characters'),
    remember_me: z.boolean().optional(),
});

export default function LoginForm() {
    const { login, isLoading: isAuthLoading } = useAuth();
    const [apiError, setApiError] = useState<string | null>(null);

    const {
        register,
        handleSubmit,
        formState: { errors, isSubmitting },
    } = useForm<LoginFormData>({
        resolver: zodResolver(loginSchema),
    });

    const onSubmit = async (data: LoginFormData) => {
        setApiError(null);
        try {
            // We use our hook which handles state update and navigation
            await login(data);
        } catch (error: any) {
            // Hook throws so we can handle it here if we want or just rely on hook's error state?
            // The hook sets global error state but also throws.
            // We can use local state for form level error if we want it to be transient.
            setApiError(error.message);
        }
    };

    const isLoading = isAuthLoading || isSubmitting;

    return (
        <form onSubmit={handleSubmit(onSubmit)} className="space-y-4">
            {apiError && <Alert variant="error">{apiError}</Alert>}

            <Input
                label="Email"
                type="email"
                placeholder="you@example.com"
                {...register('email')}
                error={errors.email?.message}
            />

            <Input
                label="Password"
                type="password"
                placeholder="••••••••"
                {...register('password')}
                error={errors.password?.message}
            />

            <div className="flex items-center">
                <input
                    type="checkbox"
                    id="remember"
                    {...register('remember_me')}
                    className="h-4 w-4 rounded border-gray-300 text-primary focus:ring-primary"
                />
                <label htmlFor="remember" className="ml-2 text-sm text-gray-500">
                    Remember me
                </label>
            </div>

            <Button
                type="submit"
                disabled={isLoading}
                isLoading={isLoading}
                className="w-full"
            >
                Sign in
            </Button>
        </form>
    );
}
