import { cn } from '../../lib/utils';

interface SpinnerProps {
    size?: 'sm' | 'md' | 'lg';
    className?: string;
}

export default function Spinner({ size = 'md', className }: SpinnerProps) {
    const sizes = {
        sm: 'h-4 w-4 border-2',
        md: 'h-8 w-8 border-3',
        lg: 'h-12 w-12 border-4',
    };

    return (
        <div
            className={cn(
                'animate-spin rounded-full border-solid border-primary border-t-transparent',
                sizes[size],
                className
            )}
            role="status"
        >
            <span className="sr-only">Loading...</span>
        </div>
    );
}
