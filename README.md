# TheMan

**TheMan** is a lightweight, script-based theme orchestrator for Linux. It manages the switching of system themes (Light/Dark) across multiple applications by executing a directory of specialized scripts.

## Features

*   **Simple Architecture:** Just a directory of bash scripts. If you can script it, TheMan can manage it.
*   **Stateful:** Remembers your current theme across reboots.
*   **Parallel Execution:** Runs all theme scripts simultaneously for instant switching.
*   **Extensible:** Add a new script to `~/.local/share/theman/scripts/` and it just works.

## Installation

```bash
./install.sh
```

This will:
1.  Link `theman` executable to `~/.local/bin/`.
2.  Link scripts to `~/.local/share/theman/scripts/`.
3.  Create a default config at `~/.config/theman/config.env`.

## Usage

```bash
# Switch to Dark Mode
theman dark

# Switch to Light Mode
theman light

# Toggle between modes
theman toggle

# Check current status
theman status
```

## Configuration

Configuration is stored in `~/.config/theman/config.env`. This file is sourced by the main executable and all child scripts, making it the perfect place to define global variables like colors, font names, or paths.

Example `config.env`:
```bash
export GTK_THEME_DARK="Adwaita-dark"
export GTK_THEME_LIGHT="Adwaita"
export FONT_NAME="Noto Sans"
```

## Creating Scripts

Create an executable script in `~/.local/share/theman/scripts/`. It will receive the target mode (`light` or `dark`) as the first argument (`$1`).

**Example: `~/.local/share/theman/scripts/my-app`**

```bash
#!/bin/bash
MODE=$1 # "light" or "dark"

if [ "$MODE" == "dark" ]; then
    # Do dark mode stuff
    cp ~/.config/myapp/dark.conf ~/.config/myapp/config
else
    # Do light mode stuff
    cp ~/.config/myapp/light.conf ~/.config/myapp/config
fi

# Reload the app
pkill -HUP myapp
```

TheMan exports a helper function `_update_symlink` and variables like `$THEME_MODE` for convenience.

## License

MIT
# theman
