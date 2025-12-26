import { useState, useRef, useEffect } from 'react';
import { Link, useNavigate } from 'react-router-dom';
import { Bell, Search, User, Settings, LogOut, ChevronDown } from 'lucide-react';
import { useAuth } from '../../hooks/useAuth';

export default function Navbar() {
    const { user, logout } = useAuth();
    const navigate = useNavigate();
    const [searchQuery, setSearchQuery] = useState('');
    const [showNotifications, setShowNotifications] = useState(false);
    const [showUserMenu, setShowUserMenu] = useState(false);
    const notificationRef = useRef<HTMLDivElement>(null);
    const userMenuRef = useRef<HTMLDivElement>(null);

    // Close dropdowns when clicking outside
    useEffect(() => {
        function handleClickOutside(event: MouseEvent) {
            if (notificationRef.current && !notificationRef.current.contains(event.target as Node)) {
                setShowNotifications(false);
            }
            if (userMenuRef.current && !userMenuRef.current.contains(event.target as Node)) {
                setShowUserMenu(false);
            }
        }
        document.addEventListener('mousedown', handleClickOutside);
        return () => document.removeEventListener('mousedown', handleClickOutside);
    }, []);

    const handleSearch = (e: React.FormEvent) => {
        e.preventDefault();
        if (searchQuery.trim()) {
            navigate(`/agents?search=${encodeURIComponent(searchQuery.trim())}`);
        }
    };

    const handleLogout = () => {
        logout();
        navigate('/login');
    };

    // Mock notifications - in real app, fetch from API
    const notifications = [
        { id: 1, message: 'Credential rotation completed', time: '5 min ago', read: false },
        { id: 2, message: 'New team member joined', time: '1 hour ago', read: true },
        { id: 3, message: 'API key expiring soon', time: '2 hours ago', read: true },
    ];

    const unreadCount = notifications.filter(n => !n.read).length;

    return (
        <header className="h-16 border-b bg-white flex items-center justify-between px-6 sticky top-0 z-10">
            {/* Search */}
            <form onSubmit={handleSearch} className="flex items-center flex-1 max-w-md">
                <div className="relative w-full">
                    <Search className="absolute left-3 top-1/2 -translate-y-1/2 text-gray-400 h-4 w-4" />
                    <input
                        type="text"
                        value={searchQuery}
                        onChange={(e) => setSearchQuery(e.target.value)}
                        placeholder="Search agents, credentials..."
                        className="w-full bg-gray-50 border border-gray-200 rounded-md pl-10 pr-4 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-primary focus:border-transparent transition-all"
                    />
                </div>
            </form>

            <div className="flex items-center gap-4">
                {/* Notifications */}
                <div ref={notificationRef} className="relative">
                    <button
                        onClick={() => setShowNotifications(!showNotifications)}
                        className="relative p-2 text-gray-500 hover:bg-gray-100 rounded-full transition-colors"
                    >
                        <Bell size={20} />
                        {unreadCount > 0 && (
                            <span className="absolute top-1 right-1 h-4 w-4 bg-red-500 rounded-full text-[10px] font-bold text-white flex items-center justify-center">
                                {unreadCount}
                            </span>
                        )}
                    </button>

                    {showNotifications && (
                        <div className="absolute right-0 mt-2 w-80 bg-white border border-gray-200 rounded-lg shadow-lg py-2 z-50">
                            <div className="px-4 py-2 border-b border-gray-100">
                                <h3 className="font-semibold text-sm">Notifications</h3>
                            </div>
                            {notifications.length === 0 ? (
                                <div className="px-4 py-6 text-center text-gray-500 text-sm">
                                    No notifications
                                </div>
                            ) : (
                                <div className="max-h-64 overflow-y-auto">
                                    {notifications.map((notification) => (
                                        <div
                                            key={notification.id}
                                            className={`px-4 py-3 hover:bg-gray-50 cursor-pointer border-l-2 ${notification.read ? 'border-transparent' : 'border-teal-500 bg-teal-50/50'
                                                }`}
                                        >
                                            <p className="text-sm text-gray-800">{notification.message}</p>
                                            <p className="text-xs text-gray-400 mt-1">{notification.time}</p>
                                        </div>
                                    ))}
                                </div>
                            )}
                            <div className="px-4 py-2 border-t border-gray-100">
                                <Link
                                    to="/audit"
                                    className="text-sm text-teal-600 hover:text-teal-500"
                                    onClick={() => setShowNotifications(false)}
                                >
                                    View all activity
                                </Link>
                            </div>
                        </div>
                    )}
                </div>

                {/* User Menu */}
                <div ref={userMenuRef} className="relative">
                    <button
                        onClick={() => setShowUserMenu(!showUserMenu)}
                        className="flex items-center gap-3 pl-4 border-l hover:bg-gray-50 rounded-lg px-3 py-1.5 transition-colors"
                    >
                        <div className="text-right hidden sm:block">
                            <div className="text-sm font-medium text-gray-900">{user?.name || user?.email?.split('@')[0]}</div>
                            <div className="text-xs text-gray-500 capitalize">{user?.role}</div>
                        </div>
                        <div className="h-8 w-8 rounded-full bg-teal-100 flex items-center justify-center text-teal-600 font-semibold">
                            {(user?.name || user?.email || 'U').charAt(0).toUpperCase()}
                        </div>
                        <ChevronDown size={14} className="text-gray-400" />
                    </button>

                    {showUserMenu && (
                        <div className="absolute right-0 mt-2 w-56 bg-white border border-gray-200 rounded-lg shadow-lg py-2 z-50">
                            <div className="px-4 py-3 border-b border-gray-100">
                                <p className="text-sm font-medium text-gray-900">{user?.name || 'User'}</p>
                                <p className="text-xs text-gray-500 truncate">{user?.email}</p>
                            </div>
                            <div className="py-1">
                                <Link
                                    to="/settings/profile"
                                    className="flex items-center gap-3 px-4 py-2 text-sm text-gray-700 hover:bg-gray-50"
                                    onClick={() => setShowUserMenu(false)}
                                >
                                    <User size={16} />
                                    Your Profile
                                </Link>
                                <Link
                                    to="/settings"
                                    className="flex items-center gap-3 px-4 py-2 text-sm text-gray-700 hover:bg-gray-50"
                                    onClick={() => setShowUserMenu(false)}
                                >
                                    <Settings size={16} />
                                    Settings
                                </Link>
                            </div>
                            <div className="border-t border-gray-100 py-1">
                                <button
                                    onClick={handleLogout}
                                    className="flex items-center gap-3 w-full px-4 py-2 text-sm text-red-600 hover:bg-red-50"
                                >
                                    <LogOut size={16} />
                                    Sign out
                                </button>
                            </div>
                        </div>
                    )}
                </div>
            </div>
        </header>
    );
}
