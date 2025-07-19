@echo off
REM Local CI validation script for Windows
REM Run this before pushing to ensure pipeline will pass

echo 🧪 Running local CI validation...
echo.

echo 📝 Checking code formatting...
cargo fmt --check
if %errorlevel% neq 0 (
    echo ❌ Formatting check failed
    exit /b 1
)
echo ✅ Formatting check passed
echo.

echo 🔍 Running Clippy linter...
cargo clippy --all-targets --all-features -- -D warnings
if %errorlevel% neq 0 (
    echo ❌ Clippy check failed
    exit /b 1
)
echo ✅ Clippy check passed
echo.

echo 🧪 Running tests...
cargo test --verbose
if %errorlevel% neq 0 (
    echo ❌ Tests failed
    exit /b 1
)
echo ✅ Tests passed
echo.

echo 🔨 Building debug version...
cargo build --verbose
if %errorlevel% neq 0 (
    echo ❌ Debug build failed
    exit /b 1
)
echo ✅ Debug build successful
echo.

echo 🚀 Building release version...
cargo build --release --verbose
if %errorlevel% neq 0 (
    echo ❌ Release build failed
    exit /b 1
)
echo ✅ Release build successful
echo.

echo 🎉 All local CI checks passed! Ready to push.
echo.
echo Your changes should pass the GitHub Actions pipeline.
echo Binary location: .\target\release\clipboard-history.exe

REM Check if release binary exists
if exist ".\target\release\clipboard-history.exe" (
    echo.
    echo 📦 Release binary ready for distribution
    dir ".\target\release\clipboard-history.exe"
)

pause
