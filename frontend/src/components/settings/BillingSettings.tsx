import Card from '../common/Card';
import Badge from '../common/Badge';
import { Package, TrendingUp } from 'lucide-react';

export default function BillingSettings() {
    return (
        <div className="space-y-6">
            {/* Current Plan */}
            <Card className="p-6">
                <div className="flex items-center justify-between mb-4">
                    <div>
                        <h3 className="text-lg font-semibold">Current Plan</h3>
                        <p className="text-gray-500 text-sm">Your subscription details</p>
                    </div>
                    <Badge variant="success">Active</Badge>
                </div>
                <div className="flex items-center gap-6">
                    <div className="p-4 bg-teal-50 rounded-lg">
                        <Package className="h-8 w-8 text-teal-600" />
                    </div>
                    <div>
                        <p className="text-2xl font-bold">Self-Hosted</p>
                        <p className="text-gray-500">Unlimited local usage</p>
                    </div>
                </div>
            </Card>

            {/* Usage Stats */}
            <Card className="p-6">
                <h3 className="text-lg font-semibold mb-4">
                    <TrendingUp className="inline h-5 w-5 mr-2" />
                    Usage Overview
                </h3>
                <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
                    <div className="p-4 bg-gray-50 rounded-lg">
                        <p className="text-sm text-gray-500">Agents</p>
                        <p className="text-2xl font-bold text-gray-900">Unlimited</p>
                        <p className="text-xs text-gray-400 mt-1">No restrictions</p>
                    </div>
                    <div className="p-4 bg-gray-50 rounded-lg">
                        <p className="text-sm text-gray-500">Credentials</p>
                        <p className="text-2xl font-bold text-gray-900">Unlimited</p>
                        <p className="text-xs text-gray-400 mt-1">No restrictions</p>
                    </div>
                    <div className="p-4 bg-gray-50 rounded-lg">
                        <p className="text-sm text-gray-500">API Calls</p>
                        <p className="text-2xl font-bold text-gray-900">Unlimited</p>
                        <p className="text-xs text-gray-400 mt-1">No restrictions</p>
                    </div>
                </div>
            </Card>

            {/* Info */}
            <Card className="p-6 bg-blue-50 border-blue-200">
                <div className="flex items-start gap-3">
                    <div className="p-2 bg-blue-100 rounded-lg">
                        <Package className="h-5 w-5 text-blue-600" />
                    </div>
                    <div>
                        <h4 className="font-semibold text-blue-900">Self-Hosted Edition</h4>
                        <p className="text-sm text-blue-700 mt-1">
                            You're running AgentKey locally. All features are available with no usage limits.
                            This instance is for personal or internal business use only.
                        </p>
                    </div>
                </div>
            </Card>
        </div>
    );
}
