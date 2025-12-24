import React, { useState } from 'react';
import { useAuthStore } from '../store/authStore';
import DashboardLayout from '../components/dashboard/DashboardLayout';
import Card from '../components/common/Card';
import Button from '../components/common/Button';
import Input from '../components/common/Input';
import Alert from '../components/common/Alert';
import Badge from '../components/common/Badge';
import { cn } from '../lib/utils';
import { User, Shield, CreditCard, Bell, Save } from 'lucide-react';

export default function SettingsPage() {
    const { user } = useAuthStore();
    const [isLoading, setIsLoading] = useState(false);
    const [successMessage, setSuccessMessage] = useState<string | null>(null);

    const handleSaveProfile = async (e: React.FormEvent) => {
        e.preventDefault();
        setIsLoading(true);
        // Mock save
        setTimeout(() => {
            setIsLoading(false);
            setSuccessMessage('Profile updated successfully');
            setTimeout(() => setSuccessMessage(null), 3000);
        }, 1000);
    };

    return (
        <DashboardLayout>
            <div className="space-y-8">
                <div>
                    <h1 className="text-3xl font-bold text-gray-900">Settings</h1>
                    <p className="text-gray-500 mt-2">Manage your account and team preferences.</p>
                </div>

                <div className="grid grid-cols-1 lg:grid-cols-4 gap-8">
                    {/* Sidebar Nav */}
                    <div className="lg:col-span-1 space-y-1">
                        {[
                            { name: 'Profile', icon: User, current: true },
                            { name: 'Security', icon: Shield, current: false },
                            { name: 'Billing', icon: CreditCard, current: false },
                            { name: 'Notifications', icon: Bell, current: false },
                        ].map((item) => (
                            <button
                                key={item.name}
                                className={cn(
                                    "w-full flex items-center px-3 py-2 text-sm font-medium rounded-md transition-colors",
                                    item.current
                                        ? "bg-primary/10 text-primary"
                                        : "text-gray-600 hover:bg-gray-50 hover:text-gray-900"
                                )}
                            >
                                <item.icon className="mr-3 h-5 w-5 flex-shrink-0" />
                                {item.name}
                            </button>
                        ))}
                    </div>

                    {/* Main Content */}
                    <div className="lg:col-span-3 space-y-6">
                        {successMessage && (
                            <Alert variant="success">{successMessage}</Alert>
                        )}

                        <Card className="p-6">
                            <h3 className="text-lg font-semibold text-gray-900 mb-6">Profile Settings</h3>
                            <form onSubmit={handleSaveProfile} className="space-y-4">
                                <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                                    <Input
                                        label="First Name"
                                        defaultValue={user?.name.split(' ')[0]}
                                    />
                                    <Input
                                        label="Last Name"
                                        defaultValue={user?.name.split(' ').slice(1).join(' ')}
                                    />
                                </div>
                                <Input
                                    label="Email Address"
                                    type="email"
                                    defaultValue={user?.email}
                                    disabled
                                />
                                <div className="flex justify-end pt-4">
                                    <Button type="submit" isLoading={isLoading}>
                                        <Save size={16} className="mr-2" />
                                        Save Changes
                                    </Button>
                                </div>
                            </form>
                        </Card>

                        <Card className="p-6">
                            <h3 className="text-lg font-semibold text-gray-900 mb-6">Team Information</h3>
                            <div className="space-y-4">
                                <div className="flex items-center justify-between p-4 bg-gray-50 rounded-lg border border-gray-200">
                                    <div>
                                        <p className="text-sm font-semibold text-gray-900">Current Team</p>
                                        <p className="text-xs text-gray-500">Free Tier Plan</p>
                                    </div>
                                    <Badge variant="info">Free</Badge>
                                </div>
                                <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                                    <div className="p-4 rounded-lg border border-gray-100 bg-white">
                                        <p className="text-xs text-gray-400 uppercase font-bold">Agents</p>
                                        <div className="flex items-end justify-between mt-1">
                                            <p className="text-2xl font-bold">2 / 5</p>
                                            <p className="text-xs text-green-500">40% used</p>
                                        </div>
                                    </div>
                                    <div className="p-4 rounded-lg border border-gray-100 bg-white">
                                        <p className="text-xs text-gray-400 uppercase font-bold">Requests</p>
                                        <div className="flex items-end justify-between mt-1">
                                            <p className="text-2xl font-bold">1,234 / 10,000</p>
                                            <p className="text-xs text-green-500">12% used</p>
                                        </div>
                                    </div>
                                </div>
                            </div>
                        </Card>
                    </div>
                </div>
            </div>
        </DashboardLayout>
    );
}
