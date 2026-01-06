# CLI Design

## Overview
The Command Line Interface (CLI) is the primary entry point for the user. It must be intuitive, fast, and adhere to standard Unix conventions.

## 1. Global Flags
*   `-v, --verbose`: Enable debug logging (INFO/DEBUG levels).
*   `-q, --quiet`: Suppress all output except errors.
*   `-c, --config <PATH>`: Path to a custom `theman.yaml` (default: `~/.config/theman/theman.yaml`).
*   `--dry-run`: Simulate actions (rendering templates, resolving vars) without writing to disk or executing commands.

## 2. Commands

### 2.1. `load` (Primary Command)
Applies a specific profile.

```bash
theman load <PROFILE_NAME>
```

*   **Arguments:**
    *   `<PROFILE_NAME>`: The name of the profile file (without extension) in `~/.config/theman/profiles/`.
*   **Options:**
    *   `--dry-run`: Simulate actions.
    *   `--only <APP>`: Apply *only* to a specific enrolled app.
    *   `--exclude <APP>`: Skip a specific enrolled app.

**Example:**
```bash
theman load nord
theman load dark
theman load work --dry-run
```

### 2.2. `list`
Lists available resources.

```bash
theman list [profiles|integrations]
```

*   **profiles**: Lists all valid YAML files in the profiles directory.
*   **integrations**: Lists all enrolled apps and their status.

### 2.3. `status`
Shows the current state.

```bash
theman status
```

*   **Output:**
    *   Current Profile: `nord`
    *   Last Updated: `2023-10-27 10:00:00`
    *   Enrolled Apps: `foot`, `gtk`

### 2.4. `init`
Scaffolds the configuration directory.

```bash
theman init
```

*   Creates `~/.config/theman/`
*   Creates `~/.config/theman/profiles/`
*   Writes a default `theman.yaml`
*   Writes a sample `profiles/default.yaml`

### 2.5. `verify`
Validates the configuration and presets.

```bash
theman verify
```

*   Checks for syntax errors in YAML files.
*   Checks for broken symlinks or missing templates.
*   Verifies that all enrolled apps have a valid integration definition.

## 3. Output Format
*   **Success:** Minimal output. `✓ Loaded preset 'nord'`
*   **Error:** Clear, actionable error messages.
    *   `Error: Preset 'foobar' not found in ~/.config/theman/presets/`
    *   `Error: Integration 'foot' requires variable 'bg' which is missing in preset 'nord'.`
