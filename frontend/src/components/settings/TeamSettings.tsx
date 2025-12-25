import { useState } from 'react';
import { useAuth } from '../../hooks/useAuth';
import Button from '../common/Button';
import Card from '../common/Card';
import Badge from '../common/Badge';
import { Users, UserPlus, Mail, Trash2 } from 'lucide-react';

interface TeamMember {
    id: string;
    email: string;
    name: string;
    role: 'admin' | 'developer' | 'viewer';
    status: 'active' | 'pending';
}

export default function TeamSettings() {
    const { user } = useAuth();
    const [inviteEmail, setInviteEmail] = useState('');
    const [inviteRole, setInviteRole] = useState<'developer' | 'viewer'>('developer');
    const [inviting, setInviting] = useState(false);

    // Mock team members
    const [members] = useState<TeamMember[]>([
        { id: '1', email: user?.email || '', name: user?.name || 'You', role: 'admin', status: 'active' },
    ]);

    const handleInvite = async () => {
        setInviting(true);
        // TODO: Implement invite API
        await new Promise((r) => setTimeout(r, 1000));
        setInviting(false);
        setInviteEmail('');
    };

    const roleColors = {
        admin: 'bg-purple-100 text-purple-700',
        developer: 'bg-blue-100 text-blue-700',
        viewer: 'bg-gray-100 text-gray-700',
    };

    return (
        <div className="space-y-6">
            {/* Team Info */}
            <Card className="p-6">
                <div className="flex items-center justify-between mb-4">
                    <div>
                        <h3 className="text-lg font-semibold">Team Information</h3>
                        <p className="text-gray-500 text-sm">Manage your team settings and members</p>
                    </div>
                    <Badge variant="info">Free Plan</Badge>
                </div>
                <div className="grid grid-cols-2 gap-4">
                    <div>
                        <label className="block text-sm font-medium text-gray-700 mb-2">Team Name</label>
                        <input
                            type="text"
                            defaultValue="My Team"
                            className="w-full px-4 py-3 rounded-lg border border-gray-300 focus:outline-none focus:ring-2 focus:ring-teal-500"
                        />
                    </div>
                    <div>
                        <label className="block text-sm font-medium text-gray-700 mb-2">Team ID</label>
                        <input
                            type="text"
                            value={user?.team_id || ''}
                            disabled
                            className="w-full px-4 py-3 rounded-lg border border-gray-200 bg-gray-50 text-gray-500 font-mono text-sm"
                        />
                    </div>
                </div>
            </Card>

            {/* Invite Member */}
            <Card className="p-6">
                <h3 className="text-lg font-semibold mb-4">
                    <UserPlus className="inline h-5 w-5 mr-2" />
                    Invite Team Member
                </h3>
                <div className="flex gap-4">
                    <div className="flex-1">
                        <input
                            type="email"
                            placeholder="colleague@company.com"
                            value={inviteEmail}
                            onChange={(e) => setInviteEmail(e.target.value)}
                            className="w-full px-4 py-3 rounded-lg border border-gray-300 focus:outline-none focus:ring-2 focus:ring-teal-500"
                        />
                    </div>
                    <select
                        value={inviteRole}
                        onChange={(e) => setInviteRole(e.target.value as 'developer' | 'viewer')}
                        className="px-4 py-3 rounded-lg border border-gray-300 focus:outline-none focus:ring-2 focus:ring-teal-500"
                    >
                        <option value="developer">Developer</option>
                        <option value="viewer">Viewer</option>
                    </select>
                    <Button onClick={handleInvite} isLoading={inviting} disabled={!inviteEmail}>
                        <Mail className="h-4 w-4 mr-2" />
                        Send Invite
                    </Button>
                </div>
            </Card>

            {/* Team Members */}
            <Card className="p-6">
                <h3 className="text-lg font-semibold mb-4">
                    <Users className="inline h-5 w-5 mr-2" />
                    Team Members ({members.length})
                </h3>
                <div className="divide-y">
                    {members.map((member) => (
                        <div key={member.id} className="py-4 flex items-center justify-between">
                            <div className="flex items-center gap-3">
                                <div className="w-10 h-10 rounded-full bg-teal-100 flex items-center justify-center text-teal-600 font-bold">
                                    {member.name.charAt(0).toUpperCase()}
                                </div>
                                <div>
                                    <p className="font-medium">{member.name}</p>
                                    <p className="text-sm text-gray-500">{member.email}</p>
                                </div>
                            </div>
                            <div className="flex items-center gap-3">
                                <span className={`px-2 py-1 rounded text-xs font-medium ${roleColors[member.role]}`}>
                                    {member.role}
                                </span>
                                {member.status === 'pending' && (
                                    <Badge variant="warning">Pending</Badge>
                                )}
                                {member.role !== 'admin' && (
                                    <button className="p-2 text-gray-400 hover:text-red-500 transition-colors">
                                        <Trash2 className="h-4 w-4" />
                                    </button>
                                )}
                            </div>
                        </div>
                    ))}
                </div>
            </Card>

            {/* Danger Zone */}
            <Card className="p-6 border-red-200">
                <h3 className="text-lg font-semibold text-red-600 mb-4">Danger Zone</h3>
                <p className="text-gray-600 mb-4">
                    Once you delete your team, there is no going back. This will delete all agents,
                    credentials, and data associated with this team.
                </p>
                <Button variant="danger">
                    <Trash2 className="h-4 w-4 mr-2" />
                    Delete Team
                </Button>
            </Card>
        </div>
    );
}
