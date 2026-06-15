.PHONY: fmt lint test check build install install-completions uninstall clean setup setup-hooks setup-tools

# Binary name (matches the package name in Cargo.toml)
BIN := themis

# Required cargo tools
CARGO_TOOLS := cargo-deny cargo-machete cargo-nextest

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
	install -Dm755 target/release/$(BIN) $(BINDIR)/$(BIN)
ifeq ($(PREFIX),/usr)
	$(MAKE) install-completions
else ifeq ($(PREFIX),/usr/local)
	$(MAKE) install-completions
else
	@echo "Note: Shell completions not installed for user prefix."
	@echo "Use 'eval \"\$$($(BIN) completions bash)\"' in your shell rc file."
endif

install-completions:
	install -dm755 $(BASH_COMPLETION_DIR)
	install -dm755 $(ZSH_COMPLETION_DIR)
	install -dm755 $(FISH_COMPLETION_DIR)
	target/release/$(BIN) completions bash > $(BASH_COMPLETION_DIR)/$(BIN)
	target/release/$(BIN) completions zsh > $(ZSH_COMPLETION_DIR)/_$(BIN)
	target/release/$(BIN) completions fish > $(FISH_COMPLETION_DIR)/$(BIN).fish

# 7. Uninstall
uninstall:
	rm -f $(BINDIR)/$(BIN)
	rm -f $(BASH_COMPLETION_DIR)/$(BIN)
	rm -f $(ZSH_COMPLETION_DIR)/_$(BIN)
	rm -f $(FISH_COMPLETION_DIR)/$(BIN).fish

# 8. Setup
# One-time setup: configure hooks and check tools
setup: setup-hooks setup-tools

# Configure git hooks (idempotent)
setup-hooks:
	@current=$$(git config core.hooksPath 2>/dev/null); \
	 if [ "$$current" = ".githooks" ]; then \
	   echo "hooks: already configured"; \
	 else \
	   git config core.hooksPath .githooks; \
	   echo "hooks: configured .githooks"; \
	 fi

# Report cargo tool status (report-only; never fails the build)
setup-tools:
	@missing=0; \
	 for tool in $(CARGO_TOOLS); do \
	   if command -v $$tool >/dev/null 2>&1; then \
	     echo "  ok: $$tool"; \
	   else \
	     echo "  missing: $$tool"; \
	     missing=1; \
	   fi; \
	 done; \
	 if [ $$missing -eq 1 ]; then \
	   echo ""; \
	   echo "Install missing tools with:"; \
	   echo "  cargo binstall $(CARGO_TOOLS)"; \
	   echo ""; \
	   echo "Or with cargo install (slower, builds from source):"; \
	   echo "  cargo install $(CARGO_TOOLS)"; \
	 else \
	   echo "All tools present."; \
	 fi

clean:
	cargo clean
