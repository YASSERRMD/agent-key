import { useState, useEffect } from 'react';
import DashboardLayout from '../components/dashboard/DashboardLayout';
import Card from '../components/common/Card';
import Spinner from '../components/common/Spinner';
import Badge from '../components/common/Badge';
import { auditService, type AuditEvent } from '../services/auditService';
import {
    Search, Filter, Key, LogIn, UserPlus, Bot, Shield,
    RefreshCw, Clock, Activity, ChevronLeft, ChevronRight
} from 'lucide-react';
import useDebounce from '../hooks/useDebounce';

// Event type categories with icons and colors
const eventCategories = {
    authentication: {
        label: 'Authentication',
        icon: LogIn,
        color: 'bg-blue-100 text-blue-600',
        types: ['login', 'logout', 'register', 'password_change']
    },
    credentials: {
        label: 'Credentials',
        icon: Key,
        color: 'bg-green-100 text-green-600',
        types: ['credential.create', 'credential.read', 'credential.update', 'credential.delete', 'credential.rotate']
    },
    agents: {
        label: 'Agents',
        icon: Bot,
        color: 'bg-purple-100 text-purple-600',
        types: ['agent.create', 'agent.update', 'agent.delete']
    },
    tokens: {
        label: 'Token Access',
        icon: Shield,
        color: 'bg-orange-100 text-orange-600',
        types: ['token.generate', 'token.revoke', 'token.access']
    },
    users: {
        label: 'Users',
        icon: UserPlus,
        color: 'bg-teal-100 text-teal-600',
        types: ['user.create', 'user.update', 'user.delete']
    }
};

const getEventIcon = (eventType: string) => {
    if (eventType.includes('login') || eventType.includes('logout') || eventType === 'register') {
        return LogIn;
    }
    if (eventType.includes('credential')) {
        return Key;
    }
    if (eventType.includes('agent')) {
        return Bot;
    }
    if (eventType.includes('token')) {
        return Shield;
    }
    if (eventType.includes('user')) {
        return UserPlus;
    }
    return Activity;
};

const getEventColor = (eventType: string): string => {
    if (eventType.includes('login') || eventType.includes('logout') || eventType === 'register') {
        return 'bg-blue-100 text-blue-600';
    }
    if (eventType.includes('credential.read') || eventType.includes('token')) {
        return 'bg-orange-100 text-orange-600';
    }
    if (eventType.includes('credential')) {
        return 'bg-green-100 text-green-600';
    }
    if (eventType.includes('agent')) {
        return 'bg-purple-100 text-purple-600';
    }
    if (eventType.includes('delete')) {
        return 'bg-red-100 text-red-600';
    }
    return 'bg-gray-100 text-gray-600';
};

const formatEventType = (eventType: string): string => {
    return eventType
        .replace(/\./g, ' ')
        .replace(/_/g, ' ')
        .replace(/\b\w/g, l => l.toUpperCase());
};

const formatTimeAgo = (date: string): string => {
    const now = new Date();
    const then = new Date(date);
    const seconds = Math.floor((now.getTime() - then.getTime()) / 1000);

    if (seconds < 60) return 'Just now';
    if (seconds < 3600) return `${Math.floor(seconds / 60)}m ago`;
    if (seconds < 86400) return `${Math.floor(seconds / 3600)}h ago`;
    if (seconds < 604800) return `${Math.floor(seconds / 86400)}d ago`;
    return then.toLocaleDateString();
};

