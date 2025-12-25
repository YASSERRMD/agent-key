import { Moon, Sun, Monitor } from 'lucide-react';
import { useUIStore } from '../../store/uiStore';

export default function ThemeToggle() {
    const { theme, setTheme } = useUIStore();

    const themes = [
        { value: 'light', icon: Sun, label: 'Light' },
        { value: 'dark', icon: Moon, label: 'Dark' },
        { value: 'system', icon: Monitor, label: 'System' },
    ] as const;

    return (
        <div className="flex items-center gap-1 p-1 bg-gray-100 dark:bg-gray-800 rounded-lg">
            {themes.map(({ value, icon: Icon, label }) => (
                <button
                    key={value}
                    onClick={() => setTheme(value)}
                    className={`p-2 rounded-md transition-colors ${theme === value
                            ? 'bg-white dark:bg-gray-700 shadow text-teal-600'
                            : 'text-gray-500 hover:text-gray-700 dark:hover:text-gray-300'
                        }`}
                    title={label}
                >
                    <Icon className="h-4 w-4" />
                </button>
            ))}
        </div>
    );
}
