# Integration Design (The Workers)

## Overview

Integrations are the logic units that apply a theme to a specific application. In "TheMan" analogy, these are the **Workers** (Painter, Electrician, Carpenter).

## 1. Integration Structure

An integration is defined by a **Manifest**. This can be defined inline in `theman.yaml` or in separate files (e.g., `~/.config/theman/integrations/foot.yaml`).

### Schema

```yaml
name: foot           # Unique ID
description: "Foot Terminal Emulator"
depends_on: []       # Dependencies (e.g., needs 'fc-cache' to run first?)

# The Validation Interface
# (Optional) Declare what variables this integration expects.
requires:
  - bg
  - fg

# The Actions
# An integration is a sequence of one or more actions.
actions:
  - type: <TYPE>
    <PARAMS>
```

## 2. Action Types

### A. `template` (Preferred)

The most powerful method. TheMan renders a template file (e.g., `theme.j2`) into a target configuration file.

**Safety Recommendation (The "Include" Pattern):**

Do **not** overwrite the application's main configuration file (e.g., `kitty.conf`). Instead:

1.  Generate a separate **hidden file**: `~/.config/kitty/.theman.conf`.

2.  Manually add an include directive (e.g., `include .theman.conf`) to your main config once.

3.  Add `.theman.*` to your `.gitignore` to keep your dotfiles clean.



This ensures TheMan never accidentally deletes your keybindings or custom logic.



```yaml

actions:

  - type: template

    input: "~/.config/theman/templates/kitty.conf.j2"

    output: "~/.config/kitty/.theman.conf"

```

### B. `symlink`

Best used for applications that support **Include** directives (like Alacritty, Sway, Kitty). Instead of swapping the whole config, you only swap the color file.

- **Use Case:** `alacritty.toml` imports `theme.toml`. TheMan symlinks `theme.toml` to `presets/nord.toml`.

```yaml
actions:
  - type: symlink
    source: "~/.config/foot/themes/{{ preset.name }}.ini"
    target: "~/.config/foot/foot.ini"
    force: true
```

### C. `script` (External Executable)
Runs a single external script or binary. This is the "Escape Hatch" for complex logic.
*   **Context:** Takes the environment variables (`THEMAN_*`) and custom arguments.
*   **Concurrency:** Runs as a standalone process.

```yaml
actions:
  - type: script
    path: "~/bin/update_rgb_keyboard.py"
    # Optional: Map internal variables to command-line flags
    args: ["--mode", "{{ mode }}", "--color", "{{ bg }}"]
```

### D. `command` (Inline Shell)
Runs a list of shell commands sequentially.
*   **Context:** Variables are interpolated into the string before execution.
*   **Concurrency:** Commands in the list run one after another (blocking).

```yaml
actions:
  - type: command
    commands:
      - "gsettings set org.gnome.desktop.interface gtk-theme '{{ preset.gtk_theme }}'"
      - "pkill -USR1 waybar"
```

## 3. The "Standard Library" (Built-ins)
"TheMan" ships with embedded templates for popular tools.

**Lookup Priority:**
1.  **User Template:** Does `~/.config/theman/templates/<app>.j2` exist? Use it.
2.  **Configured Path:** Did the user specify a custom `input` path in `theman.yaml`? Use it.
3.  **Embedded:** Use the binary's internal template.

This allows users to seamlessly "eject" from the defaults by simply creating a file in their templates directory.

*   **Shells:** Alacritty, Foot, Kitty, WezTerm
*   **Desktop:** GTK (gsettings), Qt (Kvantum/qt5ct)
*   **Bars:** Waybar, Polybar
*   **Launchers:** Rofi, Wofi
*   **Editors:** Neovim (via env var or generated file), VSCode (settings.json)

## 4. Lifecycle

1.  **Pre-Flight:** Check if the target application is installed (optional `binary_check`).
2.  **Render:** Prepare all templates in memory.
3.  **Apply:** Write files / Update Symlinks.
4.  **Post-Flight:** Run reload commands (defined in `exec` actions).

