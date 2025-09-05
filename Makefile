# aura_rs Development Makefile

.PHONY: help init ci

# Default target
help:
	@echo "aura_rs Development Commands:"
	@echo ""
	@echo "  init          Initialize repository (install hooks and dev dependencies)"
	@echo "  ci            Run all CI checks (fmt, clippy, check, test, doc)"
	@echo "  help          Show this help message"

# Initialize repository for development
init:
	@echo "🚀 Initializing aura_rs repository for development..."
	@echo ""
	@echo "🔧 Setting up Git hooks..."
	@chmod +x .githooks/pre-push
	@git config core.hooksPath .githooks
	@echo "✅ Git hooks installed!"
	@echo ""
	@echo "📦 Installing development dependencies..."
	@cargo install cargo-audit cargo-outdated cargo-expand
	@echo "✅ Development dependencies installed!"
	@echo ""
	@echo "📝 Available hooks:"
	@echo "   • pre-push: Runs cargo fmt, clippy, check, and tests before pushing"
	@echo ""
	@echo "🔧 Usage tips:"
	@echo "   • Run 'make ci' to run all quality checks"
	@echo "   • To skip tests during push: SKIP_TESTS=1 git push"
	@echo "   • To bypass all hooks: git push --no-verify"
	@echo ""
	@echo "🎉 Repository initialization complete!"

# Run all CI checks
ci:
	@echo "🔍 Running CI checks..."
	@echo ""
	@echo "🎨 Formatting code..."
	@cargo fmt --all
	@echo "✅ Code formatted"
	@echo ""
	@echo "📎 Running clippy..."
	@cargo clippy --workspace --all-targets --all-features -- -D warnings
	@echo "✅ Clippy checks passed"
	@echo ""
	@echo "🔧 Checking compilation..."
	@cargo check --workspace --all-targets --all-features
	@echo "✅ Compilation check passed"
	@echo ""
	@echo "🧪 Running tests..."
	@cargo test --workspace --all-features
	@echo "✅ All tests passed"
	@echo ""
	@echo "📚 Generating documentation..."
	@cargo doc --workspace --all-features --no-deps
	@echo "✅ Documentation generated"
	@echo ""
	@echo "🎉 All CI checks passed!"
