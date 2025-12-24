import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import { z } from 'zod';
import type { Agent, CreateAgentData } from '../../types';
import Input from '../common/Input';
import Button from '../common/Button';

const agentSchema = z.object({
    name: z.string().min(2, 'Name must be at least 2 characters'),
    description: z.string().optional(),
});

interface AgentFormProps {
    onSubmit: (data: CreateAgentData) => Promise<void>;
    initialData?: Agent | null;
    onCancel: () => void;
    isLoading?: boolean;
}

export default function AgentForm({ onSubmit, initialData, onCancel, isLoading }: AgentFormProps) {
    const {
        register,
        handleSubmit,
        formState: { errors },
    } = useForm<CreateAgentData>({
        resolver: zodResolver(agentSchema),
        defaultValues: {
            name: initialData?.name || '',
            description: initialData?.description || '',
        },
    });

    return (
        <form onSubmit={handleSubmit(onSubmit)} className="space-y-6">
            <Input
                label="Agent Name"
                placeholder="e.g. Trading Bot Alpha"
                {...register('name')}
                error={errors.name?.message}
            />

            <div className="space-y-2">
                <label className="text-sm font-medium">Description (Optional)</label>
                <textarea
                    rows={3}
                    placeholder="What does this agent do?"
                    className="flex w-full rounded-md border border-input bg-background px-3 py-2 text-sm focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50"
                    {...register('description')}
                />
            </div>

            <div className="flex justify-end gap-3 pt-2">
                <Button variant="secondary" onClick={onCancel} type="button" disabled={isLoading}>
                    Cancel
                </Button>
                <Button type="submit" isLoading={isLoading}>
                    {initialData ? 'Update Agent' : 'Create Agent'}
                </Button>
            </div>
        </form>
    );
}
