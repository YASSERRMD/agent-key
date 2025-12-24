import DashboardLayout from '../components/dashboard/DashboardLayout';
import CredentialsList from '../components/credentials/CredentialsList';

export default function CredentialsPage() {
    return (
        <DashboardLayout>
            <CredentialsList />
        </DashboardLayout>
    );
}
