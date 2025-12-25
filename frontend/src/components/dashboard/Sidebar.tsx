import { useLocation, Link } from 'react-router-dom';
import { cn } from '../../lib/utils';
import {
    LayoutDashboard,
    Users,
    Key,
    Settings,
    ChevronLeft,
    ChevronRight,
    LogOut
} from 'lucide-react';
import { useAuth } from '../../hooks/useAuth';

interface SidebarProps {
    collapsed: boolean;
    onToggle: () => void;
}

export default function Sidebar({ collapsed, onToggle }: SidebarProps) {
    const location = useLocation();
    const { logout } = useAuth();

    const navItems = [
        { title: 'Dashboard', icon: LayoutDashboard, path: '/' },
        { title: 'Agents', icon: Users, path: '/agents' },
        { title: 'Credentials', icon: Key, path: '/credentials' },
        { title: 'Settings', icon: Settings, path: '/settings' },
    ];

    return (
        <div className={cn(
            "flex flex-col border-r bg-white transition-all duration-300",
            collapsed ? "w-16" : "w-64"
        )}>
            <div className="flex h-16 items-center justify-between px-4 border-b">
                <div className="flex items-center gap-2">
                    <img src="/logo.png" alt="AgentKey" className="h-8 w-8" />
                    {!collapsed && <span className="text-xl font-bold text-primary">AgentKey</span>}
                </div>
                <button
                    onClick={onToggle}
                    className="rounded-md p-1 hover:bg-gray-100 transition-colors"
                >
                    {collapsed ? <ChevronRight size={20} /> : <ChevronLeft size={20} />}
                </button>
            </div>

            <nav className="flex-1 space-y-1 p-2">
                {navItems.map((item) => {
                    const isActive = location.pathname === item.path;
                    return (
                        <Link
                            key={item.path}
                            to={item.path}
                            className={cn(
                                "flex items-center rounded-md px-3 py-2 text-sm font-medium transition-colors",
                                isActive
                                    ? "bg-primary text-primary-foreground"
                                    : "text-gray-700 hover:bg-gray-100",
                                collapsed && "justify-center px-2"
                            )}
                            title={collapsed ? item.title : undefined}
                        >
                            <item.icon size={20} className={cn(!collapsed && "mr-3")} />
                            {!collapsed && <span>{item.title}</span>}
                        </Link>
                    );
                })}
            </nav>

            <div className="border-t p-2">
                <button
                    onClick={logout}
                    className={cn(
                        "flex w-full items-center rounded-md px-3 py-2 text-sm font-medium text-destructive hover:bg-destructive/10 transition-colors",
                        collapsed && "justify-center px-2"
                    )}
                    title={collapsed ? "Logout" : undefined}
                >
                    <LogOut size={20} className={cn(!collapsed && "mr-3")} />
                    {!collapsed && <span>Logout</span>}
                </button>
            </div>
        </div>
    );
}
