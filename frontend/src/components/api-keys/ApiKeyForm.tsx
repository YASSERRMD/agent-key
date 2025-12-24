import React, { useState } from 'react';
import { ShieldCheck, Copy, Check, Info } from 'lucide-react';
import Button from '../common/Button';
import Alert from '../common/Alert';
import Input from '../common/Input';

interface ApiKeyFormProps {
    onCreate: (data: { expires_in_days?: number }) => Promise<{ id: string; api_key: string } | null>;
    onClose: () => void;
}

export default function ApiKeyForm({ onCreate, onClose }: ApiKeyFormProps) {
    const [expiresIn, setExpiresIn] = useState<number>(30);
    const [loading, setLoading] = useState(false);
    const [newKey, setNewKey] = useState<string | null>(null);
    const [copied, setCopied] = useState(false);

    const handleSubmit = async (e: React.FormEvent) => {
        e.preventDefault();
        setLoading(true);
        const result = await onCreate({ expires_in_days: expiresIn });
        if (result) {
            setNewKey(result.api_key);
        }
        setLoading(false);
    };

    const handleCopy = () => {
        if (newKey) {
            navigator.clipboard.writeText(newKey);
            setCopied(true);
            setTimeout(() => setCopied(false), 2000);
        }
    };

    if (newKey) {
        return (
            <div className="space-y-6">
                <Alert variant="success">
                    <span className="font-bold block mb-1">API Key Created Successfully</span>
                    Please copy your new API key now. For security reasons, you won't be able to see it again.
                </Alert>

                <div className="relative">
                    <div className="flex items-center justify-between p-4 bg-gray-50 rounded-lg border border-gray-200">
                        <code className="text-sm font-mono text-primary break-all pr-12">
                            {newKey}
                        </code>
                        <Button
                            variant="primary"
                            size="sm"
                            onClick={handleCopy}
                            className="absolute right-4 top-1/2 -translate-y-1/2"
                        >
                            {copied ? <Check size={16} /> : <Copy size={16} />}
                        </Button>
                    </div>
                </div>

                <div className="flex items-start space-x-3 text-sm text-gray-500 bg-blue-50 p-4 rounded-lg">
                    <Info className="h-5 w-5 text-blue-500 flex-shrink-0 mt-0.5" />
                    <p>
                        Store this key securely. If lost, you'll need to generate a new one and update your integrations.
                    </p>
                </div>

                <div className="flex justify-end">
                    <Button variant="primary" onClick={onClose}>
                        Done
                    </Button>
                </div>
            </div>
        );
    }

    return (
        <form onSubmit={handleSubmit} className="space-y-6">
            <div className="flex items-center space-x-3 text-gray-600 mb-6">
                <ShieldCheck className="h-6 w-6 text-primary" />
                <h3 className="text-lg font-medium">Create New API Key</h3>
            </div>

            <Input
                label="Expiration (Days)"
                type="number"
                min={1}
                max={365}
                value={expiresIn}
                onChange={(e) => setExpiresIn(parseInt(e.target.value))}
            />
            <p className="text-xs text-gray-500 -mt-4">How many days until this key expires (1-365). Leave empty for no expiration.</p>

            <div className="flex justify-end space-x-3">
                <Button variant="secondary" type="button" onClick={onClose} disabled={loading}>
                    Cancel
                </Button>
                <Button variant="primary" type="submit" isLoading={loading}>
                    Generate Key
                </Button>
            </div>
        </form>
    );
}
