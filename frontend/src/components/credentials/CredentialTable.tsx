import type { Credential } from '../../types';
import Badge from '../common/Badge';
import { Edit, Trash2, RefreshCw, Key } from 'lucide-react';
import { format } from 'date-fns';

interface CredentialTableProps {
    credentials: Credential[];
    onEdit: (credential: Credential) => void;
    onDelete: (id: string) => void;
    onRotate: (id: string) => void;
}

export default function CredentialTable({ credentials, onEdit, onDelete, onRotate }: CredentialTableProps) {
    return (
        <div className="overflow-x-auto">
            <table className="w-full text-left border-collapse">
                <thead className="bg-gray-50 border-b border-gray-200">
                    <tr>
                        <th className="px-6 py-3 text-xs font-semibold text-gray-500 uppercase tracking-wider">Credential</th>
                        <th className="px-6 py-3 text-xs font-semibold text-gray-500 uppercase tracking-wider">Type</th>
                        <th className="px-6 py-3 text-xs font-semibold text-gray-500 uppercase tracking-wider">Rotation</th>
                        <th className="px-6 py-3 text-xs font-semibold text-gray-500 uppercase tracking-wider">Last Rotated</th>
                        <th className="px-6 py-3 text-xs font-semibold text-gray-500 uppercase tracking-wider text-right">Actions</th>
                    </tr>
                </thead>
                <tbody className="divide-y divide-gray-200 bg-white">
                    {credentials.map((cred) => (
                        <tr key={cred.id} className="hover:bg-gray-50 transition-colors">
                            <td className="px-6 py-4 whitespace-nowrap">
                                <div className="flex items-center">
                                    <div className="h-8 w-8 rounded bg-gray-100 flex items-center justify-center mr-3 text-gray-400">
                                        <Key size={16} />
                                    </div>
                                    <div className="flex flex-col">
                                        <span className="text-sm font-medium text-gray-900">{cred.name}</span>
                                        <span className="text-xs text-gray-500 truncate max-w-[200px]">{cred.description || 'No description'}</span>
                                    </div>
                                </div>
                            </td>
                            <td className="px-6 py-4 whitespace-nowrap">
                                <Badge variant="info">
                                    {cred.credential_type}
                                </Badge>
                            </td>
                            <td className="px-6 py-4 whitespace-nowrap">
                                {cred.rotation_enabled ? (
                                    <Badge variant="success">Auto-rotate</Badge>
                                ) : (
                                    <Badge variant="gray">Manual</Badge>
                                )}
                            </td>
                            <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                                {cred.last_rotated ? format(new Date(cred.last_rotated), 'MMM d, yyyy') : 'Never'}
                            </td>
                            <td className="px-6 py-4 whitespace-nowrap text-right text-sm font-medium">
                                <div className="flex justify-end gap-2">
                                    <button
                                        onClick={() => onRotate(cred.id)}
                                        className="p-1.5 text-gray-400 hover:text-green-600 transition-colors"
                                        title="Rotate now"
                                    >
                                        <RefreshCw size={16} />
                                    </button>
                                    <button
                                        onClick={() => onEdit(cred)}
                                        className="p-1.5 text-gray-400 hover:text-blue-600 transition-colors"
                                        title="Edit credential"
                                    >
                                        <Edit size={16} />
                                    </button>
                                    <button
                                        onClick={() => onDelete(cred.id)}
                                        className="p-1.5 text-gray-400 hover:text-destructive transition-colors"
                                        title="Delete credential"
                                    >
                                        <Trash2 size={16} />
                                    </button>
                                </div>
                            </td>
                        </tr>
                    ))}
                </tbody>
            </table>
        </div>
    );
}
