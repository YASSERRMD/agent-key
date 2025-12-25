import { useState, useEffect } from 'react';
import DashboardLayout from '../components/dashboard/DashboardLayout';
import Card from '../components/common/Card';
import Table from '../components/common/Table';
import Spinner from '../components/common/Spinner';
import { auditService, type AuditEvent } from '../services/auditService';
import { Search, Filter } from 'lucide-react';
import useDebounce from '../hooks/useDebounce';

export default function AuditLogPage() {
    const [logs, setLogs] = useState<AuditEvent[]>([]);
    const [loading, setLoading] = useState(true);
    const [total, setTotal] = useState(0);
    const [page, setPage] = useState(1);
    const [search, setSearch] = useState('');
    const [eventType, setEventType] = useState('');
    const limit = 20;

    const debouncedSearch = useDebounce(search, 300);

    useEffect(() => {
        loadLogs();
    }, [page, debouncedSearch, eventType]);

    const loadLogs = async () => {
        try {
            setLoading(true);
            const response = await auditService.getAuditLogs({
                page,
                limit,
                event_type: eventType || undefined,
            });
            setLogs(response.data);
            setTotal(response.total);
        } catch (err) {
            console.error('Failed to load audit logs:', err);
        } finally {
            setLoading(false);
        }
    };

    const eventTypes = [
        { value: '', label: 'All Events' },
        { value: 'register', label: 'User Registration' },
        { value: 'login', label: 'User Login' },
        { value: 'agent.create', label: 'Agent Created' },
        { value: 'credential.create', label: 'Credential Created' },
        { value: 'credential.read', label: 'Credential Accessed' },
        { value: 'credential.rotate', label: 'Credential Rotated' },
    ];

    const columns = [
        {
            key: 'event_type' as keyof AuditEvent,
            header: 'Event',
            render: (item: AuditEvent) => (
                <span className="font-medium text-gray-900">{item.event_type}</span>
            ),
        },
        {
            key: 'resource_type' as keyof AuditEvent,
            header: 'Resource',
            render: (item: AuditEvent) => (
                <span className="text-gray-600">
                    {item.resource_type || '-'}
                    {item.resource_id && (
                        <span className="ml-1 font-mono text-xs text-gray-400">
                            ({item.resource_id.slice(0, 8)}...)
                        </span>
                    )}
                </span>
            ),
        },
        {
            key: 'details' as keyof AuditEvent,
            header: 'Details',
            render: (item: AuditEvent) => (
                <span className="text-gray-600 text-sm">{item.details || '-'}</span>
            ),
        },
        {
            key: 'ip_address' as keyof AuditEvent,
            header: 'IP Address',
            render: (item: AuditEvent) => (
                <span className="font-mono text-sm text-gray-500">
                    {item.ip_address || '-'}
                </span>
            ),
        },
        {
            key: 'created_at' as keyof AuditEvent,
            header: 'Timestamp',
            render: (item: AuditEvent) => (
                <span className="text-sm text-gray-500">
                    {new Date(item.created_at).toLocaleString()}
                </span>
            ),
        },
    ];

    return (
        <DashboardLayout>
            <div className="space-y-6">
                <div>
                    <h1 className="text-2xl font-bold text-gray-900">Audit Log</h1>
                    <p className="text-gray-500">View all security and access events</p>
                </div>

                {/* Filters */}
                <Card className="p-4">
                    <div className="flex flex-wrap gap-4">
                        <div className="flex-1 min-w-[200px]">
                            <div className="relative">
                                <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-gray-400" />
                                <input
                                    type="text"
                                    placeholder="Search events..."
                                    value={search}
                                    onChange={(e) => setSearch(e.target.value)}
                                    className="w-full pl-10 pr-4 py-2 rounded-lg border border-gray-300 focus:outline-none focus:ring-2 focus:ring-teal-500"
                                />
                            </div>
                        </div>
                        <div className="flex items-center gap-2">
                            <Filter className="h-4 w-4 text-gray-400" />
                            <select
                                value={eventType}
                                onChange={(e) => setEventType(e.target.value)}
                                className="px-3 py-2 rounded-lg border border-gray-300 focus:outline-none focus:ring-2 focus:ring-teal-500"
                            >
                                {eventTypes.map((type) => (
                                    <option key={type.value} value={type.value}>
                                        {type.label}
                                    </option>
                                ))}
                            </select>
                        </div>
                    </div>
                </Card>

                {/* Table */}
                {loading ? (
                    <div className="flex items-center justify-center h-64">
                        <Spinner size="lg" />
                    </div>
                ) : (
                    <Table
                        data={logs}
                        columns={columns}
                        keyExtractor={(item) => item.id.toString()}
                        emptyMessage="No audit events found"
                        pagination={{
                            page,
                            limit,
                            total,
                            onPageChange: setPage,
                        }}
                    />
                )}
            </div>
        </DashboardLayout>
    );
}
