import { useState } from 'react';
import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import { z } from 'zod';
import { useAuth } from '../../hooks/useAuth';
import { Link } from 'react-router-dom';

const signupSchema = z.object({
    email: z.string().email('Invalid email address'),
    password: z.string().min(12, 'Password must be at least 12 characters')
        .regex(/[A-Z]/, 'Must contain at least one uppercase letter')
        .regex(/[0-9]/, 'Must contain at least one number')
        .regex(/[!@#$%^&*]/, 'Must contain at least one special character'),
    confirm_password: z.string(),
    team_name: z.string().min(1, 'Team name is required').max(255),
}).refine((data) => data.password === data.confirm_password, {
    message: "Passwords don't match",
    path: ["confirm_password"],
});

type FormData = z.infer<typeof signupSchema>;

export default function SignupForm() {
    const { signup, isLoading: isAuthLoading } = useAuth();
    const [apiError, setApiError] = useState<string | null>(null);

    const {
        register,
        handleSubmit,
        formState: { errors, isSubmitting },
    } = useForm<FormData>({
        resolver: zodResolver(signupSchema),
    });

    const onSubmit = async (data: FormData) => {
        setApiError(null);
        try {
            await signup({
                email: data.email,
                password: data.password,
                team_name: data.team_name,
            });
        } catch (error: any) {
            setApiError(error.message || 'Registration failed. Please try again.');
        }
    };

    const isLoading = isAuthLoading || isSubmitting;

    return (
        <div className="min-h-screen bg-gray-50 flex flex-col justify-center py-12 px-4 sm:px-6 lg:px-8">
            <div className="w-full max-w-md mx-auto">
                {/* Logo */}
                <div className="text-center mb-8">
                    <img src="/logo.png" alt="AgentKey" className="h-16 w-16 mx-auto mb-4" />
                    <h1 className="text-2xl font-bold text-gray-900">Create your account</h1>
                    <p className="mt-2 text-gray-600">Start securing your AI agents today</p>
                </div>

                {/* Form Card */}
                <div className="bg-white rounded-2xl shadow-lg border border-gray-200 p-8">
                    {apiError && (
                        <div className="mb-6 p-4 rounded-lg bg-red-50 border border-red-200 text-red-700 text-sm">
                            {apiError}
                        </div>
                    )}

                    <form onSubmit={handleSubmit(onSubmit)} className="space-y-5">
                        <div>
                            <label htmlFor="team_name" className="block text-sm font-medium text-gray-700 mb-2">
                                Team Name
                            </label>
                            <input
                                id="team_name"
                                type="text"
                                placeholder="My Company"
                                {...register('team_name')}
                                className="w-full px-4 py-3 rounded-lg border border-gray-300 text-gray-900 placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-teal-500 focus:border-transparent transition-all"
                            />
                            {errors.team_name && (
                                <p className="mt-2 text-sm text-red-600">{errors.team_name.message}</p>
                            )}
                        </div>

                        <div>
                            <label htmlFor="email" className="block text-sm font-medium text-gray-700 mb-2">
                                Email Address
                            </label>
                            <input
                                id="email"
                                type="email"
                                autoComplete="email"
                                placeholder="you@company.com"
                                {...register('email')}
                                className="w-full px-4 py-3 rounded-lg border border-gray-300 text-gray-900 placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-teal-500 focus:border-transparent transition-all"
                            />
                            {errors.email && (
                                <p className="mt-2 text-sm text-red-600">{errors.email.message}</p>
                            )}
                        </div>

                        <div>
                            <label htmlFor="password" className="block text-sm font-medium text-gray-700 mb-2">
                                Password
                            </label>
                            <input
                                id="password"
                                type="password"
                                autoComplete="new-password"
                                placeholder="Min 12 chars, uppercase, number, special"
                                {...register('password')}
                                className="w-full px-4 py-3 rounded-lg border border-gray-300 text-gray-900 placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-teal-500 focus:border-transparent transition-all"
                            />
                            {errors.password && (
                                <p className="mt-2 text-sm text-red-600">{errors.password.message}</p>
                            )}
                        </div>

                        <div>
                            <label htmlFor="confirm_password" className="block text-sm font-medium text-gray-700 mb-2">
                                Confirm Password
                            </label>
                            <input
                                id="confirm_password"
                                type="password"
                                autoComplete="new-password"
                                placeholder="Re-enter your password"
                                {...register('confirm_password')}
                                className="w-full px-4 py-3 rounded-lg border border-gray-300 text-gray-900 placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-teal-500 focus:border-transparent transition-all"
                            />
                            {errors.confirm_password && (
                                <p className="mt-2 text-sm text-red-600">{errors.confirm_password.message}</p>
                            )}
                        </div>

                        <button
                            type="submit"
                            disabled={isLoading}
                            className="w-full py-3 px-4 rounded-lg bg-teal-600 text-white font-semibold hover:bg-teal-700 focus:outline-none focus:ring-2 focus:ring-teal-500 focus:ring-offset-2 disabled:opacity-50 disabled:cursor-not-allowed transition-all flex items-center justify-center"
                        >
                            {isLoading ? (
                                <div className="w-5 h-5 border-2 border-white/30 border-t-white rounded-full animate-spin" />
                            ) : (
                                'Create Account'
                            )}
                        </button>
                    </form>
                </div>

                {/* Footer Link */}
                <p className="mt-6 text-center text-sm text-gray-600">
                    Already have an account?{' '}
                    <Link to="/login" className="font-medium text-teal-600 hover:text-teal-500">
                        Sign in
                    </Link>
                </p>

                {/* Terms */}
                <p className="mt-4 text-center text-xs text-gray-500">
                    By creating an account, you agree to our{' '}
                    <a href="#" className="text-gray-600 hover:text-gray-900">Terms of Service</a>
                    {' '}and{' '}
                    <a href="#" className="text-gray-600 hover:text-gray-900">Privacy Policy</a>
                </p>
            </div>
        </div>
    );
}
