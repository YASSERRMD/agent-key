import { useState } from 'react';
import { useAgents } from '../../hooks/useAgents';
import AgentTable from './AgentTable';
import AgentForm from './AgentForm';
import Modal from '../common/Modal';
import Button from '../common/Button';
import Card from '../common/Card';
import { Plus, Loader2 } from 'lucide-react';
import type { Agent, CreateAgentData } from '../../types';
import { useNavigate } from 'react-router-dom';

export default function AgentsList() {
    const navigate = useNavigate();
    const { agents, isLoading, createAgent, deleteAgent, updateAgent } = useAgents();
    const [isModalOpen, setIsModalOpen] = useState(false);
    const [editingAgent, setEditingAgent] = useState<Agent | null>(null);
    const [isProcessing, setIsProcessing] = useState(false);

    const handleCreateClick = () => {
        setEditingAgent(null);
        setIsModalOpen(true);
    };

    const handleEditClick = (agent: Agent) => {
        setEditingAgent(agent);
        setIsModalOpen(true);
    };

    const handleDeleteClick = async (id: string) => {
        if (window.confirm('Are you sure you want to delete this agent? This will revoke all associated API keys.')) {
            try {
                setIsProcessing(true);
                await deleteAgent(id);
            } finally {
                setIsProcessing(false);
            }
        }
    };

    const handleSubmit = async (data: CreateAgentData) => {
        try {
            setIsProcessing(true);
            if (editingAgent) {
                await updateAgent(editingAgent.id, data);
            } else {
                await createAgent(data);
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
                    <h2 className="text-2xl font-bold text-gray-900">Agents</h2>
                    <p className="text-gray-500 text-sm">Create and manage identities for your AI systems.</p>
                </div>
                <Button onClick={handleCreateClick}>
                    <Plus size={18} className="mr-2" />
                    Create Agent
                </Button>
            </div>

            <Card className="bg-white shadow-sm overflow-hidden">
                {isLoading ? (
                    <div className="h-64 flex items-center justify-center">
                        <Loader2 className="h-8 w-8 animate-spin text-primary" />
                    </div>
                ) : (
                    <AgentTable
                        agents={agents}
                        onEdit={handleEditClick}
                        onDelete={handleDeleteClick}
                        onView={(id) => navigate(`/agents/${id}`)}
                    />
                )}
            </Card>

            <Modal
                isOpen={isModalOpen}
                onClose={() => setIsModalOpen(false)}
                title={editingAgent ? 'Edit Agent' : 'Create New Agent'}
            >
                <AgentForm
                    onSubmit={handleSubmit}
                    initialData={editingAgent}
                    onCancel={() => setIsModalOpen(false)}
                    isLoading={isProcessing}
                />
            </Modal>
        </div>
    );
}
