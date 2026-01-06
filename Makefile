.PHONY: fmt lint test check build clean hook-setup

# 1. Format Code (Fix)
fmt:
	cargo fmt
	prettier --write .

# 2. Check Code (Read-only)
lint:
	cargo fmt -- --check
	cargo clippy -- -D warnings
	prettier --check .

# 3. Test
test:
	cargo test

# 4. Meta-task for CI/Pre-commit
check: lint test

# 5. Build
build:
	cargo build --release

# 6. Setup Git Hooks
hook-setup:
	git config core.hooksPath .githooks
	@echo "Git hooks configured."

clean:
	cargo clean
