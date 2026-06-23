# Themis

[![CI](https://github.com/TwoWells/Themis/actions/workflows/ci.yml/badge.svg)](https://github.com/TwoWells/Themis/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/themis-cli.svg)](https://crates.io/crates/themis-cli)
[![docs](https://img.shields.io/badge/docs-website-blue)](https://twowells.github.io/Themis/)
[![license](https://img.shields.io/badge/license-AGPL--3.0--or--later-blue.svg)](LICENSE)

**Themis** is a theme orchestrator CLI for Linux and macOS. It switches your whole desktop between
themes (Light/Dark and beyond) by coordinating profiles, palettes, and per-application integrations
from a single command.

Themis acts as a "General Contractor" for theming—it doesn't generate colors, it manages the _who,
what, and when_ of applying them. You bring the palettes and templates; Themis renders them, links
them, and reloads the right apps, atomically and without clobbering your hand-written config.

```bash
themis load dark   # one command re-themes every enrolled app
```

## Features

- **Profile-based theming:** Define profiles that include color palettes and app-specific settings
- **Palette inheritance:** System palettes (nord, dracula, etc.) can be extended by user palettes
- **Multiple integration types:** Templates, symlinks, commands, and scripts
- **Safety-first:** Generates hidden partials (`.themis.conf`) that users manually include
- **Dry-run mode:** Preview changes without modifying files
- **Cross-platform:** Runs on Linux and macOS, honoring `XDG_CONFIG_HOME`/`XDG_STATE_HOME` on both

## Install

### Quick install (Linux & macOS)

Download the prebuilt `themis` binary and put it on your `PATH`:

```bash
curl -fsSL https://raw.githubusercontent.com/TwoWells/Themis/main/install.sh | sh
```

The installer detects your OS/arch, verifies the release checksum, and installs to `~/.local/bin`
(override with `THEMIS_INSTALL_DIR`). Ensure that directory is on your `PATH`.

### With Cargo

If you have a Rust toolchain, the crate is published as `themis-cli` (the binary stays `themis`):

```bash
# No-compile: fetches the prebuilt release binary
cargo binstall themis-cli

# From source
cargo install themis-cli
```

### Arch Linux (AUR)

Build from source with [`themis`](https://aur.archlinux.org/packages/themis), or grab the prebuilt
binary with [`themis-bin`](https://aur.archlinux.org/packages/themis-bin):

```bash
# build from source
yay -S themis

# prebuilt binary
yay -S themis-bin
```

### macOS

The quick installer and `cargo` paths above both work on macOS. A Homebrew tap is on the way:

```bash
# coming soon
brew install twowells/tap/themis
```

> Note: Themis themes whatever has config files. Its example enrollments (waybar, hyprland, …) are
> Linux desktop apps; on macOS you point it at the configs you actually run.

### From source

Requires [Rust](https://rustup.rs/) (see [`rust-toolchain.toml`](rust-toolchain.toml) for the pinned
version).

```bash
git clone https://github.com/TwoWells/Themis.git
cd Themis

# User install (no sudo, installs to ~/.local)
make install PREFIX=~/.local

# System install (requires sudo, installs to /usr/local)
sudo make install
```

This installs the binary and shell completions for bash, zsh, and fish. For a user install, ensure
`~/.local/bin` is on your `PATH`.

To uninstall:

```bash
# User uninstall
make uninstall PREFIX=~/.local

# System uninstall
sudo make uninstall
```

## Quick Start

```bash
# Initialize configuration
themis init

# Load a profile
themis load my-profile

# Check current status
themis status

# Verify configuration
themis verify

# Check app configurations
themis doctor
```

## Configuration

Configuration follows the XDG directory conventions on both Linux and macOS:

- Config: `~/.config/themis/themis.yaml`
- Profiles: `~/.config/themis/profiles/<name>.yaml`
- Palettes: `~/.config/themis/palettes/<name>.yaml`
- Templates: `~/.config/themis/templates/<app>.j2`
- State: `~/.local/state/themis/state.json`

System palettes are installed to `/usr/share/themis/palettes/`. `XDG_CONFIG_HOME` and
`XDG_STATE_HOME` are honored on both platforms; see the
[documentation](https://twowells.github.io/Themis/) for the exact per-OS defaults.

### themis.yaml

The main configuration file enrolls applications:

```yaml
enroll:
  kitty:
    type: template
    input: "~/.config/themis/templates/kitty.j2"
    output: "~/.config/kitty/.themis.conf"
    reload_signal: SIGUSR1

  waybar:
    type: template
    input: "~/.config/themis/templates/waybar.j2"
    output: "~/.config/waybar/colors.css"
    reload_cmd: "pkill -SIGUSR2 waybar"

  gtk:
    type: command
    commands:
      - "gsettings set org.gnome.desktop.interface color-scheme '{{ color_scheme }}'"
```

### Profiles

Profiles define variables and can include palettes:

```yaml
# profiles/my-dark.yaml
include: nord # Include the nord palette

vars:
  color_scheme: prefer-dark
  transparency: 0.95
```

### Palettes

Palettes define color variables:

```yaml
# palettes/nord.yaml (or system: /usr/share/themis/palettes/nord.yaml)
vars:
  bg: "#2e3440"
  fg: "#eceff4"
  accent: "#88c0d0"
```

Palettes can inherit from other palettes using `include`.

## Integration Types

### Template

Renders Jinja2 templates with profile variables:

```yaml
kitty:
  type: template
  input: "~/.config/themis/templates/kitty.j2"
  output: "~/.config/kitty/.themis.conf"
  reload_cmd: "kill -SIGUSR1 $(pgrep kitty)" # optional
  reload_signal: SIGUSR1 # optional (uses pkill)
```

### Symlink

Creates symlinks with variable interpolation in the source path:

```yaml
alacritty:
  type: symlink
  source: "~/.config/themis/configs/alacritty-{{ mode }}.toml"
  target: "~/.config/alacritty/colors.toml"
```

### Command

Executes shell commands with variable interpolation:

```yaml
gtk:
  type: command
  commands:
    - "gsettings set org.gnome.desktop.interface gtk-theme '{{ gtk_theme }}'"
    - "gsettings set org.gnome.desktop.interface color-scheme '{{ color_scheme }}'"
```

### Script

Executes external scripts with environment variables:

```yaml
custom:
  type: script
  path: "~/.config/themis/scripts/custom.sh"
  args: ["--mode", "{{ mode }}"]
  env:
    CUSTOM_VAR: "value"
```

All profile variables are passed as `THEMIS_<VAR>` environment variables.

## Commands

| Command                    | Description                                          |
| -------------------------- | ---------------------------------------------------- |
| `load <PROFILE>`           | Load a profile and apply to all enrolled apps        |
| `load <PROFILE> --dry-run` | Preview changes without writing files                |
| `status`                   | Show currently loaded profile                        |
| `init`                     | Create initial configuration structure               |
| `verify`                   | Validate configuration and profiles                  |
| `doctor`                   | Check app configurations for proper include patterns |
| `completions <SHELL>`      | Generate shell completions (bash, zsh, fish)         |

## Shell Completions

If you used `sudo make install`, completions are already installed system-wide.

For manual setup (or if you used `PREFIX=~/.local`):

```bash
# Bash (add to ~/.bashrc)
eval "$(themis completions bash)"

# Zsh (add to ~/.zshrc)
eval "$(themis completions zsh)"

# Fish (add to ~/.config/fish/config.fish)
themis completions fish | source
```

## App Setup

After enrolling an app, you need to include the generated config in your app's main configuration.
Run `themis doctor` to see what changes are needed.

Example for kitty (`~/.config/kitty/kitty.conf`):

```
include .themis.conf
```

## Documentation

Full docs—getting started, profile and integration guides, and the CLI/configuration reference—live
at [twowells.github.io/Themis](https://twowells.github.io/Themis/).

## License

AGPL-3.0-or-later. See [LICENSE](LICENSE) for details.
