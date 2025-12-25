import { useState, useEffect } from 'react';
import { useParams, Link } from 'react-router-dom';
import DashboardLayout from '../components/dashboard/DashboardLayout';
import { credentialService } from '../services/credentialService';
import type { Credential } from '../types';
import { ArrowLeft, Eye, EyeOff, Copy, RefreshCw, History, Shield } from 'lucide-react';
import Button from '../components/common/Button';
import Badge from '../components/common/Badge';
import Card from '../components/common/Card';
import Spinner from '../components/common/Spinner';
import { ConfirmDialog } from '../components/common/Dialog';

interface CredentialVersion {
    id: string;
    version: number;
    created_at: string;
    status: string;
}

export default function CredentialDetailPage() {
    const { id } = useParams<{ id: string }>();
    const [credential, setCredential] = useState<Credential | null>(null);
    const [versions, setVersions] = useState<CredentialVersion[]>([]);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState<string | null>(null);
    const [showSecret, setShowSecret] = useState(false);
    const [secret, setSecret] = useState<string | null>(null);
    const [decrypting, setDecrypting] = useState(false);
    const [showRotateDialog, setShowRotateDialog] = useState(false);
    const [rotating, setRotating] = useState(false);

    useEffect(() => {
        if (id) {
            loadCredential();
            loadVersions();
        }
    }, [id]);

    const loadCredential = async () => {
        try {
            setLoading(true);
            const data = await credentialService.getCredential(id!);
            setCredential(data);
        } catch (err: any) {
            setError(err.message || 'Failed to load credential');
        } finally {
            setLoading(false);
        }
    };

    const loadVersions = async () => {
        try {
            const data = await credentialService.getVersions(id!);
            setVersions(data);
        } catch (err) {
            console.error('Failed to load versions:', err);
        }
    };

    const handleReveal = async () => {
        if (showSecret) {
            setShowSecret(false);
            setSecret(null);
            return;
        }

        try {
            setDecrypting(true);
            const data = await credentialService.decryptCredential(id!);
            setSecret(data.secret);
            setShowSecret(true);
        } catch (err: any) {
            setError(err.message || 'Failed to decrypt credential');
        } finally {
            setDecrypting(false);
        }
    };

    const handleCopy = () => {
        if (secret) {
            navigator.clipboard.writeText(secret);
        }
    };

    const handleRotate = async () => {
        try {
            setRotating(true);
            await credentialService.rotateCredential(id!);
            setShowRotateDialog(false);
            await loadCredential();
            await loadVersions();
            setSecret(null);
            setShowSecret(false);
        } catch (err: any) {
            setError(err.message || 'Failed to rotate credential');
        } finally {
            setRotating(false);
        }
    };

    if (loading) {
        return (
            <DashboardLayout>
                <div className="flex items-center justify-center h-64">
                    <Spinner size="lg" />
                </div>
            </DashboardLayout>
        );
    }

    if (error || !credential) {
        return (
            <DashboardLayout>
                <div className="text-center py-12">
                    <p className="text-red-600 mb-4">{error || 'Credential not found'}</p>
                    <Link to="/credentials" className="text-teal-600 hover:text-teal-500">
                        Back to credentials
                    </Link>
                </div>
            </DashboardLayout>
        );
    }

    return (
        <DashboardLayout>
            <div className="space-y-6">
                {/* Header */}
                <div className="flex items-center gap-4">
                    <Link
                        to="/credentials"
                        className="p-2 rounded-lg hover:bg-gray-100 transition-colors"
                    >
                        <ArrowLeft className="h-5 w-5" />
                    </Link>
                    <div className="flex-1">
                        <h1 className="text-2xl font-bold text-gray-900">{credential.name}</h1>
                        <p className="text-gray-500">{credential.description || 'No description'}</p>
                    </div>
                    <Badge variant={credential.is_active ? 'success' : 'gray'}>
                        {credential.is_active ? 'Active' : 'Inactive'}
                    </Badge>
                </div>

                <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
                    {/* Main Info */}
                    <div className="lg:col-span-2 space-y-6">
                        {/* Secret Card */}
                        <Card className="p-6">
                            <div className="flex items-center justify-between mb-4">
                                <h3 className="text-lg font-semibold">Secret Value</h3>
                                <div className="flex items-center gap-2">
                                    <Button
                                        variant="secondary"
                                        size="sm"
                                        onClick={handleReveal}
                                        isLoading={decrypting}
                                    >
                                        {showSecret ? <EyeOff className="h-4 w-4 mr-1" /> : <Eye className="h-4 w-4 mr-1" />}
                                        {showSecret ? 'Hide' : 'Reveal'}
                                    </Button>
                                    {showSecret && (
                                        <Button variant="secondary" size="sm" onClick={handleCopy}>
                                            <Copy className="h-4 w-4 mr-1" />
                                            Copy
                                        </Button>
                                    )}
                                </div>
                            </div>
                            <div className="bg-gray-100 rounded-lg p-4 font-mono text-sm">
                                {showSecret && secret ? secret : '••••••••••••••••••••••••••••••••'}
                            </div>
                            <p className="mt-2 text-xs text-gray-500">
                                <Shield className="inline h-3 w-3 mr-1" />
                                Encrypted with AES-256-GCM
                            </p>
                        </Card>

                        {/* Metadata */}
                        <Card className="p-6">
                            <h3 className="text-lg font-semibold mb-4">Details</h3>
                            <dl className="grid grid-cols-2 gap-4">
                                <div>
                                    <dt className="text-sm text-gray-500">Type</dt>
                                    <dd className="text-sm font-medium">{credential.credential_type}</dd>
                                </div>
                                <div>
                                    <dt className="text-sm text-gray-500">Created</dt>
                                    <dd className="text-sm font-medium">
                                        {new Date(credential.created_at).toLocaleDateString()}
                                    </dd>
                                </div>
                                <div>
                                    <dt className="text-sm text-gray-500">Last Accessed</dt>
                                    <dd className="text-sm font-medium">
                                        {credential.last_accessed
                                            ? new Date(credential.last_accessed).toLocaleString()
                                            : 'Never'}
                                    </dd>
                                </div>
                                <div>
                                    <dt className="text-sm text-gray-500">Agent ID</dt>
                                    <dd className="text-sm font-medium font-mono">{credential.agent_id}</dd>
                                </div>
                            </dl>
                        </Card>

                        {/* Version History */}
                        <Card className="p-6">
                            <div className="flex items-center gap-2 mb-4">
                                <History className="h-5 w-5 text-gray-500" />
                                <h3 className="text-lg font-semibold">Version History</h3>
                            </div>
                            {versions.length === 0 ? (
                                <p className="text-gray-500 text-sm">No version history available</p>
                            ) : (
                                <div className="space-y-3">
                                    {versions.map((version) => (
                                        <div
                                            key={version.id}
                                            className="flex items-center justify-between py-2 border-b last:border-0"
                                        >
                                            <div>
                                                <span className="font-medium">Version {version.version}</span>
                                                <Badge variant={version.status === 'active' ? 'success' : 'gray'} className="ml-2">
                                                    {version.status}
                                                </Badge>
                                            </div>
                                            <span className="text-sm text-gray-500">
                                                {new Date(version.created_at).toLocaleString()}
                                            </span>
                                        </div>
                                    ))}
                                </div>
                            )}
                        </Card>
                    </div>

                    {/* Sidebar */}
                    <div className="space-y-6">
                        {/* Rotation Settings */}
                        <Card className="p-6">
                            <div className="flex items-center gap-2 mb-4">
                                <RefreshCw className="h-5 w-5 text-gray-500" />
                                <h3 className="text-lg font-semibold">Rotation</h3>
                            </div>
                            <div className="space-y-4">
                                <div>
                                    <span className="text-sm text-gray-500">Auto-rotation</span>
                                    <p className="font-medium">
                                        {credential.rotation_enabled ? 'Enabled' : 'Disabled'}
                                    </p>
                                </div>
                                {credential.rotation_enabled && (
                                    <>
                                        <div>
                                            <span className="text-sm text-gray-500">Interval</span>
                                            <p className="font-medium">{credential.rotation_interval_days} days</p>
                                        </div>
                                        <div>
                                            <span className="text-sm text-gray-500">Next Rotation</span>
                                            <p className="font-medium">
                                                {credential.next_rotation_due
                                                    ? new Date(credential.next_rotation_due).toLocaleDateString()
                                                    : 'Not scheduled'}
                                            </p>
                                        </div>
                                    </>
                                )}
                                <Button
                                    variant="secondary"
                                    className="w-full"
                                    onClick={() => setShowRotateDialog(true)}
                                >
                                    <RefreshCw className="h-4 w-4 mr-2" />
                                    Rotate Now
                                </Button>
                            </div>
                        </Card>

                        {/* Quick Actions */}
                        <Card className="p-6">
                            <h3 className="text-lg font-semibold mb-4">Quick Actions</h3>
                            <div className="space-y-2">
                                <Link to={`/credentials/${id}/edit`}>
                                    <Button variant="secondary" className="w-full justify-start">
                                        Edit Credential
                                    </Button>
                                </Link>
                                <Link to={`/agents/${credential.agent_id}`}>
                                    <Button variant="secondary" className="w-full justify-start">
                                        View Agent
                                    </Button>
                                </Link>
                            </div>
                        </Card>
                    </div>
                </div>
            </div>

            <ConfirmDialog
                isOpen={showRotateDialog}
                onClose={() => setShowRotateDialog(false)}
                onConfirm={handleRotate}
                title="Rotate Credential"
                message="Are you sure you want to rotate this credential? The current secret will be invalidated and a new one will be generated."
                confirmText="Rotate"
                variant="warning"
                loading={rotating}
            />
        </DashboardLayout>
    );
}
