import { useState } from 'react';
import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import { z } from 'zod';
import { useAuth } from '../../hooks/useAuth';
import { userService } from '../../services/userService';
import Button from '../common/Button';
import Card from '../common/Card';
import { User, Mail, Lock, Camera } from 'lucide-react';

const profileSchema = z.object({
    name: z.string().min(1, 'Name is required').max(100),
    email: z.string().email('Invalid email'),
});

const passwordSchema = z.object({
    current_password: z.string().min(1, 'Current password is required'),
    new_password: z.string().min(12, 'Password must be at least 12 characters'),
    confirm_password: z.string(),
}).refine((data) => data.new_password === data.confirm_password, {
    message: "Passwords don't match",
    path: ['confirm_password'],
});

type ProfileFormData = z.infer<typeof profileSchema>;
type PasswordFormData = z.infer<typeof passwordSchema>;

export default function ProfileSettings() {
    const { user } = useAuth();
    const [profileLoading, setProfileLoading] = useState(false);
    const [passwordLoading, setPasswordLoading] = useState(false);
    const [profileSuccess, setProfileSuccess] = useState(false);
    const [passwordSuccess, setPasswordSuccess] = useState(false);
    const [error, setError] = useState<string | null>(null);

    const profileForm = useForm<ProfileFormData>({
        resolver: zodResolver(profileSchema),
        defaultValues: {
            name: user?.name || '',
            email: user?.email || '',
        },
    });

    const passwordForm = useForm<PasswordFormData>({
        resolver: zodResolver(passwordSchema),
    });

    const handleProfileSubmit = async (data: ProfileFormData) => {
        try {
            setProfileLoading(true);
            setError(null);
            await userService.updateProfile(data);
            setProfileSuccess(true);
            setTimeout(() => setProfileSuccess(false), 3000);
        } catch (err: any) {
            setError(err.message || 'Failed to update profile');
        } finally {
            setProfileLoading(false);
        }
    };

    const handlePasswordSubmit = async (data: PasswordFormData) => {
        try {
            setPasswordLoading(true);
            setError(null);
            await userService.changePassword({
                current_password: data.current_password,
                new_password: data.new_password,
            });
            setPasswordSuccess(true);
            passwordForm.reset();
            setTimeout(() => setPasswordSuccess(false), 3000);
        } catch (err: any) {
            setError(err.message || 'Failed to change password');
        } finally {
            setPasswordLoading(false);
        }
    };

    return (
        <div className="space-y-6">
            {error && (
                <div className="p-4 rounded-lg bg-red-50 border border-red-200 text-red-700">
                    {error}
                </div>
            )}

            {/* Avatar Section */}
            <Card className="p-6">
                <h3 className="text-lg font-semibold mb-4">Profile Picture</h3>
                <div className="flex items-center gap-4">
                    <div className="w-20 h-20 rounded-full bg-gradient-to-br from-teal-400 to-teal-600 flex items-center justify-center text-white text-3xl font-bold shadow-lg">
                        {(user?.name || user?.email || 'U').charAt(0).toUpperCase()}
                    </div>
                    <div>
                        <p className="font-medium text-gray-900">{user?.name || user?.email?.split('@')[0]}</p>
                        <p className="text-sm text-gray-500">{user?.email}</p>
                        <p className="text-xs text-gray-400 mt-2">
                            <Camera className="inline h-3 w-3 mr-1" />
                            Avatar upload coming soon
                        </p>
                    </div>
                </div>
            </Card>

            {/* Profile Form */}
            <Card className="p-6">
                <h3 className="text-lg font-semibold mb-4">Personal Information</h3>
                <form onSubmit={profileForm.handleSubmit(handleProfileSubmit)} className="space-y-4">
                    <div>
                        <label className="block text-sm font-medium text-gray-700 mb-2">
                            <User className="inline h-4 w-4 mr-1" />
                            Full Name
                        </label>
                        <input
                            {...profileForm.register('name')}
                            className="w-full px-4 py-3 rounded-lg border border-gray-300 focus:outline-none focus:ring-2 focus:ring-teal-500"
                        />
                        {profileForm.formState.errors.name && (
                            <p className="mt-1 text-sm text-red-600">{profileForm.formState.errors.name.message}</p>
                        )}
                    </div>
                    <div>
                        <label className="block text-sm font-medium text-gray-700 mb-2">
                            <Mail className="inline h-4 w-4 mr-1" />
                            Email Address
                        </label>
                        <input
                            {...profileForm.register('email')}
                            type="email"
                            className="w-full px-4 py-3 rounded-lg border border-gray-300 focus:outline-none focus:ring-2 focus:ring-teal-500"
                        />
                        {profileForm.formState.errors.email && (
                            <p className="mt-1 text-sm text-red-600">{profileForm.formState.errors.email.message}</p>
                        )}
                    </div>
                    <div className="flex items-center gap-2">
                        <Button type="submit" isLoading={profileLoading}>
                            Save Changes
                        </Button>
                        {profileSuccess && (
                            <span className="text-sm text-green-600">Profile updated!</span>
                        )}
                    </div>
                </form>
            </Card>

            {/* Password Form */}
            <Card className="p-6">
                <h3 className="text-lg font-semibold mb-4">
                    <Lock className="inline h-5 w-5 mr-2" />
                    Change Password
                </h3>
                <form onSubmit={passwordForm.handleSubmit(handlePasswordSubmit)} className="space-y-4">
                    <div>
                        <label className="block text-sm font-medium text-gray-700 mb-2">
                            Current Password
                        </label>
                        <input
                            {...passwordForm.register('current_password')}
                            type="password"
                            className="w-full px-4 py-3 rounded-lg border border-gray-300 focus:outline-none focus:ring-2 focus:ring-teal-500"
                        />
                    </div>
                    <div>
                        <label className="block text-sm font-medium text-gray-700 mb-2">
                            New Password
                        </label>
                        <input
                            {...passwordForm.register('new_password')}
                            type="password"
                            placeholder="Min 12 characters"
                            className="w-full px-4 py-3 rounded-lg border border-gray-300 focus:outline-none focus:ring-2 focus:ring-teal-500"
                        />
                        {passwordForm.formState.errors.new_password && (
                            <p className="mt-1 text-sm text-red-600">{passwordForm.formState.errors.new_password.message}</p>
                        )}
                    </div>
                    <div>
                        <label className="block text-sm font-medium text-gray-700 mb-2">
                            Confirm New Password
                        </label>
                        <input
                            {...passwordForm.register('confirm_password')}
                            type="password"
                            className="w-full px-4 py-3 rounded-lg border border-gray-300 focus:outline-none focus:ring-2 focus:ring-teal-500"
                        />
                        {passwordForm.formState.errors.confirm_password && (
                            <p className="mt-1 text-sm text-red-600">{passwordForm.formState.errors.confirm_password.message}</p>
                        )}
                    </div>
                    <div className="flex items-center gap-2">
                        <Button type="submit" isLoading={passwordLoading}>
                            Update Password
                        </Button>
                        {passwordSuccess && (
                            <span className="text-sm text-green-600">Password changed!</span>
                        )}
                    </div>
                </form>
            </Card>
        </div>
    );
}
