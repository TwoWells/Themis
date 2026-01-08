# TheMan

**TheMan** is a theme orchestrator CLI for Linux. It manages switching system themes across multiple
applications by coordinating profiles, palettes, and integrations.

TheMan acts as a "General Contractor" for desktop theming—it doesn't generate colors, but manages
the _who, what, and when_ of applying themes.

## Features

- **Profile-based theming:** Define profiles that include color palettes and app-specific settings
- **Palette inheritance:** System palettes (nord, dracula, etc.) can be extended by user palettes
- **Multiple integration types:** Templates, symlinks, commands, and scripts
- **Safety-first:** Generates hidden partials (`.theman.conf`) that users manually include
- **Dry-run mode:** Preview changes without modifying files
- **XDG compliant:** Respects `XDG_CONFIG_HOME` and `XDG_STATE_HOME`

## Installation

```bash
cargo build --release
cp target/release/theman ~/.local/bin/
```

## Quick Start

```bash
# Initialize configuration
theman init

# Load a profile
theman load my-profile

# Check current status
theman status

# Verify configuration
theman verify

# Check app configurations
theman doctor
```

## Configuration

Configuration follows XDG directories:

- Config: `~/.config/theman/theman.yaml`
- Profiles: `~/.config/theman/profiles/<name>.yaml`
- Palettes: `~/.config/theman/palettes/<name>.yaml`
- Templates: `~/.config/theman/templates/<app>.j2`
- State: `~/.local/state/theman/state.json`

System palettes are installed to `/usr/share/theman/palettes/`.

### theman.yaml

The main configuration file enrolls applications:

```yaml
enroll:
  kitty:
    type: template
    input: "~/.config/theman/templates/kitty.j2"
    output: "~/.config/kitty/.theman.conf"
    reload_signal: SIGUSR1

  waybar:
    type: template
    input: "~/.config/theman/templates/waybar.j2"
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
# palettes/nord.yaml (or system: /usr/share/theman/palettes/nord.yaml)
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
  input: "~/.config/theman/templates/kitty.j2"
  output: "~/.config/kitty/.theman.conf"
  reload_cmd: "kill -SIGUSR1 $(pgrep kitty)" # optional
  reload_signal: SIGUSR1 # optional (uses pkill)
```

### Symlink

Creates symlinks with variable interpolation in the source path:

```yaml
alacritty:
  type: symlink
  source: "~/.config/theman/configs/alacritty-{{ mode }}.toml"
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
  path: "~/.config/theman/scripts/custom.sh"
  args: ["--mode", "{{ mode }}"]
  env:
    CUSTOM_VAR: "value"
```

All profile variables are passed as `THEMAN_<VAR>` environment variables.

## Commands

| Command                    | Description                                          |
| -------------------------- | ---------------------------------------------------- |
| `load <PROFILE>`           | Load a profile and apply to all enrolled apps        |
| `load <PROFILE> --dry-run` | Preview changes without writing files                |
| `status`                   | Show currently loaded profile                        |
| `init`                     | Create initial configuration structure               |
| `verify`                   | Validate configuration and profiles                  |
| `doctor`                   | Check app configurations for proper include patterns |

## App Setup

After enrolling an app, you need to include the generated config in your app's main configuration.
Run `theman doctor` to see what changes are needed.

Example for kitty (`~/.config/kitty/kitty.conf`):

```
include .theman.conf
```

## License

MIT
