import { useState, useEffect } from 'react';
import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import { z } from 'zod';
import type { Credential, CreateCredentialData, Agent } from '../../types';
import { agentService } from '../../services/agentService';
import Input from '../common/Input';
import Button from '../common/Button';

const credentialSchema = z.object({
    name: z.string().min(2, 'Name must be at least 2 characters'),
    agent_id: z.string().min(1, 'Agent is required'),
    credential_type: z.string().min(1, 'Type is required'),
    description: z.string().optional(),
    secret: z.string().min(1, 'Secret is required'),
    rotation_enabled: z.boolean().optional(),
    rotation_interval_days: z.number().min(1).max(365).optional(),
});

interface CredentialFormProps {
    onSubmit: (data: CreateCredentialData) => Promise<void>;
    initialData?: Credential | null;
    onCancel: () => void;
    isLoading?: boolean;
    agentId?: string; // Pre-selected agent ID when creating from agent detail page
}

export default function CredentialForm({ onSubmit, initialData, onCancel, isLoading, agentId }: CredentialFormProps) {
    const [agents, setAgents] = useState<Agent[]>([]);
    const [isLoadingAgents, setIsLoadingAgents] = useState(false);

    useEffect(() => {
        const fetchAgents = async () => {
            try {
                setIsLoadingAgents(true);
                const response = await agentService.getAgents(1, 100); // Fetch up to 100 agents for dropdown
                setAgents(response.data);
            } catch (error) {
                console.error('Failed to fetch agents', error);
            } finally {
                setIsLoadingAgents(false);
            }
        };

        if (!initialData) {
            fetchAgents();
        }
    }, [initialData]);

    const {
        register,
        handleSubmit,
        watch,
        formState: { errors },
    } = useForm<CreateCredentialData>({
        resolver: zodResolver(credentialSchema),
        defaultValues: {
            name: initialData?.name || '',
            agent_id: initialData?.agent_id || agentId || '',
            credential_type: initialData?.credential_type || 'generic',
            description: initialData?.description || '',
            secret: '', // We don't populate secret back for security
            rotation_enabled: initialData?.rotation_enabled || false,
            rotation_interval_days: initialData?.rotation_interval_days || 30,
        },
    });

    const rotationEnabled = watch('rotation_enabled');

    return (
        <form onSubmit={handleSubmit(onSubmit)} className="space-y-6">
            {/* Only show agent dropdown when not editing and no agentId is pre-selected */}
            {!initialData && !agentId && (
                <div className="space-y-2">
                    <label className="text-sm font-medium">Agent</label>
                    <select
                        className="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring disabled:opacity-50"
                        {...register('agent_id')}
                        disabled={isLoadingAgents}
                    >
                        <option value="">Select an Agent</option>
                        {agents.map((agent) => (
                            <option key={agent.id} value={agent.id}>
                                {agent.name}
                            </option>
                        ))}
                    </select>
                    {errors.agent_id && (
                        <p className="text-sm text-red-500">{errors.agent_id.message}</p>
                    )}
                </div>
            )}

            <div className="grid grid-cols-2 gap-4">
                <Input
                    label="Name"
                    placeholder="e.g. AWS Production Key"
                    {...register('name')}
                    error={errors.name?.message}
                />
                <div className="space-y-2">
                    <label className="text-sm font-medium">Type</label>
                    <select
                        className="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
                        {...register('credential_type')}
                    >
                        <option value="generic">Generic</option>
                        <option value="aws">AWS</option>
                        <option value="openai">OpenAI</option>
                        <option value="database">Database</option>
                        <option value="api_key">API Key</option>
                    </select>
                </div>
            </div>

            <Input
                label="Description (Optional)"
                placeholder="What is this credential used for?"
                {...register('description')}
                error={errors.description?.message}
            />

            <Input
                label={initialData ? "Update Secret (Leave empty to keep current)" : "Secret Value"}
                type="password"
                placeholder="••••••••••••••••"
                {...register('secret')}
                error={errors.secret?.message}
            />

            <div className="space-y-4 pt-2 border-t border-gray-100">
                <div className="flex items-center">
                    <input
                        type="checkbox"
                        id="rotation_enabled"
                        {...register('rotation_enabled')}
                        className="h-4 w-4 rounded border-gray-300 text-primary focus:ring-primary"
                    />
                    <label htmlFor="rotation_enabled" className="ml-2 text-sm font-medium text-gray-700">
                        Enable Automatic Rotation
                    </label>
                </div>

                {rotationEnabled && (
                    <Input
                        label="Rotation Interval (Days)"
                        type="number"
                        {...register('rotation_interval_days', { valueAsNumber: true })}
                        error={errors.rotation_interval_days?.message}
                    />
                )}
            </div>

            <div className="flex justify-end gap-3 pt-4">
                <Button variant="secondary" onClick={onCancel} type="button" disabled={isLoading}>
                    Cancel
                </Button>
                <Button type="submit" isLoading={isLoading}>
                    {initialData ? 'Update Credential' : 'Create Credential'}
                </Button>
            </div>
        </form>
    );
}
