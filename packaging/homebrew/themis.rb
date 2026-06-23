# Homebrew formula for Themis — installs the prebuilt Release binary.
#
# This formula lives in the tap repo `TwoWells/homebrew-tap` (so users run
# `brew install twowells/tap/themis`). It is mirrored here, under the main
# repo's `packaging/homebrew/`, as the source of truth; copy it to the tap on
# each release.
#
# Per-release pinning (see packaging/homebrew/README.md):
#   1. Bump `version`.
#   2. Replace each REPLACE_WITH_*_SHA256 with the real checksum from the
#      published `themis-<target>.tar.gz.sha256` sidecar on the GitHub Release.
# The placeholders below are intentionally invalid so an unpinned release fails
# loudly at `brew install` rather than installing an unverified artifact.
class Themis < Formula
  desc "Theme orchestrator CLI for Linux and macOS"
  homepage "https://github.com/TwoWells/Themis"
  version "0.1.0"
  license "AGPL-3.0-or-later"

  # Only two Release targets exist: macOS arm64 and Linux x86_64. Each platform
  # gets its own url + sha256; unsupported platforms get a clear error instead
  # of a 404 from a guessed URL.
  on_macos do
    on_arm do
      url "https://github.com/TwoWells/Themis/releases/download/v0.1.0/themis-aarch64-apple-darwin.tar.gz"
      sha256 "623693a8ab3b89acaa91713782cac160834906c5d1d2a12aa38c77121ecd9558"
    end
    on_intel do
      odie "Themis has no prebuilt Intel macOS binary; build from source or use Apple Silicon."
    end
  end

  on_linux do
    on_intel do
      url "https://github.com/TwoWells/Themis/releases/download/v0.1.0/themis-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "3ce410c9ecb6381454054ecdb130b985c1f2423194f1ba621fafa4be92bae194"
    end
    on_arm do
      odie "Themis has no prebuilt ARM Linux binary; build from source or use x86_64."
    end
  end

  def install
    # The Release tarball ships only the `themis` binary at its root.
    bin.install "themis"

    # Generate + install shell completions from the binary itself.
    generate_completions_from_executable(bin/"themis", "completions", shells: [:bash, :zsh, :fish])
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/themis --version")
  end
end
