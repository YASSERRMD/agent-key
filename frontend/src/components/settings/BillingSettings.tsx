import Card from '../common/Card';
import Button from '../common/Button';
import Badge from '../common/Badge';
import { CreditCard, Package, TrendingUp, Check } from 'lucide-react';

const plans = [
    {
        name: 'Free',
        price: '$0',
        period: 'forever',
        features: ['5 Agents', '100 Credentials', '1,000 API calls/month', 'Community support'],
        current: true,
    },
    {
        name: 'Pro',
        price: '$29',
        period: '/month',
        features: ['Unlimited Agents', 'Unlimited Credentials', '100,000 API calls/month', 'Priority support', 'SSO'],
        current: false,
        recommended: true,
    },
    {
        name: 'Enterprise',
        price: 'Custom',
        period: '',
        features: ['Everything in Pro', 'Dedicated support', 'SLA guarantee', 'Custom integrations', 'On-premise option'],
        current: false,
    },
];

export default function BillingSettings() {
    return (
        <div className="space-y-6">
            {/* Current Plan */}
            <Card className="p-6">
                <div className="flex items-center justify-between mb-4">
                    <div>
                        <h3 className="text-lg font-semibold">Current Plan</h3>
                        <p className="text-gray-500 text-sm">Manage your subscription and billing</p>
                    </div>
                    <Badge variant="success">Active</Badge>
                </div>
                <div className="flex items-center gap-6">
                    <div className="p-4 bg-teal-50 rounded-lg">
                        <Package className="h-8 w-8 text-teal-600" />
                    </div>
                    <div>
                        <p className="text-2xl font-bold">Free Plan</p>
                        <p className="text-gray-500">Perfect for getting started</p>
                    </div>
                </div>
            </Card>

            {/* Usage Stats */}
            <Card className="p-6">
                <h3 className="text-lg font-semibold mb-4">
                    <TrendingUp className="inline h-5 w-5 mr-2" />
                    Usage This Month
                </h3>
                <div className="grid grid-cols-3 gap-6">
                    <div>
                        <p className="text-sm text-gray-500">Agents</p>
                        <p className="text-2xl font-bold">2 / 5</p>
                        <div className="mt-2 h-2 bg-gray-200 rounded-full overflow-hidden">
                            <div className="h-full bg-teal-500 rounded-full" style={{ width: '40%' }} />
                        </div>
                    </div>
                    <div>
                        <p className="text-sm text-gray-500">Credentials</p>
                        <p className="text-2xl font-bold">15 / 100</p>
                        <div className="mt-2 h-2 bg-gray-200 rounded-full overflow-hidden">
                            <div className="h-full bg-teal-500 rounded-full" style={{ width: '15%' }} />
                        </div>
                    </div>
                    <div>
                        <p className="text-sm text-gray-500">API Calls</p>
                        <p className="text-2xl font-bold">450 / 1,000</p>
                        <div className="mt-2 h-2 bg-gray-200 rounded-full overflow-hidden">
                            <div className="h-full bg-teal-500 rounded-full" style={{ width: '45%' }} />
                        </div>
                    </div>
                </div>
            </Card>

            {/* Plans */}
            <div>
                <h3 className="text-lg font-semibold mb-4">Available Plans</h3>
                <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
                    {plans.map((plan) => (
                        <Card
                            key={plan.name}
                            className={`p-6 relative ${plan.recommended ? 'border-teal-500 border-2' : ''}`}
                        >
                            {plan.recommended && (
                                <div className="absolute -top-3 left-1/2 -translate-x-1/2">
                                    <Badge variant="primary">Recommended</Badge>
                                </div>
                            )}
                            <div className="text-center mb-6">
                                <h4 className="text-xl font-bold">{plan.name}</h4>
                                <p className="text-3xl font-bold mt-2">
                                    {plan.price}
                                    <span className="text-sm font-normal text-gray-500">{plan.period}</span>
                                </p>
                            </div>
                            <ul className="space-y-3 mb-6">
                                {plan.features.map((feature) => (
                                    <li key={feature} className="flex items-center gap-2 text-sm">
                                        <Check className="h-4 w-4 text-teal-500" />
                                        {feature}
                                    </li>
                                ))}
                            </ul>
                            <Button
                                variant={plan.current ? 'secondary' : 'primary'}
                                className="w-full"
                                disabled={plan.current}
                            >
                                {plan.current ? 'Current Plan' : plan.price === 'Custom' ? 'Contact Sales' : 'Upgrade'}
                            </Button>
                        </Card>
                    ))}
                </div>
            </div>

            {/* Payment Method */}
            <Card className="p-6">
                <h3 className="text-lg font-semibold mb-4">
                    <CreditCard className="inline h-5 w-5 mr-2" />
                    Payment Method
                </h3>
                <p className="text-gray-500 mb-4">No payment method on file (Free plan)</p>
                <Button variant="secondary">Add Payment Method</Button>
            </Card>
        </div>
    );
}
