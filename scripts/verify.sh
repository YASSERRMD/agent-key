#!/bin/bash
# AgentKey Phase 0 Verification Script

set -e

echo "🔍 Running AgentKey Phase 0 Verification..."
echo ""

# Check Rust version
echo "📦 Rust version:"
rustc --version
cargo --version
echo ""

# Format check
echo "📝 Checking code format..."
cargo fmt --check || echo "⚠️  Format issues found (run 'cargo fmt' to fix)"
echo ""

# Clippy lints
echo "🔍 Running clippy..."
cargo clippy -- -D warnings 2>/dev/null || echo "⚠️  Clippy warnings found"
echo ""

# Build check
echo "🔨 Building project..."
cargo build --release
echo "✅ Build successful"
echo ""

# Run tests
echo "🧪 Running tests..."
cargo test -- --test-threads=1
echo "✅ All tests passed"
echo ""

# Summary
echo "🎉 Phase 0 Verification Complete!"
echo ""
echo "To start the server:"
echo "  1. docker-compose up -d"
echo "  2. cp .env.example .env"
echo "  3. cargo run"
