import { useState } from 'react';
import { useCredentials } from '../../hooks/useCredentials';
import CredentialTable from './CredentialTable';
import CredentialForm from './CredentialForm';
import Modal from '../common/Modal';
import Button from '../common/Button';
import Card from '../common/Card';
import { Plus, Loader2, Key } from 'lucide-react';
import type { Credential } from '../../types';

export default function CredentialsList() {
    const { credentials, isLoading, createCredential, deleteCredential, updateCredential, rotateCredential } = useCredentials();
    const [isModalOpen, setIsModalOpen] = useState(false);
    const [editingCred, setEditingCred] = useState<Credential | null>(null);
    const [isProcessing, setIsProcessing] = useState(false);

    const handleCreateClick = () => {
        setEditingCred(null);
        setIsModalOpen(true);
    };

    const handleEditClick = (cred: Credential) => {
        setEditingCred(cred);
        setIsModalOpen(true);
    };

    const handleDeleteClick = async (id: string) => {
        if (window.confirm('Are you sure you want to delete this credential? This action cannot be undone.')) {
            try {
                setIsProcessing(true);
                await deleteCredential(id);
            } finally {
                setIsProcessing(false);
            }
        }
    };

    const handleRotateClick = async (id: string) => {
        if (window.confirm('Trigger immediate rotation for this credential?')) {
            try {
                setIsProcessing(true);
                await rotateCredential(id);
            } finally {
                setIsProcessing(false);
            }
        }
    }

    const handleSubmit = async (data: any) => {
        try {
            setIsProcessing(true);
            if (editingCred) {
                await updateCredential(editingCred.id, data);
            } else {
                await createCredential(data);
            }
            setIsModalOpen(false);
        } finally {
            setIsProcessing(false);
        }
    };

    return (
        <div className="space-y-6">
            <div className="flex items-center justify-between">
                <div>
                    <h2 className="text-2xl font-bold text-gray-900">Credentials</h2>
                    <p className="text-gray-500 text-sm">Manage secrets and keys used by your agents.</p>
                </div>
                <Button onClick={handleCreateClick}>
                    <Plus size={18} className="mr-2" />
                    Add Credential
                </Button>
            </div>

            <Card className="bg-white shadow-sm overflow-hidden border border-gray-200">
                {isLoading ? (
                    <div className="h-64 flex items-center justify-center">
                        <Loader2 className="h-8 w-8 animate-spin text-primary" />
                    </div>
                ) : credentials.length > 0 ? (
                    <CredentialTable
                        credentials={credentials}
                        onEdit={handleEditClick}
                        onDelete={handleDeleteClick}
                        onRotate={handleRotateClick}
                    />
                ) : (
                    <div className="py-20 text-center">
                        <div className="bg-gray-50 h-16 w-16 rounded-full flex items-center justify-center mx-auto mb-4">
                            <Key className="text-gray-400" size={32} />
                        </div>
                        <h3 className="text-lg font-medium text-gray-900">No credentials yet</h3>
                        <p className="text-gray-500 mt-1">Start by adding your first secret.</p>
                        <Button variant="secondary" className="mt-6" onClick={handleCreateClick}>
                            Add Your First Credential
                        </Button>
                    </div>
                )}
            </Card>

            <Modal
                isOpen={isModalOpen}
                onClose={() => setIsModalOpen(false)}
                title={editingCred ? 'Edit Credential' : 'Add New Credential'}
            >
                <CredentialForm
                    onSubmit={handleSubmit}
                    initialData={editingCred}
                    onCancel={() => setIsModalOpen(false)}
                    isLoading={isProcessing}
                />
            </Modal>
        </div>
    );
}
