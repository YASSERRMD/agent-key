import React from 'react';
import { Key, Trash2, ShieldCheck, ShieldAlert, Clock, Copy } from 'lucide-react';
import type { ApiKey } from '../../types';
import Button from '../common/Button';
import Badge from '../common/Badge';
import { format } from 'date-fns';

interface ApiKeyListProps {
    keys: ApiKey[];
    onRevoke: (id: string) => void;
    loading?: boolean;
}

export default function ApiKeyList({ keys, onRevoke, loading }: ApiKeyListProps) {
    const [copiedId, setCopiedId] = React.useState<string | null>(null);

    const handleCopy = (id: string, keySuffix: string) => {
        // Since we only have the prefix/suffix for security, we just mock the copy
        // until we get the full key on creation
        navigator.clipboard.writeText(keySuffix);
        setCopiedId(id);
        setTimeout(() => setCopiedId(null), 2000);
    };

    if (keys.length === 0 && !loading) {
        return (
            <div className="text-center py-12 bg-gray-50 rounded-lg border-2 border-dashed border-gray-200">
                <Key className="mx-auto h-12 w-12 text-gray-400 mb-4" />
                <h3 className="text-lg font-medium text-gray-900">No API keys found</h3>
                <p className="mt-1 text-sm text-gray-500">
                    Create a new API key to start using AgentKey SDK.
                </p>
            </div>
        );
    }

    return (
        <div className="overflow-hidden bg-white shadow sm:rounded-md border border-gray-200">
            <ul role="list" className="divide-y divide-gray-200">
                {keys.map((key) => (
                    <li key={key.id}>
                        <div className="px-4 py-4 sm:px-6 hover:bg-gray-50 transition-colors">
                            <div className="flex items-center justify-between">
                                <div className="flex items-center space-x-3">
                                    <div className="flex-shrink-0">
                                        <Key className="h-6 w-6 text-gray-400" />
                                    </div>
                                    <div className="min-w-0 flex-1">
                                        <p className="truncate text-sm font-medium text-primary">
                                            {key.name || `API Key (${key.key_prefix || '...'})`}
                                        </p>
                                        <div className="mt-1 flex items-center space-x-4 text-xs text-gray-500">
                                            <div className="flex items-center">
                                                <Clock className="mr-1.5 h-4 w-4 flex-shrink-0 text-gray-400" />
                                                Created {format(new Date(key.created_at), 'MMM d, yyyy')}
                                            </div>
                                            {key.last_used && (
                                                <div className="flex items-center">
                                                    <ShieldCheck className="mr-1.5 h-4 w-4 flex-shrink-0 text-green-500" />
                                                    Last used {format(new Date(key.last_used), 'MMM d, h:mm a')}
                                                </div>
                                            )}
                                            {key.expires_at && (
                                                <div className="flex items-center">
                                                    <ShieldAlert className="mr-1.5 h-4 w-4 flex-shrink-0 text-amber-500" />
                                                    Expires {format(new Date(key.expires_at), 'MMM d, yyyy')}
                                                </div>
                                            )}
                                        </div>
                                    </div>
                                </div>
                                <div className="flex items-center space-x-2">
                                    <Badge variant={key.status === 'active' ? 'success' : 'danger'}>
                                        {key.status}
                                    </Badge>
                                    <Button
                                        variant="secondary"
                                        size="sm"
                                        onClick={() => handleCopy(key.id, key.key_prefix || '')}
                                        title="Copy Key Hash"
                                    >
                                        <Copy size={16} className={copiedId === key.id ? "text-green-500" : ""} />
                                    </Button>
                                    <Button
                                        variant="ghost"
                                        size="sm"
                                        onClick={() => onRevoke(key.id)}
                                        className="text-destructive hover:text-destructive hover:bg-destructive/10"
                                        title="Revoke Key"
                                        disabled={key.status !== 'active'}
                                    >
                                        <Trash2 size={16} />
                                    </Button>
                                </div>
                            </div>
                        </div>
                    </li>
                ))}
            </ul>
        </div>
    );
}
