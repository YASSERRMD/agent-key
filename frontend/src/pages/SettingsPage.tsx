import { useParams, Link } from 'react-router-dom';
import DashboardLayout from '../components/dashboard/DashboardLayout';
import ProfileSettings from '../components/settings/ProfileSettings';
import TeamSettings from '../components/settings/TeamSettings';
import BillingSettings from '../components/settings/BillingSettings';
import ApiKeysSettings from '../components/settings/ApiKeysSettings';
import CredentialTypesSettings from '../components/settings/CredentialTypesSettings';
import { User, Users, CreditCard, Key, FileText, Tag } from 'lucide-react';

const tabs = [
    { id: 'profile', label: 'Profile', icon: User, component: ProfileSettings },
    { id: 'team', label: 'Team', icon: Users, component: TeamSettings },
    { id: 'api-keys', label: 'API Keys', icon: Key, component: ApiKeysSettings },
    { id: 'credential-types', label: 'Credential Types', icon: Tag, component: CredentialTypesSettings },
    { id: 'billing', label: 'Billing', icon: CreditCard, component: BillingSettings },
];

export default function SettingsPage() {
    const { tab } = useParams<{ tab?: string }>();
    const activeTab = tab || 'profile';

    const ActiveComponent = tabs.find((t) => t.id === activeTab)?.component || ProfileSettings;

    return (
        <DashboardLayout>
            <div className="space-y-6">
                <div>
                    <h1 className="text-2xl font-bold text-gray-900">Settings</h1>
                    <p className="text-gray-500">Manage your account and preferences</p>
                </div>

                <div className="flex gap-8">
                    {/* Sidebar Navigation */}
                    <nav className="w-48 flex-shrink-0">
                        <ul className="space-y-1">
                            {tabs.map(({ id, label, icon: Icon }) => (
                                <li key={id}>
                                    <Link
                                        to={`/settings/${id}`}
                                        className={`flex items-center gap-3 px-4 py-2 rounded-lg text-sm font-medium transition-colors ${activeTab === id
                                            ? 'bg-teal-50 text-teal-700'
                                            : 'text-gray-600 hover:bg-gray-100'
                                            }`}
                                    >
                                        <Icon className="h-4 w-4" />
                                        {label}
                                    </Link>
                                </li>
                            ))}
                            <li>
                                <Link
                                    to="/audit"
                                    className="flex items-center gap-3 px-4 py-2 rounded-lg text-sm font-medium text-gray-600 hover:bg-gray-100 transition-colors"
                                >
                                    <FileText className="h-4 w-4" />
                                    Audit Log
                                </Link>
                            </li>
                        </ul>
                    </nav>

                    {/* Content */}
                    <div className="flex-1 min-w-0">
                        <ActiveComponent />
                    </div>
                </div>
            </div>
        </DashboardLayout>
    );
}
