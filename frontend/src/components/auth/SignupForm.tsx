import { useState } from 'react';
import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import { z } from 'zod';
import { useAuth } from '../../hooks/useAuth';
import type { SignupFormData } from '../../types';
import Button from '../common/Button';
import Input from '../common/Input';
import Alert from '../common/Alert';

const signupSchema = z.object({
    name: z.string().min(2, 'Name must be at least 2 characters'),
    email: z.string().email('Invalid email address'),
    password: z.string().min(8, 'Password must be at least 8 characters')
        .regex(/[A-Z]/, 'Must contain at least one uppercase letter')
        .regex(/[0-9]/, 'Must contain at least one number'),
    confirm_password: z.string(),
    terms_accepted: z.boolean().refine((val) => val === true, 'You must accept the terms'),
}).refine((data) => data.password === data.confirm_password, {
    message: "Passwords don't match",
    path: ["confirm_password"],
});

export default function SignupForm() {
    const { signup, isLoading: isAuthLoading } = useAuth();
    const [apiError, setApiError] = useState<string | null>(null);

    const {
        register,
        handleSubmit,
        formState: { errors, isSubmitting },
    } = useForm<SignupFormData>({
        resolver: zodResolver(signupSchema),
    });

    const onSubmit = async (data: SignupFormData) => {
        setApiError(null);
        try {
            await signup(data);
        } catch (error: any) {
            setApiError(error.message);
        }
    };

    const isLoading = isAuthLoading || isSubmitting;

    return (
        <form onSubmit={handleSubmit(onSubmit)} className="space-y-4">
            {apiError && <Alert variant="error">{apiError}</Alert>}

            <Input
                label="Full Name"
                type="text"
                placeholder="John Doe"
                {...register('name')}
                error={errors.name?.message}
            />

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

            <Input
                label="Confirm Password"
                type="password"
                placeholder="••••••••"
                {...register('confirm_password')}
                error={errors.confirm_password?.message}
            />

            <div className="flex items-center">
                <input
                    type="checkbox"
                    id="terms"
                    {...register('terms_accepted')}
                    className="h-4 w-4 rounded border-gray-300 text-primary focus:ring-primary"
                />
                <label htmlFor="terms" className="ml-2 text-sm text-gray-500">
                    I accept the <a href="#" className="text-primary hover:underline">Terms of Service</a>
                </label>
            </div>
            {errors.terms_accepted && (
                <p className="text-sm text-destructive">{errors.terms_accepted.message}</p>
            )}

            <Button
                type="submit"
                disabled={isLoading}
                isLoading={isLoading}
                className="w-full"
            >
                Sign up
            </Button>
        </form>
    );
}
