---
title: CLI Reference
description: Complete reference for all Themis commands and options
---

Complete reference for all Themis commands and options.

## Global Options

These options work with any command:

| Option                | Description                                            |
| --------------------- | ------------------------------------------------------ |
| `-c, --config <PATH>` | Path to config directory (default: `~/.config/themis`) |
| `-v, --verbose`       | Enable debug logging                                   |
| `--help`              | Show help for any command                              |
| `--version`           | Show version                                           |

## Commands

### `themis load <PROFILE>`

Load a profile and apply it to all enrolled applications.

```bash
themis load nord
themis load my-dark --dry-run
```

**Arguments:**

- `<PROFILE>` - Name of the profile (without `.yaml` extension)

**Options:**

| Option      | Description                                               |
| ----------- | --------------------------------------------------------- |
| `--dry-run` | Preview changes without writing files or running commands |

**Exit Codes:**

- `0` - All apps configured successfully
- `1` - One or more apps failed (partial success)

### `themis status`

Show the currently loaded profile.

```bash
themis status
```

**Output:**

```
Current profile: nord
Last loaded: 2024-01-15T10:30:00Z
```

If no profile has been loaded:

```
No state found. Run 'themis load <profile>' first.
```

### `themis init`

Create the initial configuration directory structure.

```bash
themis init
```

**Creates:**

```
~/.config/themis/
├── themis.yaml           # Main config with example enrollment
├── profiles/
│   └── example.yaml      # Sample profile
├── palettes/             # Empty directory for user palettes
└── templates/            # Empty directory for templates
```

Running `init` again is safe - it won't overwrite existing files.

### `themis verify`

Validate configuration files and references.

```bash
themis verify
```

**Checks:**

- YAML syntax in all config files
- Template file paths exist
- Palette references in profiles exist
- Profile syntax is valid

**Exit Codes:**

- `0` - All checks passed
- `1` - Errors found

### `themis doctor`

Check that enrolled applications have proper include patterns configured.

```bash
themis doctor
```

**Checks for each enrolled app:**

- App's main config file exists
- Config includes the Themis-generated partial

**Example Output:**

```
kitty: ✓ includes .themis.conf
waybar: ✗ missing include for style.css
```

**Exit Codes:**

- `0` - All apps properly configured
- `1` - One or more apps missing includes

### `themis completions <SHELL>`

Generate shell completions.

```bash
themis completions bash
themis completions zsh
themis completions fish
```

**Arguments:**

- `<SHELL>` - Shell type: `bash`, `zsh`, or `fish`

**Usage:**

```bash
# Bash
eval "$(themis completions bash)"

# Zsh
eval "$(themis completions zsh)"

# Fish
themis completions fish | source
```

## Environment Variables

These are honored identically on Linux and macOS.

| Variable            | Description                                                          |
| ------------------- | -------------------------------------------------------------------- |
| `THEMIS_CONFIG_DIR` | Override config directory (same as `-c`)                             |
| `XDG_CONFIG_HOME`   | Base for config dir (default: `~/.config`)                           |
| `XDG_STATE_HOME`    | Base for state dir (default: `~/.local/state`)                       |
| `XDG_DATA_DIRS`     | System palette search roots (default: `/usr/local/share:/usr/share`) |

See the [Configuration Reference](./config.md) for the full per-OS path layout, including the
Homebrew prefixes searched for system palettes on macOS.

## Examples

```bash
# Initialize and create first profile
themis init
vim ~/.config/themis/profiles/dark.yaml

# Preview changes
themis load dark --dry-run

# Apply profile
themis load dark

# Check status
themis status

# Validate configuration
themis verify

# Check app setup
themis doctor

# Use custom config directory
themis -c ~/my-themes load special
```
