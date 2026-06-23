#!/bin/sh
# Themis installer — downloads the prebuilt `themis` binary from the latest
# GitHub Release and installs it on your PATH. POSIX sh; no bashisms.
#
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/TwoWells/Themis/main/install.sh | sh
#
# Environment overrides:
#   THEMIS_VERSION       Version to install (e.g. 0.1.0). Default: latest release.
#   THEMIS_INSTALL_DIR   Install directory. Default: $HOME/.local/bin.
#
# Requires: curl, tar, and one of sha256sum or shasum.

set -eu

REPO="TwoWells/Themis"
BIN="themis"
INSTALL_DIR="${THEMIS_INSTALL_DIR:-$HOME/.local/bin}"

# --- helpers ---------------------------------------------------------------

# Print to stderr.
err() {
  echo "themis-install: $*" >&2
}

# Print a fatal error and exit non-zero.
fatal() {
  err "error: $*"
  exit 1
}

# Verify a command exists, or abort with a clear message.
need_cmd() {
  if ! command -v "$1" >/dev/null 2>&1; then
    fatal "required command not found: $1"
  fi
}

# --- preflight -------------------------------------------------------------

need_cmd uname
need_cmd curl
need_cmd tar
need_cmd mkdir
need_cmd chmod

# Pick a SHA-256 tool: sha256sum (Linux) or shasum (macOS/BSD).
SHA_CMD=""
if command -v sha256sum >/dev/null 2>&1; then
  SHA_CMD="sha256sum"
elif command -v shasum >/dev/null 2>&1; then
  SHA_CMD="shasum -a 256"
else
  fatal "need sha256sum or shasum to verify the download"
fi

# --- detect target triple --------------------------------------------------

os="$(uname -s)"
arch="$(uname -m)"

case "$os" in
  Linux) os_part="unknown-linux-gnu" ;;
  Darwin) os_part="apple-darwin" ;;
  *) fatal "unsupported operating system: $os (Themis ships Linux x86_64 today)" ;;
esac

case "$arch" in
  x86_64 | amd64) arch_part="x86_64" ;;
  aarch64 | arm64) arch_part="aarch64" ;;
  *) fatal "unsupported architecture: $arch" ;;
esac

TARGET="${arch_part}-${os_part}"

# Releases publish x86_64-unknown-linux-gnu and aarch64-apple-darwin. Fail
# clearly for anything else so the user gets a cargo-install pointer instead of
# a 404.
case "$TARGET" in
  x86_64-unknown-linux-gnu | aarch64-apple-darwin) : ;;
  *)
    err "no prebuilt binary for target: $TARGET"
    err "Themis ships x86_64-unknown-linux-gnu and aarch64-apple-darwin."
    err "Build from source instead: cargo install themis-cli"
    exit 1
    ;;
esac

# --- resolve version -------------------------------------------------------

VERSION="${THEMIS_VERSION:-}"
if [ -z "$VERSION" ]; then
  echo "themis-install: resolving latest release..."
  api_url="https://api.github.com/repos/${REPO}/releases/latest"
  # Pull the tag_name out of the JSON without a JSON parser. The field looks
  # like:  "tag_name": "v0.1.0",
  tag="$(
    curl -fsSL "$api_url" \
      | grep '"tag_name"' \
      | head -1 \
      | sed 's/.*"tag_name"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/'
  )"
  if [ -z "$tag" ]; then
    fatal "could not determine the latest release from $api_url"
  fi
  # Strip a leading 'v' so VERSION is the bare semver (tags are vX.Y.Z).
  VERSION="${tag#v}"
fi

echo "themis-install: installing themis v${VERSION} (${TARGET})"

# --- download + verify -----------------------------------------------------

ARCHIVE="${BIN}-${TARGET}.tar.gz"
BASE_URL="https://github.com/${REPO}/releases/download/v${VERSION}"
ARCHIVE_URL="${BASE_URL}/${ARCHIVE}"
SHA_URL="${ARCHIVE_URL}.sha256"

# Scratch dir, cleaned up on any exit.
tmp="$(mktemp -d 2>/dev/null || mktemp -d -t themis-install)"
cleanup() {
  rm -rf "$tmp"
}
trap cleanup EXIT INT TERM

echo "themis-install: downloading ${ARCHIVE_URL}"
if ! curl -fSL --proto '=https' --tlsv1.2 -o "${tmp}/${ARCHIVE}" "$ARCHIVE_URL"; then
  fatal "download failed: $ARCHIVE_URL"
fi

echo "themis-install: downloading checksum"
if ! curl -fSL --proto '=https' --tlsv1.2 -o "${tmp}/${ARCHIVE}.sha256" "$SHA_URL"; then
  fatal "checksum download failed: $SHA_URL"
fi

# The .sha256 file is in `sha256sum` format: "<hash>  <filename>". Take the
# first field as the expected hash; recompute locally and compare. We avoid
# `sha256sum -c` so the same logic works with shasum on macOS.
expected="$(awk '{ print $1; exit }' "${tmp}/${ARCHIVE}.sha256")"
if [ -z "$expected" ]; then
  fatal "could not read expected checksum from ${ARCHIVE}.sha256"
fi

actual="$(
  cd "$tmp" && $SHA_CMD "$ARCHIVE" | awk '{ print $1; exit }'
)"

if [ "$expected" != "$actual" ]; then
  err "checksum mismatch for $ARCHIVE"
  err "  expected: $expected"
  err "  actual:   $actual"
  fatal "refusing to install a binary that failed verification"
fi

echo "themis-install: checksum OK"

# --- extract + install -----------------------------------------------------

# The tarball contains `themis` at the archive root.
if ! tar -xzf "${tmp}/${ARCHIVE}" -C "$tmp"; then
  fatal "failed to extract $ARCHIVE"
fi

if [ ! -f "${tmp}/${BIN}" ]; then
  fatal "expected '${BIN}' at the archive root, but it was not found"
fi

mkdir -p "$INSTALL_DIR"
# Use cp + chmod rather than `install` (not guaranteed POSIX everywhere).
cp "${tmp}/${BIN}" "${INSTALL_DIR}/${BIN}"
chmod +x "${INSTALL_DIR}/${BIN}"

echo "themis-install: installed themis to ${INSTALL_DIR}/${BIN}"

# --- PATH check ------------------------------------------------------------

# Warn (don't fail) if the install dir is not on PATH. Pad PATH with ':' on
# both ends so a clean prefix/suffix/exact match all work in one test.
case ":${PATH}:" in
  *":${INSTALL_DIR}:"*) : ;;
  *)
    err "warning: ${INSTALL_DIR} is not on your PATH."
    err "add this to your shell profile (e.g. ~/.profile, ~/.bashrc, ~/.zshrc):"
    err "  export PATH=\"${INSTALL_DIR}:\$PATH\""
    ;;
esac

echo "themis-install: done. Run 'themis --help' to get started."
