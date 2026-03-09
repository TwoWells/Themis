---
title: Configuration Reference
description: Complete reference for Themis configuration files
---

Complete reference for Themis configuration files.

## File Locations

| File            | Location                                 |
| --------------- | ---------------------------------------- |
| Main config     | `~/.config/themis/themis.yaml`           |
| Profiles        | `~/.config/themis/profiles/<name>.yaml`  |
| User palettes   | `~/.config/themis/palettes/<name>.yaml`  |
| System palettes | `/usr/share/themis/palettes/<name>.yaml` |
| Templates       | `~/.config/themis/templates/<name>.j2`   |
| State           | `~/.local/state/themis/state.json`       |

## themis.yaml

The main configuration file defines enrolled applications and overrides.

```yaml
# Currently active profile (optional, informational)
current_profile: nord

# Enrolled applications
enroll:
  <app_name>:
    type: template | symlink | command | script
    # ... type-specific options

# Variable overrides (optional)
overrides:
  global:
    <var>: <value>
  <app_name>:
    <var>: <value>
```

### Template Integration

```yaml
enroll:
  kitty:
    type: template
    input: "~/.config/themis/templates/kitty.j2" # Required
    output: "~/.config/kitty/.themis.conf" # Required
    reload_cmd: "pkill -SIGUSR1 kitty" # Optional
    reload_signal: SIGUSR1 # Optional
```

### Symlink Integration

```yaml
enroll:
  alacritty:
    type: symlink
    source: "~/.config/themis/configs/alacritty-{{ mode }}.toml" # Required
    target: "~/.config/alacritty/colors.toml" # Required
    reload_cmd: "touch ~/.config/alacritty/alacritty.toml" # Optional
```

### Command Integration

```yaml
enroll:
  gtk:
    type: command
    commands: # Required, list of shell commands
      - "gsettings set org.gnome.desktop.interface gtk-theme '{{ gtk_theme }}'"
      - "gsettings set org.gnome.desktop.interface color-scheme '{{ color_scheme }}'"
```

### Script Integration

```yaml
enroll:
  custom:
    type: script
    path: "~/.config/themis/scripts/custom.sh" # Required
    args: ["--mode", "{{ mode }}"] # Optional
    env: # Optional
      CUSTOM_VAR: "value"
```

## Profile Schema

Profiles define variables for theming.

```yaml
# Optional metadata
metadata:
  name: "My Dark Theme"
  description: "A dark theme based on Nord"

# Include a palette (optional)
include: nord

# Variable definitions
vars:
  bg: "#2e3440"
  fg: "#eceff4"
  font_family: "JetBrains Mono"
  font_size: 12
  transparency: 0.95
```

### Variable Types

```yaml
vars:
  # Strings
  bg: "#2e3440"
  font_family: "JetBrains Mono"

  # Numbers
  font_size: 12
  transparency: 0.95

  # Booleans
  bold_text: true

  # Arrays (colon-delimited in script env vars)
  colors: ["#111", "#222", "#333"]
```

## Palette Schema

Palettes have the same structure as profiles:

```yaml
# palettes/nord.yaml
include: base # Optional, for palette inheritance

vars:
  bg: "#2e3440"
  fg: "#eceff4"
  color0: "#3b4252"
  # ...
```

## State File

The state file tracks the current profile (managed automatically):

```json
{
  "last_run": "2024-01-15T10:30:00Z",
  "success": true,
  "current": {
    "profile": "nord"
  }
}
```

## Path Expansion

All paths support tilde expansion:

- `~/.config/themis/...` expands to `/home/user/.config/themis/...`

## Variable Interpolation

Templates and some fields support Jinja2 variable interpolation:

```yaml
# In themis.yaml
source: "~/.config/themis/configs/alacritty-{{ mode }}.toml"

# With profile vars:
# mode: dark
# Expands to: ~/.config/themis/configs/alacritty-dark.toml
```

Supported in:

- Template `input`/`output` paths
- Symlink `source` path
- Command strings
- Script `args`