export default function AuditLogPage() {
    const [logs, setLogs] = useState<AuditEvent[]>([]);
    const [loading, setLoading] = useState(true);
    const [total, setTotal] = useState(0);
    const [page, setPage] = useState(1);
    const [search, setSearch] = useState('');
    const [eventType, setEventType] = useState('');
    const [activeCategory, setActiveCategory] = useState<string | null>(null);
    const limit = 15;

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

    const handleCategoryClick = (category: string) => {
        if (activeCategory === category) {
            setActiveCategory(null);
            setEventType('');
        } else {
            setActiveCategory(category);
            // Set first event type in category
            const cat = eventCategories[category as keyof typeof eventCategories];
            if (cat?.types.length > 0) {
                setEventType(cat.types[0]);
            }
        }
        setPage(1);
    };

    const eventTypes = [
        { value: '', label: 'All Events' },
        { value: 'register', label: 'User Registration' },
        { value: 'login', label: 'User Login' },
        { value: 'logout', label: 'User Logout' },
        { value: 'agent.create', label: 'Agent Created' },
        { value: 'agent.update', label: 'Agent Updated' },
        { value: 'credential.create', label: 'Credential Created' },
        { value: 'credential.read', label: 'Credential Accessed' },
        { value: 'credential.update', label: 'Credential Updated' },
        { value: 'credential.delete', label: 'Credential Deleted' },
        { value: 'token.generate', label: 'Token Generated' },
        { value: 'user.update', label: 'User Updated' },
    ];

    const totalPages = Math.ceil(total / limit);

    return (
        <DashboardLayout>
            <div className="space-y-6">
                {/* Header */}
                <div className="flex items-center justify-between">
                    <div>
                        <h1 className="text-2xl font-bold text-gray-900">Audit Log</h1>
                        <p className="text-gray-500">Track all security and access events</p>
                    </div>
                    <button
                        onClick={loadLogs}
                        className="flex items-center gap-2 px-4 py-2 text-sm font-medium text-gray-600 bg-white border border-gray-300 rounded-lg hover:bg-gray-50"
                    >
                        <RefreshCw className="h-4 w-4" />
                        Refresh
                    </button>
                </div>

                {/* Category Cards */}
                <div className="grid grid-cols-2 md:grid-cols-5 gap-4">
                    {Object.entries(eventCategories).map(([key, config]) => {
                        const Icon = config.icon;
                        const isActive = activeCategory === key;
                        return (
                            <button
                                key={key}
                                onClick={() => handleCategoryClick(key)}
                                className={`p-4 rounded-xl border-2 transition-all ${isActive
                                    ? 'border-teal-500 bg-teal-50'
                                    : 'border-transparent bg-white hover:border-gray-200'
                                    }`}
                            >
                                <div className={`inline-flex p-2 rounded-lg ${config.color}`}>
                                    <Icon className="h-5 w-5" />
                                </div>
                                <h3 className="mt-2 font-semibold text-gray-900">{config.label}</h3>
                                <p className="text-xs text-gray-500">
                                    {config.types.length} event types
                                </p>
                            </button>
                        );
                    })}
                </div>

                {/* Filters */}
                <Card className="p-4">
                    <div className="flex flex-wrap gap-4 items-center">
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
                                onChange={(e) => {
                                    setEventType(e.target.value);
                                    setActiveCategory(null);
                                    setPage(1);
                                }}
                                className="px-3 py-2 rounded-lg border border-gray-300 focus:outline-none focus:ring-2 focus:ring-teal-500"
                            >
                                {eventTypes.map((type) => (
                                    <option key={type.value} value={type.value}>
                                        {type.label}
                                    </option>
                                ))}
                            </select>
                        </div>
                        <div className="text-sm text-gray-500">
                            {total} total events
                        </div>
                    </div>
                </Card>

                {/* Event List */}
                {loading ? (
                    <div className="flex items-center justify-center h-64">
                        <Spinner size="lg" />
                    </div>
                ) : logs.length === 0 ? (
                    <Card className="p-12 text-center">
                        <Activity className="h-12 w-12 text-gray-300 mx-auto mb-4" />
                        <h3 className="text-lg font-medium text-gray-900">No events found</h3>
                        <p className="text-gray-500 mt-1">
                            {eventType ? 'Try selecting a different filter' : 'Events will appear here as they occur'}
                        </p>
                    </Card>
                ) : (
                    <div className="space-y-3">
                        {logs.map((event) => {
                            const Icon = getEventIcon(event.event_type);
                            const colorClass = getEventColor(event.event_type);

                            return (
                                <Card key={event.id} className="p-4 hover:shadow-md transition-shadow">
                                    <div className="flex items-start gap-4">
                                        {/* Icon */}
                                        <div className={`p-2 rounded-lg ${colorClass}`}>
                                            <Icon className="h-5 w-5" />
                                        </div>

                                        {/* Content */}
                                        <div className="flex-1 min-w-0">
                                            <div className="flex items-center gap-2">
                                                <span className="font-medium text-gray-900">
                                                    {formatEventType(event.event_type)}
                                                </span>
                                                {event.resource_type && (
                                                    <Badge variant="gray">
                                                        {event.resource_type}
                                                    </Badge>
                                                )}
                                            </div>

                                            {event.details && (
                                                <p className="text-sm text-gray-600 mt-1">
                                                    {event.details}
                                                </p>
                                            )}

                                            <div className="flex items-center gap-4 mt-2 text-xs text-gray-400">
                                                <span className="flex items-center gap-1">
                                                    <Clock className="h-3 w-3" />
                                                    {formatTimeAgo(event.created_at)}
                                                </span>
                                                {event.ip_address && (
                                                    <span className="font-mono">
                                                        IP: {event.ip_address}
                                                    </span>
                                                )}
                                                {event.resource_id && (
                                                    <span className="font-mono">
                                                        ID: {event.resource_id.slice(0, 8)}...
                                                    </span>
                                                )}
                                            </div>
                                        </div>

                                        {/* Timestamp */}
                                        <div className="text-right text-sm text-gray-500">
                                            {new Date(event.created_at).toLocaleString()}
                                        </div>
                                    </div>
                                </Card>
                            );
                        })}

                        {/* Pagination */}
                        {totalPages > 1 && (
                            <div className="flex items-center justify-between pt-4">
                                <p className="text-sm text-gray-500">
                                    Showing {(page - 1) * limit + 1} to {Math.min(page * limit, total)} of {total} events
                                </p>
                                <div className="flex items-center gap-2">
                                    <button
                                        onClick={() => setPage(p => Math.max(1, p - 1))}
                                        disabled={page === 1}
                                        className="p-2 rounded-lg border border-gray-300 hover:bg-gray-50 disabled:opacity-50 disabled:cursor-not-allowed"
                                    >
                                        <ChevronLeft className="h-4 w-4" />
                                    </button>
                                    <span className="px-3 py-1 text-sm">
                                        Page {page} of {totalPages}
                                    </span>
                                    <button
                                        onClick={() => setPage(p => Math.min(totalPages, p + 1))}
                                        disabled={page === totalPages}
                                        className="p-2 rounded-lg border border-gray-300 hover:bg-gray-50 disabled:opacity-50 disabled:cursor-not-allowed"
                                    >
                                        <ChevronRight className="h-4 w-4" />
                                    </button>
                                </div>
                            </div>
                        )}
                    </div>
                )}
            </div>
        </DashboardLayout>
    );
}
