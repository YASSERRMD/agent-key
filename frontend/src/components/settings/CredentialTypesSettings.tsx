import { useState, useEffect } from 'react';
import Card from '../common/Card';
import Button from '../common/Button';
import Modal from '../common/Modal';
import Badge from '../common/Badge';
import { Plus, Edit, Trash2, Tag, Loader2 } from 'lucide-react';
import { credentialTypeService, type CredentialType, type CreateCredentialTypeData } from '../../services/credentialTypeService';

export default function CredentialTypesSettings() {
    const [types, setTypes] = useState<CredentialType[]>([]);
    const [isLoading, setIsLoading] = useState(true);
    const [showModal, setShowModal] = useState(false);
    const [editingType, setEditingType] = useState<CredentialType | null>(null);
    const [formData, setFormData] = useState<CreateCredentialTypeData>({
        name: '',
        display_name: '',
        description: '',
        icon: 'key',
        color: 'gray'
    });
    const [isSubmitting, setIsSubmitting] = useState(false);
    const [error, setError] = useState<string | null>(null);

    const colors = [
        { value: 'gray', label: 'Gray' },
        { value: 'blue', label: 'Blue' },
        { value: 'green', label: 'Green' },
        { value: 'orange', label: 'Orange' },
        { value: 'purple', label: 'Purple' },
        { value: 'teal', label: 'Teal' },
        { value: 'red', label: 'Red' },
    ];

    useEffect(() => {
        loadTypes();
    }, []);

    const loadTypes = async () => {
        try {
            setIsLoading(true);
            const data = await credentialTypeService.getCredentialTypes();
            setTypes(data);
        } catch (err) {
            console.error('Failed to load credential types:', err);
        } finally {
            setIsLoading(false);
        }
    };

    const handleAdd = () => {
        setEditingType(null);
        setFormData({ name: '', display_name: '', description: '', icon: 'key', color: 'gray' });
        setError(null);
        setShowModal(true);
    };

    const handleEdit = (type: CredentialType) => {
        setEditingType(type);
        setFormData({
            name: type.name,
            display_name: type.display_name,
            description: type.description || '',
            icon: type.icon || 'key',
            color: type.color || 'gray'
        });
        setError(null);
        setShowModal(true);
    };

    const handleDelete = async (type: CredentialType) => {
        if (!window.confirm(`Delete credential type "${type.display_name}"? This cannot be undone.`)) {
            return;
        }
        try {
            await credentialTypeService.deleteCredentialType(type.id);
            await loadTypes();
        } catch (err: any) {
            alert(err.response?.data?.message || 'Failed to delete');
        }
    };

    const handleSubmit = async (e: React.FormEvent) => {
        e.preventDefault();
        if (!formData.name.trim() || !formData.display_name.trim()) {
            setError('Name and display name are required');
            return;
        }

        try {
            setIsSubmitting(true);
            setError(null);
            if (editingType) {
                await credentialTypeService.updateCredentialType(editingType.id, {
                    display_name: formData.display_name,
                    description: formData.description,
                    icon: formData.icon,
                    color: formData.color
                });
            } else {
                await credentialTypeService.createCredentialType(formData);
            }
            setShowModal(false);
            await loadTypes();
        } catch (err: any) {
            setError(err.response?.data?.message || 'Failed to save');
        } finally {
            setIsSubmitting(false);
        }
    };

    const getColorClass = (color?: string) => {
        const colorMap: Record<string, string> = {
            gray: 'bg-gray-100 text-gray-600',
            blue: 'bg-blue-100 text-blue-600',
            green: 'bg-green-100 text-green-600',
            orange: 'bg-orange-100 text-orange-600',
            purple: 'bg-purple-100 text-purple-600',
            teal: 'bg-teal-100 text-teal-600',
            red: 'bg-red-100 text-red-600',
        };
        return colorMap[color || 'gray'] || colorMap.gray;
    };

    return (
        <div className="space-y-6">
            <div className="flex items-center justify-between">
                <div>
                    <h3 className="text-lg font-semibold">Credential Types</h3>
                    <p className="text-sm text-gray-500">Customize the types of credentials your team can create</p>
                </div>
                <Button onClick={handleAdd}>
                    <Plus size={16} className="mr-2" />
                    Add Type
                </Button>
            </div>

            <Card className="p-0 overflow-hidden">
                {isLoading ? (
                    <div className="flex items-center justify-center py-12">
                        <Loader2 className="h-8 w-8 animate-spin text-primary" />
                    </div>
                ) : types.length === 0 ? (
                    <div className="py-12 text-center">
                        <Tag className="h-12 w-12 text-gray-300 mx-auto mb-3" />
                        <p className="text-gray-500">No credential types configured</p>
                    </div>
                ) : (
                    <table className="w-full">
                        <thead className="bg-gray-50 border-b">
                            <tr>
                                <th className="text-left px-6 py-3 text-xs font-medium text-gray-500 uppercase">Type</th>
                                <th className="text-left px-6 py-3 text-xs font-medium text-gray-500 uppercase">Name (Key)</th>
                                <th className="text-left px-6 py-3 text-xs font-medium text-gray-500 uppercase">Description</th>
                                <th className="text-left px-6 py-3 text-xs font-medium text-gray-500 uppercase">Status</th>
                                <th className="text-right px-6 py-3 text-xs font-medium text-gray-500 uppercase">Actions</th>
                            </tr>
                        </thead>
                        <tbody className="divide-y">
                            {types.map((type) => (
                                <tr key={type.id} className="hover:bg-gray-50">
                                    <td className="px-6 py-4">
                                        <div className="flex items-center gap-3">
                                            <div className={`p-2 rounded-lg ${getColorClass(type.color)}`}>
                                                <Tag size={18} />
                                            </div>
                                            <span className="font-medium">{type.display_name}</span>
                                        </div>
                                    </td>
                                    <td className="px-6 py-4">
                                        <code className="px-2 py-1 bg-gray-100 rounded text-sm">{type.name}</code>
                                    </td>
                                    <td className="px-6 py-4 text-sm text-gray-500">
                                        {type.description || '-'}
                                    </td>
                                    <td className="px-6 py-4">
                                        <Badge variant={type.is_system ? 'gray' : 'success'}>
                                            {type.is_system ? 'System' : 'Custom'}
                                        </Badge>
                                    </td>
                                    <td className="px-6 py-4 text-right">
                                        {!type.is_system && (
                                            <div className="flex justify-end gap-2">
                                                <Button variant="ghost" size="sm" onClick={() => handleEdit(type)}>
                                                    <Edit size={16} />
                                                </Button>
                                                <Button variant="ghost" size="sm" onClick={() => handleDelete(type)} className="text-red-500 hover:text-red-600">
                                                    <Trash2 size={16} />
                                                </Button>
                                            </div>
                                        )}
                                    </td>
                                </tr>
                            ))}
                        </tbody>
                    </table>
                )}
            </Card>

            <Modal
                isOpen={showModal}
                onClose={() => setShowModal(false)}
                title={editingType ? 'Edit Credential Type' : 'Add Credential Type'}
            >
                <form onSubmit={handleSubmit} className="space-y-4">
                    {error && (
                        <div className="p-3 rounded-lg bg-red-50 border border-red-200 text-red-700 text-sm">
                            {error}
                        </div>
                    )}
                    {!editingType && (
                        <div>
                            <label className="block text-sm font-medium text-gray-700 mb-2">
                                Name (Key) *
                            </label>
                            <input
                                type="text"
                                value={formData.name}
                                onChange={(e) => setFormData({ ...formData, name: e.target.value.toLowerCase().replace(/\s/g, '_') })}
                                placeholder="e.g., github_token"
                                className="w-full px-4 py-3 rounded-lg border border-gray-300 focus:outline-none focus:ring-2 focus:ring-teal-500"
                            />
                            <p className="text-xs text-gray-400 mt-1">Lowercase, no spaces (used in SDK)</p>
                        </div>
                    )}
                    <div>
                        <label className="block text-sm font-medium text-gray-700 mb-2">
                            Display Name *
                        </label>
                        <input
                            type="text"
                            value={formData.display_name}
                            onChange={(e) => setFormData({ ...formData, display_name: e.target.value })}
                            placeholder="e.g., GitHub Token"
                            className="w-full px-4 py-3 rounded-lg border border-gray-300 focus:outline-none focus:ring-2 focus:ring-teal-500"
                        />
                    </div>
                    <div>
                        <label className="block text-sm font-medium text-gray-700 mb-2">
                            Description
                        </label>
                        <input
                            type="text"
                            value={formData.description}
                            onChange={(e) => setFormData({ ...formData, description: e.target.value })}
                            placeholder="e.g., Personal access token for GitHub"
                            className="w-full px-4 py-3 rounded-lg border border-gray-300 focus:outline-none focus:ring-2 focus:ring-teal-500"
                        />
                    </div>
                    <div>
                        <label className="block text-sm font-medium text-gray-700 mb-2">
                            Color
                        </label>
                        <select
                            value={formData.color}
                            onChange={(e) => setFormData({ ...formData, color: e.target.value })}
                            className="w-full px-4 py-3 rounded-lg border border-gray-300 focus:outline-none focus:ring-2 focus:ring-teal-500"
                        >
                            {colors.map((c) => (
                                <option key={c.value} value={c.value}>{c.label}</option>
                            ))}
                        </select>
                    </div>
                    <div className="flex gap-3 pt-4">
                        <Button type="submit" isLoading={isSubmitting} className="flex-1">
                            {editingType ? 'Save Changes' : 'Create Type'}
                        </Button>
                        <Button variant="secondary" onClick={() => setShowModal(false)} type="button">
                            Cancel
                        </Button>
                    </div>
                </form>
            </Modal>
        </div>
    );
}
