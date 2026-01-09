.PHONY: fmt lint test check build install install-completions uninstall clean hook-setup

# Install paths (override with PREFIX=/usr for system install)
PREFIX ?= /usr/local
DESTDIR ?=
BINDIR = $(DESTDIR)$(PREFIX)/bin
BASH_COMPLETION_DIR = $(DESTDIR)$(PREFIX)/share/bash-completion/completions
ZSH_COMPLETION_DIR = $(DESTDIR)$(PREFIX)/share/zsh/site-functions
FISH_COMPLETION_DIR = $(DESTDIR)$(PREFIX)/share/fish/vendor_completions.d

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

# 6. Install (use sudo for system install, or PREFIX=~/.local for user install)
install: build
	install -Dm755 target/release/theman $(BINDIR)/theman
ifeq ($(PREFIX),/usr)
	$(MAKE) install-completions
else ifeq ($(PREFIX),/usr/local)
	$(MAKE) install-completions
else
	@echo "Note: Shell completions not installed for user prefix."
	@echo "Use 'eval \"\$$(theman completions bash)\"' in your shell rc file."
endif

install-completions:
	install -dm755 $(BASH_COMPLETION_DIR)
	install -dm755 $(ZSH_COMPLETION_DIR)
	install -dm755 $(FISH_COMPLETION_DIR)
	target/release/theman completions bash > $(BASH_COMPLETION_DIR)/theman
	target/release/theman completions zsh > $(ZSH_COMPLETION_DIR)/_theman
	target/release/theman completions fish > $(FISH_COMPLETION_DIR)/theman.fish

# 7. Uninstall
uninstall:
	rm -f $(BINDIR)/theman
	rm -f $(BASH_COMPLETION_DIR)/theman
	rm -f $(ZSH_COMPLETION_DIR)/_theman
	rm -f $(FISH_COMPLETION_DIR)/theman.fish

# 8. Setup Git Hooks
hook-setup:
	git config core.hooksPath .githooks
	@echo "Git hooks configured."

clean:
	cargo clean
