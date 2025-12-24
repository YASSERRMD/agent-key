import type { Agent } from '../../types';
import Badge from '../common/Badge';
import { Edit, Trash2, ExternalLink } from 'lucide-react';
import { format } from 'date-fns';

interface AgentTableProps {
    agents: Agent[];
    onEdit: (agent: Agent) => void;
    onDelete: (id: string) => void;
    onView: (id: string) => void;
}

export default function AgentTable({ agents, onEdit, onDelete, onView }: AgentTableProps) {
    return (
        <div className="overflow-x-auto">
            <table className="w-full text-left border-collapse">
                <thead className="bg-gray-50 border-b border-gray-200">
                    <tr>
                        <th className="px-6 py-3 text-xs font-semibold text-gray-500 uppercase tracking-wider">Agent Name</th>
                        <th className="px-6 py-3 text-xs font-semibold text-gray-500 uppercase tracking-wider">Status</th>
                        <th className="px-6 py-3 text-xs font-semibold text-gray-500 uppercase tracking-wider">Usage</th>
                        <th className="px-6 py-3 text-xs font-semibold text-gray-500 uppercase tracking-wider">Created</th>
                        <th className="px-6 py-3 text-xs font-semibold text-gray-500 uppercase tracking-wider text-right">Actions</th>
                    </tr>
                </thead>
                <tbody className="divide-y divide-gray-200 bg-white">
                    {agents.map((agent) => (
                        <tr key={agent.id} className="hover:bg-gray-50 transition-colors">
                            <td className="px-6 py-4 whitespace-nowrap">
                                <div className="flex flex-col">
                                    <span className="text-sm font-medium text-gray-900">{agent.name}</span>
                                    <span className="text-xs text-gray-500 truncate max-w-[200px]">{agent.description || 'No description'}</span>
                                </div>
                            </td>
                            <td className="px-6 py-4 whitespace-nowrap">
                                <Badge variant={agent.status === 'active' ? 'success' : 'gray'}>
                                    {agent.status}
                                </Badge>
                            </td>
                            <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                                {agent.usage_count} calls
                            </td>
                            <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                                {format(new Date(agent.created_at), 'MMM d, yyyy')}
                            </td>
                            <td className="px-6 py-4 whitespace-nowrap text-right text-sm font-medium">
                                <div className="flex justify-end gap-2">
                                    <button
                                        onClick={() => onView(agent.id)}
                                        className="p-1.5 text-gray-400 hover:text-primary transition-colors"
                                        title="View details"
                                    >
                                        <ExternalLink size={16} />
                                    </button>
                                    <button
                                        onClick={() => onEdit(agent)}
                                        className="p-1.5 text-gray-400 hover:text-blue-600 transition-colors"
                                        title="Edit agent"
                                    >
                                        <Edit size={16} />
                                    </button>
                                    <button
                                        onClick={() => onDelete(agent.id)}
                                        className="p-1.5 text-gray-400 hover:text-destructive transition-colors"
                                        title="Delete agent"
                                    >
                                        <Trash2 size={16} />
                                    </button>
                                </div>
                            </td>
                        </tr>
                    ))}
                </tbody>
            </table>
            {agents.length === 0 && (
                <div className="py-12 text-center">
                    <p className="text-gray-500 text-sm">No agents found. Create one to get started.</p>
                </div>
            )}
        </div>
    );
}
