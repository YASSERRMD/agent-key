import { Link } from 'react-router-dom';
import DashboardLayout from '../components/dashboard/DashboardLayout';
import { useAuth } from '../hooks/useAuth';
import { Users, Key, ShieldCheck, Activity, Plus, Lock, Settings, Eye, Clock, TrendingUp } from 'lucide-react';
import Card from '../components/common/Card';
import { useState, useEffect } from 'react';
import type { DashboardStats } from '../services/dashboardService';
import { dashboardService } from '../services/dashboardService';
import { cn } from '../lib/utils';

export default function DashboardPage() {
    const { user } = useAuth();
    const [stats, setStats] = useState<DashboardStats | null>(null);
    const [loading, setLoading] = useState(true);

    useEffect(() => {
        const fetchStats = async () => {
            try {
                const data = await dashboardService.getStats();
                setStats(data);
            } catch (error) {
                console.error("Failed to fetch dashboard stats", error);
            } finally {
                setLoading(false);
            }
        };

        fetchStats();
    }, []);

    const statCards = [
        { title: 'Total Agents', value: stats?.total_agents?.toString() || '0', icon: Users, color: 'text-blue-600', bg: 'bg-blue-100' },
        { title: 'Credentials', value: stats?.total_credentials?.toString() || '0', icon: Key, color: 'text-teal-600', bg: 'bg-teal-100' },
        { title: 'Token Retrievals', value: stats?.api_access_count?.toString() || '0', icon: Eye, color: 'text-purple-600', bg: 'bg-purple-100', subtitle: 'Last 30 days' },
        { title: 'Success Rate', value: `${stats?.success_rate || 99.9}%`, icon: TrendingUp, color: 'text-green-600', bg: 'bg-green-100' },
    ];

    const quickActions = [
        { label: 'Create New Agent', icon: Plus, href: '/agents?create=true' },
        { label: 'Add Credential', icon: Lock, href: '/credentials?create=true' },
        { label: 'Manage API Keys', icon: Key, href: '/settings/api-keys' },
        { label: 'View Settings', icon: Settings, href: '/settings' },
    ];

    if (loading) {
        return (
            <DashboardLayout>
                <div className="flex items-center justify-center h-full">
                    <p className="text-gray-500">Loading dashboard...</p>
                </div>
            </DashboardLayout>
        );
    }

    return (
        <DashboardLayout>
            <div className="space-y-8">
                <div>
                    <h1 className="text-3xl font-bold text-gray-900">Dashboard</h1>
                    <p className="text-gray-500 mt-1">Welcome back, {user?.name || user?.email}. Here's what's happening with your agents.</p>
                </div>

                <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-6">
                    {statCards.map((stat) => (
                        <Card key={stat.title} className="flex items-center p-6 bg-white border border-gray-200 shadow-sm rounded-xl">
                            <div className={cn("p-3 rounded-lg mr-4", stat.bg)}>
                                <stat.icon size={24} className={stat.color} />
                            </div>
                            <div>
                                <p className="text-sm font-medium text-gray-500">{stat.title}</p>
                                <p className="text-2xl font-bold text-gray-900">{stat.value}</p>
                                {(stat as any).subtitle && (
                                    <p className="text-xs text-gray-400">{(stat as any).subtitle}</p>
                                )}
                            </div>
                        </Card>
                    ))}
                </div>

                {/* SDK Usage Section */}
                <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
                    <Card className="p-6 bg-gradient-to-br from-purple-50 to-blue-50 border border-purple-100">
                        <div className="flex items-center justify-between mb-4">
                            <h3 className="text-lg font-semibold text-gray-900">SDK Token Usage</h3>
                            <div className="p-2 bg-purple-100 rounded-lg">
                                <Eye className="h-5 w-5 text-purple-600" />
                            </div>
                        </div>
                        <div className="space-y-4">
                            <div className="flex justify-between items-center">
                                <span className="text-sm text-gray-600">Ephemeral tokens generated</span>
                                <span className="text-lg font-bold text-purple-600">{stats?.api_access_count || 0}</span>
                            </div>
                            <div className="flex justify-between items-center">
                                <span className="text-sm text-gray-600">Active agents using SDK</span>
                                <span className="text-lg font-bold text-blue-600">{stats?.total_agents || 0}</span>
                            </div>
                            <div className="flex justify-between items-center">
                                <span className="text-sm text-gray-600">Credentials accessible</span>
                                <span className="text-lg font-bold text-teal-600">{stats?.total_credentials || 0}</span>
                            </div>
                            <div className="pt-4 border-t border-purple-200">
                                <p className="text-xs text-gray-500">
                                    <Clock className="inline h-3 w-3 mr-1" />
                                    Tokens are short-lived (5 min default) for security
                                </p>
                            </div>
                        </div>
                    </Card>

                    <Card className="p-6 bg-gradient-to-br from-green-50 to-teal-50 border border-green-100">
                        <div className="flex items-center justify-between mb-4">
                            <h3 className="text-lg font-semibold text-gray-900">Security Status</h3>
                            <div className="p-2 bg-green-100 rounded-lg">
                                <ShieldCheck className="h-5 w-5 text-green-600" />
                            </div>
                        </div>
                        <div className="space-y-4">
                            <div className="flex justify-between items-center">
                                <span className="text-sm text-gray-600">Encryption</span>
                                <span className="text-sm font-medium text-green-600 flex items-center gap-1">
                                    <span className="w-2 h-2 rounded-full bg-green-500"></span>
                                    AES-256-GCM
                                </span>
                            </div>
                            <div className="flex justify-between items-center">
                                <span className="text-sm text-gray-600">Token authentication</span>
                                <span className="text-sm font-medium text-green-600 flex items-center gap-1">
                                    <span className="w-2 h-2 rounded-full bg-green-500"></span>
                                    JWT Signed
                                </span>
                            </div>
                            <div className="flex justify-between items-center">
                                <span className="text-sm text-gray-600">Audit logging</span>
                                <span className="text-sm font-medium text-green-600 flex items-center gap-1">
                                    <span className="w-2 h-2 rounded-full bg-green-500"></span>
                                    Enabled
                                </span>
                            </div>
                            <div className="pt-4 border-t border-green-200">
                                <Link to="/audit" className="text-sm text-teal-600 hover:text-teal-700 font-medium">
                                    View audit log →
                                </Link>
                            </div>
                        </div>
                    </Card>
                </div>

                <div className="grid grid-cols-1 lg:grid-cols-3 gap-8">
                    <Card className="lg:col-span-2 p-6 bg-white border border-gray-200 shadow-sm rounded-xl min-h-[400px]">
                        <div className="flex items-center justify-between mb-6">
                            <h3 className="text-lg font-semibold">Recent Activity</h3>
                            <Link to="/audit" className="text-sm text-teal-600 hover:text-teal-500">
                                View all
                            </Link>
                        </div>
                        <div className="space-y-6">
                            {!stats?.recent_activity || stats.recent_activity.length === 0 ? (
                                <div className="text-center py-8">
                                    <Activity className="h-12 w-12 text-gray-300 mx-auto mb-3" />
                                    <p className="text-gray-500 text-sm">No recent activity yet.</p>
                                    <p className="text-gray-400 text-xs mt-1">Activity will appear here as you use the platform.</p>
                                </div>
                            ) : (
                                stats.recent_activity.map((activity) => (
                                    <div key={activity.id} className="flex items-start gap-4">
                                        <div className={cn(
                                            "h-8 w-8 rounded-full flex items-center justify-center shrink-0",
                                            activity.description.includes('token') || activity.description.includes('credential')
                                                ? 'bg-purple-100'
                                                : activity.description.includes('agent')
                                                    ? 'bg-blue-100'
                                                    : 'bg-gray-100'
                                        )}>
                                            {activity.description.includes('token') || activity.description.includes('credential') ? (
                                                <Key size={16} className="text-purple-500" />
                                            ) : activity.description.includes('agent') ? (
                                                <Users size={16} className="text-blue-500" />
                                            ) : (
                                                <Activity size={16} className="text-gray-500" />
                                            )}
                                        </div>
                                        <div className="flex-1 border-b pb-4 last:border-0">
                                            <p className="text-sm font-medium">{activity.description}</p>
                                            <p className="text-xs text-gray-400 mt-1">
                                                {new Date(activity.timestamp).toLocaleString()}
                                                {activity.ip_address && ` • IP ${activity.ip_address}`}
                                            </p>
                                        </div>
                                        <div className={cn(
                                            "px-2 py-1 rounded text-[10px] font-bold uppercase",
                                            activity.status === 'success' ? 'bg-green-100 text-green-700' : 'bg-gray-100 text-gray-700'
                                        )}>
                                            {activity.status}
                                        </div>
                                    </div>
                                ))
                            )}
                        </div>
                    </Card>

                    <Card className="p-6 bg-white border border-gray-200 shadow-sm rounded-xl h-fit">
                        <h3 className="text-lg font-semibold mb-6">Quick Actions</h3>
                        <div className="space-y-3">
                            {quickActions.map((action) => (
                                <Link
                                    key={action.label}
                                    to={action.href}
                                    className="flex items-center gap-3 w-full text-left px-4 py-3 rounded-lg border border-gray-200 hover:border-teal-500 hover:bg-teal-50 transition-all text-sm font-medium"
                                >
                                    <action.icon className="h-4 w-4 text-teal-600" />
                                    {action.label}
                                </Link>
                            ))}
                        </div>
                    </Card>
                </div>
            </div>
        </DashboardLayout>
    );
}
