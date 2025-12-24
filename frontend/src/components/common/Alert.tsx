import { cn } from '../../lib/utils';
import { AlertCircle, CheckCircle2, Info } from 'lucide-react';

interface AlertProps {
    variant?: 'error' | 'success' | 'warning' | 'info';
    children: React.ReactNode;
    className?: string;
}

export default function Alert({ variant = 'info', children, className }: AlertProps) {
    const variants = {
        error: 'bg-destructive/15 text-destructive border-destructive/50',
        success: 'bg-green-50 text-green-700 border-green-200',
        warning: 'bg-yellow-50 text-yellow-700 border-yellow-200',
        info: 'bg-blue-50 text-blue-700 border-blue-200',
    };

    const icons = {
        error: AlertCircle,
        success: CheckCircle2,
        warning: AlertCircle,
        info: Info,
    };

    const Icon = icons[variant];

    return (
        <div
            className={cn(
                'relative w-full rounded-lg border p-4 [&>svg~*]:pl-7 [&>svg+div]:translate-y-[-3px] [&>svg]:absolute [&>svg]:left-4 [&>svg]:top-4 [&>svg]:text-foreground',
                variants[variant],
                className
            )}
        >
            <Icon className="h-4 w-4" />
            <div className="text-sm font-medium">{children}</div>
        </div>
    );
}
