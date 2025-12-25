import { useState } from 'react';
import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import { z } from 'zod';
import { Link } from 'react-router-dom';
import { authService } from '../../services/authService';

const resetSchema = z.object({
    email: z.string().email('Invalid email address'),
});

type FormData = z.infer<typeof resetSchema>;

export default function ResetPasswordForm() {
    const [isSubmitting, setIsSubmitting] = useState(false);
    const [success, setSuccess] = useState(false);
    const [apiError, setApiError] = useState<string | null>(null);

    const {
        register,
        handleSubmit,
        formState: { errors },
    } = useForm<FormData>({
        resolver: zodResolver(resetSchema),
    });

    const onSubmit = async (data: FormData) => {
        setApiError(null);
        setIsSubmitting(true);
        try {
            await authService.resetPassword(data.email);
            setSuccess(true);
        } catch (error: any) {
            setApiError(error.message || 'Failed to send reset email. Please try again.');
        } finally {
            setIsSubmitting(false);
        }
    };

    if (success) {
        return (
            <div className="min-h-screen bg-gray-50 flex flex-col justify-center py-12 px-4 sm:px-6 lg:px-8">
                <div className="w-full max-w-md mx-auto">
                    <div className="text-center mb-8">
                        <img src="/logo.png" alt="AgentKey" className="h-16 w-16 mx-auto mb-4" />
                        <h1 className="text-2xl font-bold text-gray-900">Check your email</h1>
                        <p className="mt-2 text-gray-600">
                            We've sent a password reset link to your email address.
                        </p>
                    </div>
                    <div className="bg-white rounded-2xl shadow-lg border border-gray-200 p-8 text-center">
                        <p className="text-gray-600 mb-6">
                            Didn't receive the email? Check your spam folder or try again.
                        </p>
                        <Link
                            to="/login"
                            className="text-teal-600 hover:text-teal-500 font-medium"
                        >
                            Back to login
                        </Link>
                    </div>
                </div>
            </div>
        );
    }

    return (
        <div className="min-h-screen bg-gray-50 flex flex-col justify-center py-12 px-4 sm:px-6 lg:px-8">
            <div className="w-full max-w-md mx-auto">
                <div className="text-center mb-8">
                    <img src="/logo.png" alt="AgentKey" className="h-16 w-16 mx-auto mb-4" />
                    <h1 className="text-2xl font-bold text-gray-900">Reset your password</h1>
                    <p className="mt-2 text-gray-600">
                        Enter your email and we'll send you a reset link
                    </p>
                </div>

                <div className="bg-white rounded-2xl shadow-lg border border-gray-200 p-8">
                    {apiError && (
                        <div className="mb-6 p-4 rounded-lg bg-red-50 border border-red-200 text-red-700 text-sm">
                            {apiError}
                        </div>
                    )}

                    <form onSubmit={handleSubmit(onSubmit)} className="space-y-6">
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

                        <button
                            type="submit"
                            disabled={isSubmitting}
                            className="w-full py-3 px-4 rounded-lg bg-teal-600 text-white font-semibold hover:bg-teal-700 focus:outline-none focus:ring-2 focus:ring-teal-500 focus:ring-offset-2 disabled:opacity-50 disabled:cursor-not-allowed transition-all flex items-center justify-center"
                        >
                            {isSubmitting ? (
                                <div className="w-5 h-5 border-2 border-white/30 border-t-white rounded-full animate-spin" />
                            ) : (
                                'Send Reset Link'
                            )}
                        </button>
                    </form>
                </div>

                <p className="mt-6 text-center text-sm text-gray-600">
                    Remember your password?{' '}
                    <Link to="/login" className="font-medium text-teal-600 hover:text-teal-500">
                        Sign in
                    </Link>
                </p>
            </div>
        </div>
    );
}
