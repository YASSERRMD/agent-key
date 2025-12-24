import DashboardLayout from '../components/dashboard/DashboardLayout';
import { useAuth } from '../hooks/useAuth';
import { Users, Key, ShieldCheck, Activity } from 'lucide-react';
import Card from '../components/common/Card';

export default function DashboardPage() {
    const { user } = useAuth();

    const stats = [
        { title: 'Total Agents', value: '12', icon: Users, color: 'text-blue-600', bg: 'bg-blue-100' },
        { title: 'Credentials', value: '48', icon: Key, color: 'text-teal-600', bg: 'bg-teal-100' },
        { title: 'API Access', value: '1.2k', icon: ShieldCheck, color: 'text-purple-600', bg: 'bg-purple-100' },
        { title: 'Success Rate', value: '99.9%', icon: Activity, color: 'text-green-600', bg: 'bg-green-100' },
    ];

    return (
        <DashboardLayout>
            <div className="space-y-8">
                <div>
                    <h1 className="text-3xl font-bold text-gray-900">Dashboard</h1>
                    <p className="text-gray-500 mt-1">Welcome back, {user?.name}. Here's what's happening with your agents.</p>
                </div>

                <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-6">
                    {stats.map((stat) => (
                        <Card key={stat.title} className="flex items-center p-6 bg-white border border-gray-200 shadow-sm rounded-xl">
                            <div className={cn("p-3 rounded-lg mr-4", stat.bg)}>
                                <stat.icon size={24} className={stat.color} />
                            </div>
                            <div>
                                <p className="text-sm font-medium text-gray-500">{stat.title}</p>
                                <p className="text-2xl font-bold text-gray-900">{stat.value}</p>
                            </div>
                        </Card>
                    ))}
                </div>

                <div className="grid grid-cols-1 lg:grid-cols-3 gap-8">
                    <Card className="lg:col-span-2 p-6 bg-white border border-gray-200 shadow-sm rounded-xl min-h-[400px]">
                        <h3 className="text-lg font-semibold mb-6">Recent Activity</h3>
                        <div className="space-y-6">
                            {[1, 2, 3, 4, 5].map((i) => (
                                <div key={i} className="flex items-start gap-4">
                                    <div className="h-8 w-8 rounded-full bg-gray-100 flex items-center justify-center shrink-0">
                                        <Activity size={16} className="text-gray-500" />
                                    </div>
                                    <div className="flex-1 border-b pb-4 last:border-0">
                                        <p className="text-sm font-medium">Agent "Alpha-1" accessed credential "DB_PASS"</p>
                                        <p className="text-xs text-gray-400 mt-1">2 hours ago • IP 192.168.1.45</p>
                                    </div>
                                    <div className="px-2 py-1 rounded text-[10px] font-bold uppercase bg-green-100 text-green-700">
                                        Success
                                    </div>
                                </div>
                            ))}
                        </div>
                    </Card>

                    <Card className="p-6 bg-white border border-gray-200 shadow-sm rounded-xl h-fit">
                        <h3 className="text-lg font-semibold mb-6">Quick Actions</h3>
                        <div className="space-y-3">
                            <button className="w-full text-left px-4 py-3 rounded-lg border border-gray-200 hover:border-primary hover:bg-primary/5 transition-all text-sm font-medium">
                                Create New Agent
                            </button>
                            <button className="w-full text-left px-4 py-3 rounded-lg border border-gray-200 hover:border-primary hover:bg-primary/5 transition-all text-sm font-medium">
                                Add Credential
                            </button>
                            <button className="w-full text-left px-4 py-3 rounded-lg border border-gray-200 hover:border-primary hover:bg-primary/5 transition-all text-sm font-medium">
                                Generate API Key
                            </button>
                        </div>
                    </Card>
                </div>
            </div>
        </DashboardLayout>
    );
}

// Inline helper for now as Card helper isn't made yet
function cn(...inputs: any[]) {
    return inputs.filter(Boolean).join(' ');
}
