.PHONY: fmt lint test test-doc deny machete mutants check build install install-completions uninstall clean setup setup-hooks setup-tools pre-release-check bump-version next-patch next-minor next-major release release-patch release-minor release-major publish tag-current version

# Binary name (matches the package name in Cargo.toml)
BIN := themis

# Current version, parsed from Cargo.toml
CURRENT_VERSION := $(shell grep '^version = ' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')

# Required cargo tools
CARGO_TOOLS := cargo-deny cargo-machete cargo-nextest cargo-mutants

# Hard per-process address-space cap (KiB) applied to test runs, so a runaway
# allocation aborts that single process at the limit instead of exhausting
# system RAM. Override with MEMLIMIT_KB=<kib>, or MEMLIMIT_KB=unlimited to
# disable.
MEMLIMIT_KB ?= 8388608

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
	cargo clippy --tests -- -D warnings
	prettier --check .

# 3. Test (via cargo-nextest)
# Pass T= to filter, N= to repeat (--stress-count), I=1 to include ignored.
# Note: nextest does not run doctests; the `test-doc` target covers those.
test:
	@if [ "$(MEMLIMIT_KB)" != unlimited ]; then ulimit -v $(MEMLIMIT_KB); fi; \
	 cargo nextest run $(if $(I),--run-ignored all,) $(if $(N),--stress-count $(N),) $(if $(T),-E 'test($(T))',)

# Doctests are not run by nextest; run them separately so `check` keeps
# doctest coverage.
test-doc:
	cargo test --doc

# 4. Dependency license + advisory gating (cargo-deny)
# Retry on exit 139 (cargo-deny#855 segfault); give up after 5 tries.
deny:
	@tries=0; while true; do \
	   cargo deny --log-level error check; rc=$$?; \
	   if [ $$rc -eq 0 ]; then break; \
	   elif [ $$rc -ne 139 ]; then exit $$rc; \
	   else \
	     tries=$$((tries + 1)); \
	     if [ $$tries -ge 5 ]; then echo "cargo-deny segfaulted 5 times, giving up"; exit 139; fi; \
	     echo "cargo-deny segfaulted (EmbarkStudios/cargo-deny#855), retry $$tries/5..."; \
	   fi; \
	 done

# 5. Unused-dependency check (cargo-machete)
machete:
	cargo machete --skip-target-dir

# Mutation testing (cargo-mutants). Slow (minutes); run on demand, not in
# `check`. Bare `cargo mutants` uses the default `cargo test` harness, so it also
# exercises the doctests (nextest would skip them).
mutants:
	cargo mutants --timeout 60

# 6. Meta-task for CI/Pre-commit
# nextest (test) runs unit/integration tests; test-doc preserves doctest
# coverage that nextest does not run.
check: lint deny machete test test-doc

# 7. Build
build:
	cargo build --release

# 8. Install (use sudo for system install, or PREFIX=~/.local for user install)
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

# 9. Uninstall
uninstall:
	rm -f $(BINDIR)/$(BIN)
	rm -f $(BASH_COMPLETION_DIR)/$(BIN)
	rm -f $(ZSH_COMPLETION_DIR)/_$(BIN)
	rm -f $(FISH_COMPLETION_DIR)/$(BIN).fish

# 10. Setup
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

# --- Release ---
# Note: Cargo.lock is gitignored in Themis, so the recipes below stage and roll
# back only Cargo.toml (touching the untracked lockfile with git would error).

# Abort unless the working tree is clean, on main, and in sync with origin/main.
pre-release-check:
	@echo "Checking release prerequisites..."
	@if [ -n "$$(git status --porcelain)" ]; then \
		echo "Error: Working tree is not clean. Commit or stash changes first."; \
		exit 1; \
	fi
	@if [ "$$(git branch --show-current)" != "main" ]; then \
		echo "Error: Not on main branch."; \
		exit 1; \
	fi
	@git fetch origin main --quiet
	@if [ "$$(git rev-parse HEAD)" != "$$(git rev-parse origin/main)" ]; then \
		echo "Error: Local main is not up to date with origin/main."; \
		exit 1; \
	fi
	@echo "Prerequisites OK."

# Bump the version in Cargo.toml (V=x.y.z) and refresh the lockfile via cargo check.
bump-version:
	@if [ -z "$(V)" ]; then \
		echo "Error: Version not specified. Use V=x.y.z"; \
		exit 1; \
	fi
	@echo "Bumping version: $(CURRENT_VERSION) -> $(V)"
	@sed -i 's/^version = "$(CURRENT_VERSION)"/version = "$(V)"/' Cargo.toml
	@cargo check --quiet
	@echo "Version bumped to $(V)"

next-patch:
	$(eval V := $(shell echo $(CURRENT_VERSION) | awk -F. '{print $$1"."$$2"."$$3+1}'))

next-minor:
	$(eval V := $(shell echo $(CURRENT_VERSION) | awk -F. '{print $$1"."$$2+1".0"}'))

next-major:
	$(eval V := $(shell echo $(CURRENT_VERSION) | awk -F. '{print $$1+1".0.0"}'))

# Cut a release locally: pre-flight checks -> bump -> check (rollback on fail) ->
# commit -> annotated tag. Cargo.lock is gitignored, so only Cargo.toml is
# rolled back / staged.
release: pre-release-check
	@if [ -z "$(V)" ]; then \
		echo "Error: Version not specified. Use 'make release V=x.y.z' or 'make release-patch'"; \
		exit 1; \
	fi
	@$(MAKE) bump-version V=$(V)
	@if ! $(MAKE) check; then \
		echo "Checks failed. Rolling back version bump..."; \
		git checkout HEAD -- Cargo.toml; \
		exit 1; \
	fi
	@git add Cargo.toml
	@if ! git commit -m "chore: Bump version to $(V)"; then \
		echo "Commit failed. Rolling back version bump..."; \
		git checkout HEAD -- Cargo.toml; \
		exit 1; \
	fi
	@git tag -a "v$(V)" -m "Release v$(V)"
	@echo ""
	@echo "Release v$(V) prepared locally."
	@echo "Run 'make publish' to push and create the release."

release-patch: pre-release-check next-patch
	@$(MAKE) release V=$(V)

release-minor: pre-release-check next-minor
	@$(MAKE) release V=$(V)

release-major: pre-release-check next-major
	@$(MAKE) release V=$(V)

publish:
	@echo "Pushing to origin..."
	@git push && git push --tags
	@echo ""
	@echo "Release v$(CURRENT_VERSION) pushed."

tag-current:
	@if git rev-parse "v$(CURRENT_VERSION)" >/dev/null 2>&1; then \
		echo "Tag v$(CURRENT_VERSION) already exists."; \
		exit 1; \
	fi
	@echo "Creating tag v$(CURRENT_VERSION) for current version..."
	@git tag -a "v$(CURRENT_VERSION)" -m "Release v$(CURRENT_VERSION)"
	@echo "Tag created. Run 'make publish' to push and release."

version:
	@echo "Current version: $(CURRENT_VERSION)"
	@echo "Latest tag:      $$(git describe --tags --abbrev=0 2>/dev/null || echo 'none')"
