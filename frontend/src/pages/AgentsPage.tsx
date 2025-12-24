import DashboardLayout from '../components/dashboard/DashboardLayout';
import AgentsList from '../components/agents/AgentsList';

export default function AgentsPage() {
    return (
        <DashboardLayout>
            <AgentsList />
        </DashboardLayout>
    );
}
