# Homebrew tap for Themis

This directory holds the Homebrew formula ([`themis.rb`](themis.rb)) for installing the prebuilt
Themis Release binary on macOS (Apple Silicon) and Linuxbrew (x86_64). The formula is the source of
truth; the published tap is a copy of it.

## Supported platforms

The Release builds exactly two prebuilt targets, so the formula installs on exactly two platforms:

- macOS Apple Silicon — `aarch64-apple-darwin`
- Linux x86_64 — `x86_64-unknown-linux-gnu` (Linuxbrew)

Intel macOS and ARM Linux have no prebuilt binary; the formula errors out clearly on those rather
than guessing a download URL. Those users should build from source or use another channel
(crates.io, the AUR, or the installer script).

## Installing (for users)

The formula is published through a tap repo, so a separate `brew tap` step is not required:

```sh
brew install twowells/themis/themis
```

That `twowells/themis/themis` triple expands to "the `themis` formula in the
`TwoWells/homebrew-themis` tap repo". Equivalently:

```sh
brew tap twowells/themis
brew install themis
```

Shell completions for bash, zsh, and fish are generated from the binary at install time and placed
where Homebrew expects them, so they activate automatically once Homebrew's completion directories
are on your shell path.

## Creating the tap repo (one-time, at go-public)

Homebrew fetches Release assets over unauthenticated URLs, so the tap and the main repo must both be
public before `brew install` works.

1. Create a public repo named **`homebrew-themis`** under the `TwoWells` org. The `homebrew-` prefix
   is what lets users write `twowells/themis` instead of the full repo name.
2. Copy [`themis.rb`](themis.rb) to the tap repo as `Formula/themis.rb` (the `Formula/` subdirectory
   is the convention Homebrew auto-discovers).
3. Commit and push. The tap is now installable.

Keep the formula in this directory and the one in the tap in sync — treat this copy as canonical and
copy it over on each release.

## Per-release SHA256 pinning (every release)

The formula ships with placeholder version and checksums, because the Release artifacts do not exist
until a tag is cut. On each release, update the formula (here and in the tap):

1. Bump `version` in `themis.rb` to the new `X.Y.Z`. The per-platform `url`s embed the version, so
   they update with it.
2. Replace the two placeholder checksums with the real values:
   - `REPLACE_WITH_AARCH64_APPLE_DARWIN_SHA256` — the SHA256 of `themis-aarch64-apple-darwin.tar.gz`
   - `REPLACE_WITH_X86_64_UNKNOWN_LINUX_GNU_SHA256` — the SHA256 of
     `themis-x86_64-unknown-linux-gnu.tar.gz`

The release workflow publishes a `themis-<target>.tar.gz.sha256` sidecar next to each tarball, so
the checksums can be read straight off the Release page. For example:

```sh
v=0.1.0
for t in aarch64-apple-darwin x86_64-unknown-linux-gnu; do
  curl -fsSL \
    "https://github.com/TwoWells/Themis/releases/download/v${v}/themis-${t}.tar.gz.sha256"
done
```

This mirrors the per-release pinning the AUR `themis-bin` package documents (see
`packaging/aur/themis-bin/PKGBUILD`): the binary artifacts are versioned and their checksums are
pinned per release.

## Follow-ups

- Automating the formula version/URL/sha bump from `release.yml` (so a new tag pushes the update to
  the tap automatically) is a planned improvement; until then the bump is the manual step above.
