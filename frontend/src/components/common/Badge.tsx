import { cn } from '../../lib/utils';

interface BadgeProps {
    children: React.ReactNode;
    variant?: 'success' | 'warning' | 'danger' | 'info' | 'gray' | 'primary' | 'secondary';
    className?: string;
}

export default function Badge({ children, variant = 'gray', className }: BadgeProps) {
    const variants = {
        success: 'bg-green-100 text-green-700 border-green-200',
        warning: 'bg-yellow-100 text-yellow-700 border-yellow-200',
        danger: 'bg-destructive/15 text-destructive border-destructive/50',
        info: 'bg-blue-100 text-blue-700 border-blue-200',
        gray: 'bg-gray-100 text-gray-700 border-gray-200',
        primary: 'bg-teal-100 text-teal-700 border-teal-200',
        secondary: 'bg-gray-100 text-gray-600 border-gray-200',
    };

    return (
        <span className={cn(
            "inline-flex items-center rounded-full border px-2.5 py-0.5 text-xs font-semibold transition-colors",
            variants[variant],
            className
        )}>
            {children}
        </span>
    );
}
