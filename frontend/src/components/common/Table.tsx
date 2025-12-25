import { ChevronLeft, ChevronRight } from 'lucide-react';

interface Column<T> {
    key: keyof T | string;
    header: string;
    render?: (item: T) => React.ReactNode;
    className?: string;
}

interface TableProps<T> {
    data: T[];
    columns: Column<T>[];
    keyExtractor: (item: T) => string;
    loading?: boolean;
    emptyMessage?: string;
    pagination?: {
        page: number;
        limit: number;
        total: number;
        onPageChange: (page: number) => void;
    };
}

export default function Table<T>({
    data,
    columns,
    keyExtractor,
    loading = false,
    emptyMessage = 'No data available',
    pagination,
}: TableProps<T>) {
    const totalPages = pagination ? Math.ceil(pagination.total / pagination.limit) : 1;

    const getValue = (item: T, key: keyof T | string): React.ReactNode => {
        if (typeof key === 'string' && key.includes('.')) {
            const keys = key.split('.');
            let value: unknown = item;
            for (const k of keys) {
                value = (value as Record<string, unknown>)?.[k];
            }
            return value as React.ReactNode;
        }
        return item[key as keyof T] as React.ReactNode;
    };

    if (loading) {
        return (
            <div className="bg-white rounded-xl border border-gray-200 overflow-hidden">
                <div className="animate-pulse">
                    <div className="h-12 bg-gray-100 border-b" />
                    {[1, 2, 3].map((i) => (
                        <div key={i} className="h-16 border-b flex items-center px-6 gap-4">
                            <div className="h-4 bg-gray-200 rounded w-1/4" />
                            <div className="h-4 bg-gray-200 rounded w-1/3" />
                            <div className="h-4 bg-gray-200 rounded w-1/4" />
                        </div>
                    ))}
                </div>
            </div>
        );
    }

    if (data.length === 0) {
        return (
            <div className="bg-white rounded-xl border border-gray-200 p-8 text-center">
                <p className="text-gray-500">{emptyMessage}</p>
            </div>
        );
    }

    return (
        <div className="bg-white rounded-xl border border-gray-200 overflow-hidden">
            <div className="overflow-x-auto">
                <table className="w-full">
                    <thead>
                        <tr className="bg-gray-50 border-b border-gray-200">
                            {columns.map((column) => (
                                <th
                                    key={String(column.key)}
                                    className={`px-6 py-3 text-left text-xs font-semibold text-gray-600 uppercase tracking-wider ${column.className || ''}`}
                                >
                                    {column.header}
                                </th>
                            ))}
                        </tr>
                    </thead>
                    <tbody className="divide-y divide-gray-200">
                        {data.map((item) => (
                            <tr key={keyExtractor(item)} className="hover:bg-gray-50 transition-colors">
                                {columns.map((column) => (
                                    <td
                                        key={String(column.key)}
                                        className={`px-6 py-4 text-sm text-gray-900 ${column.className || ''}`}
                                    >
                                        {column.render ? column.render(item) : getValue(item, column.key)}
                                    </td>
                                ))}
                            </tr>
                        ))}
                    </tbody>
                </table>
            </div>

            {pagination && totalPages > 1 && (
                <div className="px-6 py-3 border-t border-gray-200 flex items-center justify-between">
                    <span className="text-sm text-gray-600">
                        Showing {(pagination.page - 1) * pagination.limit + 1} to{' '}
                        {Math.min(pagination.page * pagination.limit, pagination.total)} of {pagination.total}
                    </span>
                    <div className="flex items-center gap-2">
                        <button
                            onClick={() => pagination.onPageChange(pagination.page - 1)}
                            disabled={pagination.page === 1}
                            className="p-2 rounded-lg border border-gray-300 hover:bg-gray-50 disabled:opacity-50 disabled:cursor-not-allowed"
                        >
                            <ChevronLeft className="h-4 w-4" />
                        </button>
                        <span className="text-sm text-gray-600">
                            Page {pagination.page} of {totalPages}
                        </span>
                        <button
                            onClick={() => pagination.onPageChange(pagination.page + 1)}
                            disabled={pagination.page >= totalPages}
                            className="p-2 rounded-lg border border-gray-300 hover:bg-gray-50 disabled:opacity-50 disabled:cursor-not-allowed"
                        >
                            <ChevronRight className="h-4 w-4" />
                        </button>
                    </div>
                </div>
            )}
        </div>
    );
}
