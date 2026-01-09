# CLI Reference

Complete reference for all TheMan commands and options.

## Global Options

These options work with any command:

| Option                | Description                                            |
| --------------------- | ------------------------------------------------------ |
| `-c, --config <PATH>` | Path to config directory (default: `~/.config/theman`) |
| `-v, --verbose`       | Enable debug logging                                   |
| `--help`              | Show help for any command                              |
| `--version`           | Show version                                           |

## Commands

### `theman load <PROFILE>`

Load a profile and apply it to all enrolled applications.

```bash
theman load nord
theman load my-dark --dry-run
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

### `theman status`

Show the currently loaded profile.

```bash
theman status
```

**Output:**

```
Current profile: nord
Last loaded: 2024-01-15T10:30:00Z
```

If no profile has been loaded:

```
No state found. Run 'theman load <profile>' first.
```

### `theman init`

Create the initial configuration directory structure.

```bash
theman init
```

**Creates:**

```
~/.config/theman/
├── theman.yaml           # Main config with example enrollment
├── profiles/
│   └── example.yaml      # Sample profile
├── palettes/             # Empty directory for user palettes
└── templates/            # Empty directory for templates
```

Running `init` again is safe - it won't overwrite existing files.

### `theman verify`

Validate configuration files and references.

```bash
theman verify
```

**Checks:**

- YAML syntax in all config files
- Template file paths exist
- Palette references in profiles exist
- Profile syntax is valid

**Exit Codes:**

- `0` - All checks passed
- `1` - Errors found

### `theman doctor`

Check that enrolled applications have proper include patterns configured.

```bash
theman doctor
```

**Checks for each enrolled app:**

- App's main config file exists
- Config includes the TheMan-generated partial

**Example Output:**

```
kitty: ✓ includes .theman.conf
waybar: ✗ missing include for style.css
```

**Exit Codes:**

- `0` - All apps properly configured
- `1` - One or more apps missing includes

### `theman completions <SHELL>`

Generate shell completions.

```bash
theman completions bash
theman completions zsh
theman completions fish
```

**Arguments:**

- `<SHELL>` - Shell type: `bash`, `zsh`, or `fish`

**Usage:**

```bash
# Bash
eval "$(theman completions bash)"

# Zsh
eval "$(theman completions zsh)"

# Fish
theman completions fish | source
```

## Environment Variables

| Variable            | Description                                    |
| ------------------- | ---------------------------------------------- |
| `THEMAN_CONFIG_DIR` | Override config directory (same as `-c`)       |
| `XDG_CONFIG_HOME`   | Base for config dir (default: `~/.config`)     |
| `XDG_STATE_HOME`    | Base for state dir (default: `~/.local/state`) |

## Examples

```bash
# Initialize and create first profile
theman init
vim ~/.config/theman/profiles/dark.yaml

# Preview changes
theman load dark --dry-run

# Apply profile
theman load dark

# Check status
theman status

# Validate configuration
theman verify

# Check app setup
theman doctor

# Use custom config directory
theman -c ~/my-themes load special
```
