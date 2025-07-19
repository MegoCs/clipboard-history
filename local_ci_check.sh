#!/bin/bash
# Local CI validation script
# Run this before pushing to ensure pipeline will pass

set -e  # Exit on any error

echo "🧪 Running local CI validation..."
echo

echo "📝 Checking code formatting..."
cargo fmt --check
echo "✅ Formatting check passed"
echo

echo "🔍 Running Clippy linter..."
cargo clippy --all-targets --all-features -- -D warnings
echo "✅ Clippy check passed"
echo

echo "🧪 Running tests..."
cargo test --verbose
echo "✅ Tests passed"
echo

echo "🔨 Building debug version..."
cargo build --verbose
echo "✅ Debug build successful"
echo

echo "🚀 Building release version..."
cargo build --release --verbose
echo "✅ Release build successful"
echo

echo "🎉 All local CI checks passed! Ready to push."
echo
echo "Your changes should pass the GitHub Actions pipeline."
echo "Binary location: ./target/release/clipboard-history"

# Check if release binary exists and is executable
if [ -f "./target/release/clipboard-history" ] || [ -f "./target/release/clipboard-history.exe" ]; then
    echo
    echo "📦 Release binary ready for distribution"
    ls -la target/release/clipboard-history* 2>/dev/null || ls -la target/release/clipboard-history.exe 2>/dev/null
fi
