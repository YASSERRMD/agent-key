import { useEffect, useState } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { useAgentStore } from '../store/agentStore';
import DashboardLayout from '../components/dashboard/DashboardLayout';
import Button from '../components/common/Button';
import Card from '../components/common/Card';
import Badge from '../components/common/Badge';
import { cn } from '../lib/utils';
import { ChevronLeft, Key, Shield, History, Activity, Edit, Trash2, Plus } from 'lucide-react';
import { format } from 'date-fns';
import { useApiKeys } from '../hooks/useApiKeys';
import ApiKeyList from '../components/api-keys/ApiKeyList';
import ApiKeyForm from '../components/api-keys/ApiKeyForm';
import Modal from '../components/common/Modal';

export default function AgentDetailPage() {
    const { id } = useParams<{ id: string }>();
    const navigate = useNavigate();
    const { currentAgent, fetchAgent, isLoading, error, deleteAgent } = useAgentStore();
    const [activeTab, setActiveTab] = useState<'credentials' | 'api-keys' | 'activity'>('credentials');
    const [showApiKeyModal, setShowApiKeyModal] = useState(false);

    const {
        keys: apiKeys,
        loading: keysLoading,
        createKey,
        revokeKey
    } = useApiKeys(id);

    useEffect(() => {
        if (id) {
            fetchAgent(id);
        }
    }, [id, fetchAgent]);

    const handleDelete = async () => {
        if (window.confirm('Are you sure you want to delete this agent?')) {
            await deleteAgent(id!);
            navigate('/agents');
        }
    };

    if (isLoading) return <DashboardLayout><div className="flex h-64 items-center justify-center">Loading...</div></DashboardLayout>;
    if (error || !currentAgent) return <DashboardLayout><div className="p-8 text-destructive">Error: {error || 'Agent not found'}</div></DashboardLayout>;

    return (
        <DashboardLayout>
            <div className="space-y-6">
                <button
                    onClick={() => navigate('/agents')}
                    className="flex items-center text-sm text-gray-500 hover:text-primary transition-colors"
                >
                    <ChevronLeft size={16} className="mr-1" />
                    Back to Agents
                </button>

                <div className="flex flex-col md:flex-row justify-between items-start md:items-center gap-4">
                    <div className="flex items-center gap-4">
                        <div className="h-16 w-16 rounded-xl bg-primary/10 flex items-center justify-center text-primary border border-primary/20">
                            <Shield size={32} />
                        </div>
                        <div>
                            <div className="flex items-center gap-2">
                                <h1 className="text-3xl font-bold text-gray-900">{currentAgent.name}</h1>
                                <Badge variant={currentAgent.status === 'active' ? 'success' : 'gray'}>
                                    {currentAgent.status}
                                </Badge>
                            </div>
                            <p className="text-gray-500 mt-1">{currentAgent.description || 'No description provided.'}</p>
                        </div>
                    </div>
                    <div className="flex gap-2 w-full md:w-auto">
                        <Button variant="secondary" className="flex-1 md:flex-none">
                            <Edit size={16} className="mr-2" />
                            Edit
                        </Button>
                        <Button variant="danger" className="flex-1 md:flex-none" onClick={handleDelete}>
                            <Trash2 size={16} className="mr-2" />
                            Delete
                        </Button>
                    </div>
                </div>

                <div className="grid grid-cols-1 lg:grid-cols-4 gap-6">
                    <Card className="p-6 bg-white border border-gray-200">
                        <h4 className="text-sm font-semibold text-gray-500 uppercase mb-4">Agent Info</h4>
                        <dl className="space-y-4">
                            <div>
                                <dt className="text-xs text-gray-400">Agent ID</dt>
                                <dd className="text-sm font-mono mt-1 break-all bg-gray-50 p-2 rounded">{currentAgent.id}</dd>
                            </div>
                            <div>
                                <dt className="text-xs text-gray-400">Created At</dt>
                                <dd className="text-sm mt-1">{format(new Date(currentAgent.created_at), 'PPP p')}</dd>
                            </div>
                            <div>
                                <dt className="text-xs text-gray-400">Total Usage</dt>
                                <dd className="text-sm mt-1">{currentAgent.usage_count} requests</dd>
                            </div>
                        </dl>
                    </Card>

                    <div className="lg:col-span-3 space-y-6">
                        <div className="border-b border-gray-200">
                            <nav className="-mb-px flex space-x-8">
                                {[
                                    { id: 'credentials', label: 'Credentials', icon: Key },
                                    { id: 'api-keys', label: 'API Keys', icon: Shield },
                                    { id: 'activity', label: 'Recent Activity', icon: History }
                                ].map((tab) => (
                                    <button
                                        key={tab.id}
                                        onClick={() => setActiveTab(tab.id as any)}
                                        className={cn(
                                            "flex items-center gap-2 py-4 px-1 border-b-2 font-medium text-sm transition-colors",
                                            activeTab === tab.id
                                                ? "border-primary text-primary"
                                                : "border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300"
                                        )}
                                    >
                                        <tab.icon size={18} />
                                        {tab.label}
                                    </button>
                                ))}
                            </nav>
                        </div>

                        <Card className="p-6 bg-white">
                            {activeTab === 'credentials' && (
                                <div className="space-y-4">
                                    <div className="flex justify-between items-center mb-4">
                                        <h3 className="text-lg font-semiboldText text-gray-900">Assigned Credentials</h3>
                                        <Button size="sm">Add Credential</Button>
                                    </div>
                                    <p className="text-sm text-gray-500 text-center py-12">No credentials assigned to this agent.</p>
                                </div>
                            )}
                            {activeTab === 'api-keys' && (
                                <div className="space-y-4">
                                    <div className="flex justify-between items-center mb-4">
                                        <h3 className="text-lg font-semibold text-gray-900">Active API Keys</h3>
                                        <Button size="sm" onClick={() => setShowApiKeyModal(true)}>
                                            <Plus size={16} className="mr-2" />
                                            Generate Key
                                        </Button>
                                    </div>
                                    <ApiKeyList
                                        keys={apiKeys}
                                        onRevoke={revokeKey}
                                        loading={keysLoading}
                                    />
                                </div>
                            )}
                            {activeTab === 'activity' && (
                                <div className="space-y-4">
                                    <h3 className="text-lg font-semibold text-gray-900 mb-4">Audit Logs</h3>
                                    <div className="flex flex-col gap-4">
                                        {[1, 2, 3].map(i => (
                                            <div key={i} className="flex gap-4 p-3 rounded-lg border border-gray-50">
                                                <Activity size={16} className="text-primary mt-1" />
                                                <div>
                                                    <p className="text-sm font-medium">Authentication successful</p>
                                                    <p className="text-xs text-gray-400">2025-12-24 23:00:00 • IP 127.0.0.1</p>
                                                </div>
                                            </div>
                                        ))}
                                    </div>
                                </div>
                            )}
                        </Card>
                    </div>
                </div>
            </div>

            <Modal
                isOpen={showApiKeyModal}
                onClose={() => setShowApiKeyModal(false)}
                title="Manage API Keys"
            >
                <ApiKeyForm
                    onCreate={createKey}
                    onClose={() => setShowApiKeyModal(false)}
                />
            </Modal>
        </DashboardLayout>
    );
}
