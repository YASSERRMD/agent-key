import { useEffect, useState } from 'react';
import { useParams, useNavigate, Link } from 'react-router-dom';
import { useAgentStore } from '../store/agentStore';
import { useCredentials } from '../hooks/useCredentials';
import DashboardLayout from '../components/dashboard/DashboardLayout';
import Button from '../components/common/Button';
import Card from '../components/common/Card';
import Badge from '../components/common/Badge';
import Modal from '../components/common/Modal';
import CredentialForm from '../components/credentials/CredentialForm';
import { cn } from '../lib/utils';
import { ChevronLeft, Key, Shield, History, Activity, Edit, Trash2, Plus, Lock, Eye, RefreshCw } from 'lucide-react';
import { format } from 'date-fns';
import { useApiKeys } from '../hooks/useApiKeys';
import ApiKeyList from '../components/api-keys/ApiKeyList';
import ApiKeyForm from '../components/api-keys/ApiKeyForm';
import type { Credential } from '../types';

export default function AgentDetailPage() {
    const { id } = useParams<{ id: string }>();
    const navigate = useNavigate();
    const { currentAgent, fetchAgent, isLoading, error, deleteAgent } = useAgentStore();
    const [activeTab, setActiveTab] = useState<'credentials' | 'api-keys' | 'activity'>('credentials');
    const [showApiKeyModal, setShowApiKeyModal] = useState(false);
    const [showCredentialModal, setShowCredentialModal] = useState(false);
    const [editingCredential, setEditingCredential] = useState<Credential | null>(null);
    const [isProcessing, setIsProcessing] = useState(false);

    // Use the credentials hook filtered by this agent
    const {
        credentials,
        isLoading: credentialsLoading,
        createCredential,
        updateCredential,
        deleteCredential,
        rotateCredential
    } = useCredentials(id);

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

    const handleCreateCredential = () => {
        setEditingCredential(null);
        setShowCredentialModal(true);
    };

    const handleEditCredential = (cred: Credential) => {
        setEditingCredential(cred);
        setShowCredentialModal(true);
    };

    const handleCredentialSubmit = async (data: any) => {
        try {
            setIsProcessing(true);
            if (editingCredential) {
                await updateCredential(editingCredential.id, data);
            } else {
                await createCredential({ ...data, agent_id: id });
            }
            setShowCredentialModal(false);
            setEditingCredential(null);
        } finally {
            setIsProcessing(false);
        }
    };

    const handleDeleteCredential = async (credId: string) => {
        if (window.confirm('Are you sure you want to delete this credential? This action cannot be undone.')) {
            try {
                setIsProcessing(true);
                await deleteCredential(credId);
            } finally {
                setIsProcessing(false);
            }
        }
    };

    const handleRotateCredential = async (credId: string) => {
        if (window.confirm('Rotate this credential? The old value will be replaced with a new one.')) {
            try {
                setIsProcessing(true);
                await rotateCredential(credId);
            } finally {
                setIsProcessing(false);
            }
        }
    };

    const handleDeleteAgent = async () => {
        if (window.confirm('Are you sure you want to delete this agent? All associated credentials and API keys will also be deleted.')) {
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
                        <Button variant="danger" className="flex-1 md:flex-none" onClick={handleDeleteAgent}>
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
                            <div>
                                <dt className="text-xs text-gray-400">Credentials</dt>
                                <dd className="text-sm mt-1">{credentials.length} assigned</dd>
                            </div>
                        </dl>
                    </Card>

                    <div className="lg:col-span-3 space-y-6">
                        <div className="border-b border-gray-200">
                            <nav className="-mb-px flex space-x-8">
                                {[
                                    { id: 'credentials', label: 'Credentials', icon: Key, count: credentials.length },
                                    { id: 'api-keys', label: 'API Keys', icon: Shield, count: apiKeys.length },
                                    { id: 'activity', label: 'Activity', icon: History }
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
                                        {tab.count !== undefined && (
                                            <span className="ml-1 px-2 py-0.5 text-xs rounded-full bg-gray-100">
                                                {tab.count}
                                            </span>
                                        )}
                                    </button>
                                ))}
                            </nav>
                        </div>

                        <Card className="p-6 bg-white">
                            {activeTab === 'credentials' && (
                                <div className="space-y-4">
                                    <div className="flex justify-between items-center mb-4">
                                        <h3 className="text-lg font-semibold text-gray-900">Agent Credentials</h3>
                                        <Button size="sm" onClick={handleCreateCredential}>
                                            <Plus size={16} className="mr-2" />
                                            Add Credential
                                        </Button>
                                    </div>
                                    {credentialsLoading ? (
                                        <p className="text-sm text-gray-500 text-center py-8">Loading credentials...</p>
                                    ) : credentials.length === 0 ? (
                                        <div className="text-center py-12">
                                            <Lock className="h-12 w-12 text-gray-300 mx-auto mb-3" />
                                            <p className="text-sm text-gray-500">No credentials assigned to this agent.</p>
                                            <p className="text-xs text-gray-400 mt-1">Click "Add Credential" to create one.</p>
                                        </div>
                                    ) : (
                                        <div className="divide-y">
                                            {credentials.map((cred) => (
                                                <div key={cred.id} className="py-4 flex items-center justify-between">
                                                    <div className="flex items-center gap-3">
                                                        <div className="p-2 bg-teal-50 rounded-lg">
                                                            <Key className="h-5 w-5 text-teal-600" />
                                                        </div>
                                                        <div>
                                                            <p className="font-medium">{cred.name}</p>
                                                            <div className="flex items-center gap-2 mt-1">
                                                                <span className="text-xs text-gray-500 bg-gray-100 px-2 py-0.5 rounded">
                                                                    {cred.credential_type}
                                                                </span>
                                                                {cred.description && (
                                                                    <span className="text-xs text-gray-400">
                                                                        {cred.description}
                                                                    </span>
                                                                )}
                                                            </div>
                                                        </div>
                                                    </div>
                                                    <div className="flex items-center gap-2">
                                                        <Badge variant={cred.is_active ? 'success' : 'gray'}>
                                                            {cred.is_active ? 'Active' : 'Inactive'}
                                                        </Badge>
                                                        <Button
                                                            variant="ghost"
                                                            size="sm"
                                                            onClick={() => handleRotateCredential(cred.id)}
                                                            title="Rotate credential"
                                                        >
                                                            <RefreshCw className="h-4 w-4" />
                                                        </Button>
                                                        <Button
                                                            variant="ghost"
                                                            size="sm"
                                                            onClick={() => handleEditCredential(cred)}
                                                            title="Edit credential"
                                                        >
                                                            <Edit className="h-4 w-4" />
                                                        </Button>
                                                        <Link to={`/credentials/${cred.id}`}>
                                                            <Button variant="ghost" size="sm" title="View details">
                                                                <Eye className="h-4 w-4" />
                                                            </Button>
                                                        </Link>
                                                        <Button
                                                            variant="ghost"
                                                            size="sm"
                                                            onClick={() => handleDeleteCredential(cred.id)}
                                                            title="Delete credential"
                                                            className="text-red-500 hover:text-red-600"
                                                        >
                                                            <Trash2 className="h-4 w-4" />
                                                        </Button>
                                                    </div>
                                                </div>
                                            ))}
                                        </div>
                                    )}
                                </div>
                            )}
                            {activeTab === 'api-keys' && (
                                <div className="space-y-4">
                                    <div className="flex justify-between items-center mb-4">
                                        <h3 className="text-lg font-semibold text-gray-900">Agent API Keys</h3>
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
                                    <h3 className="text-lg font-semibold text-gray-900 mb-4">Recent Activity</h3>
                                    <div className="text-center py-12">
                                        <Activity className="h-12 w-12 text-gray-300 mx-auto mb-3" />
                                        <p className="text-sm text-gray-500">No activity recorded yet.</p>
                                        <p className="text-xs text-gray-400 mt-1">Activity will appear here when the agent accesses credentials.</p>
                                    </div>
                                </div>
                            )}
                        </Card>
                    </div>
                </div>
            </div>

            {/* API Key Modal */}
            <Modal
                isOpen={showApiKeyModal}
                onClose={() => setShowApiKeyModal(false)}
                title="Generate API Key"
            >
                <ApiKeyForm
                    onCreate={createKey}
                    onClose={() => setShowApiKeyModal(false)}
                />
            </Modal>

            {/* Credential Modal */}
            <Modal
                isOpen={showCredentialModal}
                onClose={() => { setShowCredentialModal(false); setEditingCredential(null); }}
                title={editingCredential ? 'Edit Credential' : 'Add Credential'}
            >
                <CredentialForm
                    onSubmit={handleCredentialSubmit}
                    initialData={editingCredential}
                    onCancel={() => { setShowCredentialModal(false); setEditingCredential(null); }}
                    isLoading={isProcessing}
                    agentId={id}
                />
            </Modal>
        </DashboardLayout>
    );
}
