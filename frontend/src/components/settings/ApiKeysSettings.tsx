import { useState, useEffect } from 'react';
import Card from '../common/Card';
import Button from '../common/Button';
import Badge from '../common/Badge';
import { ConfirmDialog } from '../common/Dialog';
import { apiKeyService } from '../../services/apiKeyService';
import type { ApiKey } from '../../types';
import { Key, Copy, Trash2, Plus } from 'lucide-react';

export default function ApiKeysSettings() {
    const [apiKeys, setApiKeys] = useState<ApiKey[]>([]);
    const [loading, setLoading] = useState(true);
    const [creating, setCreating] = useState(false);
    const [newKeyName, setNewKeyName] = useState('');
    const [showCreateForm, setShowCreateForm] = useState(false);
    const [newKey, setNewKey] = useState<string | null>(null);
    const [deleteId, setDeleteId] = useState<string | null>(null);
    const [deleting, setDeleting] = useState(false);

    useEffect(() => {
        loadApiKeys();
    }, []);

    const loadApiKeys = async () => {
        try {
            const keys = await apiKeyService.getApiKeys();
            setApiKeys(keys);
        } catch (err) {
            console.error('Failed to load API keys:', err);
        } finally {
            setLoading(false);
        }
    };

    const handleCreate = async () => {
        if (!newKeyName.trim()) return;

        try {
            setCreating(true);
            const response = await apiKeyService.createApiKey({ name: newKeyName });
            setNewKey(response.api_key);
            setNewKeyName('');
            await loadApiKeys();
        } catch (err) {
            console.error('Failed to create API key:', err);
        } finally {
            setCreating(false);
        }
    };

    const handleCopy = (key: string) => {
        navigator.clipboard.writeText(key);
    };

    const handleDelete = async () => {
        if (!deleteId) return;

        try {
            setDeleting(true);
            await apiKeyService.revokeApiKey(deleteId);
            setDeleteId(null);
            await loadApiKeys();
        } catch (err) {
            console.error('Failed to revoke API key:', err);
        } finally {
            setDeleting(false);
        }
    };

    return (
        <div className="space-y-6">
            {/* New Key Display */}
            {newKey && (
                <Card className="p-6 border-green-200 bg-green-50">
                    <div className="flex items-start gap-3">
                        <Key className="h-5 w-5 text-green-600 mt-0.5" />
                        <div className="flex-1">
                            <h4 className="font-semibold text-green-800">Your new API key</h4>
                            <p className="text-sm text-green-700 mb-3">
                                Make sure to copy it now. You won't be able to see it again!
                            </p>
                            <div className="flex items-center gap-2">
                                <code className="flex-1 px-3 py-2 bg-white rounded border border-green-200 font-mono text-sm">
                                    {newKey}
                                </code>
                                <Button size="sm" onClick={() => handleCopy(newKey)}>
                                    <Copy className="h-4 w-4" />
                                </Button>
                            </div>
                        </div>
                        <button
                            onClick={() => setNewKey(null)}
                            className="text-green-600 hover:text-green-800"
                        >
                            ×
                        </button>
                    </div>
                </Card>
            )}

            {/* Create API Key */}
            <Card className="p-6">
                <div className="flex items-center justify-between mb-4">
                    <div>
                        <h3 className="text-lg font-semibold">Team API Keys (Admin)</h3>
                        <p className="text-gray-500 text-sm">Manage administrative keys with full access to your team's data.</p>
                    </div>
                    {!showCreateForm && (
                        <Button onClick={() => setShowCreateForm(true)}>
                            <Plus className="h-4 w-4 mr-2" />
                            Create Admin Key
                        </Button>
                    )}
                </div>

                <div className="mb-6 p-4 bg-blue-50 text-blue-800 rounded-lg text-sm border border-blue-100">
                    <p className="font-semibold mb-1">Looking for Agent Keys?</p>
                    <p>
                        These keys provide <strong>full administrative access</strong> to your team.
                        To create isolated keys for your agents (that can only access specific credentials),
                        go to the <strong><a href="/agents" className="underline">Agents page</a></strong>, select an agent, and manage its keys there.
                    </p>
                </div>

                {showCreateForm && (
                    <div className="mb-6 p-4 bg-gray-50 rounded-lg">
                        <label className="block text-sm font-medium text-gray-700 mb-2">
                            Key Name
                        </label>
                        <div className="flex gap-3">
                            <input
                                type="text"
                                placeholder="e.g., Production API Key"
                                value={newKeyName}
                                onChange={(e) => setNewKeyName(e.target.value)}
                                className="flex-1 px-4 py-2 rounded-lg border border-gray-300 focus:outline-none focus:ring-2 focus:ring-teal-500"
                            />
                            <Button onClick={handleCreate} isLoading={creating} disabled={!newKeyName.trim()}>
                                Generate
                            </Button>
                            <Button variant="secondary" onClick={() => setShowCreateForm(false)}>
                                Cancel
                            </Button>
                        </div>
                    </div>
                )}

                {/* API Keys List */}
                {loading ? (
                    <div className="text-center py-8 text-gray-500">Loading...</div>
                ) : apiKeys.length === 0 ? (
                    <div className="text-center py-8 text-gray-500">
                        No API keys yet. Create one to get started.
                    </div>
                ) : (
                    <div className="divide-y">
                        {apiKeys.map((key) => (
                            <div key={key.id} className="py-4 flex items-center justify-between">
                                <div className="flex items-center gap-3">
                                    <Key className="h-5 w-5 text-gray-400" />
                                    <div>
                                        <p className="font-medium">{key.name}</p>
                                        <p className="text-sm text-gray-500 font-mono">
                                            {key.key_prefix}••••••••
                                        </p>
                                    </div>
                                </div>
                                <div className="flex items-center gap-3">
                                    <Badge variant={key.status === 'active' ? 'success' : 'gray'}>
                                        {key.status}
                                    </Badge>
                                    <span className="text-sm text-gray-500">
                                        {key.last_used ? `Last used ${new Date(key.last_used).toLocaleDateString()}` : 'Never used'}
                                    </span>
                                    <button
                                        onClick={() => setDeleteId(key.id)}
                                        className="p-2 text-gray-400 hover:text-red-500 transition-colors"
                                    >
                                        <Trash2 className="h-4 w-4" />
                                    </button>
                                </div>
                            </div>
                        ))}
                    </div>
                )}
            </Card>

            <ConfirmDialog
                isOpen={!!deleteId}
                onClose={() => setDeleteId(null)}
                onConfirm={handleDelete}
                title="Revoke API Key"
                message="Are you sure you want to revoke this API key? Any applications using this key will stop working."
                confirmText="Revoke"
                variant="danger"
                loading={deleting}
            />
        </div>
    );
}
