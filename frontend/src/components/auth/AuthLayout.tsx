import { Link } from 'react-router-dom';

interface AuthLayoutProps {
    children: React.ReactNode;
    title: string;
    subtitle: string;
    linkText: string;
    linkTo: string;
}

export default function AuthLayout({
    children,
    title,
    subtitle,
    linkText,
    linkTo,
}: AuthLayoutProps) {
    return (
        <div className="flex min-h-screen flex-col justify-center bg-gray-50 py-12 sm:px-6 lg:px-8">
            <div className="sm:mx-auto sm:w-full sm:max-w-md">
                <div className="flex justify-center">
                    {/* Logo placeholder */}
                    <div className="h-12 w-12 rounded-lg bg-primary flex items-center justify-center text-primary-foreground font-bold text-xl">AK</div>
                </div>
                <h2 className="mt-6 text-center text-3xl font-bold tracking-tight text-foreground">
                    {title}
                </h2>
                <p className="mt-2 text-center text-sm text-muted-foreground">
                    {subtitle}{' '}
                    <Link to={linkTo} className="font-medium text-primary hover:text-primary/80">
                        {linkText}
                    </Link>
                </p>
            </div>

            <div className="mt-8 sm:mx-auto sm:w-full sm:max-w-md">
                <div className="bg-card px-4 py-8 shadow sm:rounded-lg sm:px-10 border border-border">
                    {children}
                </div>
            </div>
        </div>
    );
}
