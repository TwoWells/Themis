# Integration Types

TheMan supports four integration types for theming applications. Each type is suited for different
scenarios.

## Template

Renders Jinja2 templates with profile variables. This is the **recommended** approach for most
applications.

```yaml
enroll:
  kitty:
    type: template
    input: "~/.config/theman/templates/kitty.j2"
    output: "~/.config/kitty/.theman.conf"
    reload_cmd: "kill -SIGUSR1 $(pgrep kitty)" # optional
    reload_signal: SIGUSR1 # optional (uses pkill)
```

### Template Syntax

Templates use Jinja2 syntax:

```jinja2
# ~/.config/theman/templates/kitty.j2
foreground {{ fg }}
background {{ bg }}
font_family {{ font_family }}
font_size {{ font_size }}

{% if transparency is defined %}
background_opacity {{ transparency }}
{% endif %}
```

### Special Variables

Templates receive these additional variables:

- `profile_name` - Name of the loaded profile
- `app_name` - Name of the current app (e.g., "kitty")

### Reload Options

- `reload_cmd` - Shell command to reload the app
- `reload_signal` - Signal name (e.g., `SIGUSR1`, `USR2`) sent via `pkill -<signal> <app_name>`

Most apps with live reload only need one of these. Kitty, for example, watches its config files
automatically and needs neither.

## Symlink

Creates symlinks with variable interpolation in the source path. Useful for apps that need entire
config files swapped.

```yaml
enroll:
  alacritty:
    type: symlink
    source: "~/.config/theman/configs/alacritty-{{ mode }}.toml"
    target: "~/.config/alacritty/colors.toml"
    reload_cmd: "touch ~/.config/alacritty/alacritty.toml"
```

With a profile containing `mode: dark`, this creates:

```
~/.config/alacritty/colors.toml -> ~/.config/theman/configs/alacritty-dark.toml
```

## Command

Executes shell commands with variable interpolation. Ideal for apps configured via CLI tools.

```yaml
enroll:
  gtk:
    type: command
    commands:
      - "gsettings set org.gnome.desktop.interface gtk-theme '{{ gtk_theme }}'"
      - "gsettings set org.gnome.desktop.interface color-scheme '{{ color_scheme }}'"
```

Commands are executed sequentially. If one fails, subsequent commands still run (with a warning
logged).

### Common Use Cases

- GTK/GNOME settings via `gsettings`
- Plasma settings via `kwriteconfig5`
- Wallpaper changes via `feh`, `swaybg`, etc.

## Script

Executes external scripts with environment variables. Best for complex logic that doesn't fit in
commands.

```yaml
enroll:
  custom:
    type: script
    path: "~/.config/theman/scripts/custom.sh"
    args: ["--mode", "{{ mode }}"]
    env:
      CUSTOM_VAR: "value"
```

### Environment Variables

All profile variables are passed as `THEMAN_<VAR>` environment variables:

```bash
#!/bin/bash
# All variables available as THEMAN_* env vars
echo "Background: $THEMAN_BG"
echo "Foreground: $THEMAN_FG"
echo "Mode: $THEMAN_MODE"
```

Array values are colon-delimited (Unix convention):

```yaml
# Profile
vars:
  colors: ["#111", "#222", "#333"]
```

```bash
# In script
echo $THEMAN_COLORS  # "#111:#222:#333"
```

## Choosing the Right Type

| Scenario                       | Recommended Type |
| ------------------------------ | ---------------- |
| App supports config includes   | Template         |
| App watches config files       | Template         |
| App needs entire file replaced | Symlink          |
| App configured via CLI tools   | Command          |
| Complex conditional logic      | Script           |
| Need to call other programs    | Script           |

## Integration Order

Apps are processed in the order they appear in `theman.yaml`. Use this to ensure dependencies are
set up first:

```yaml
enroll:
  # Set GTK theme first
  gtk:
    type: command
    commands: [...]

  # Then apps that might read GTK settings
  firefox:
    type: script
    path: [...]
```

## Error Handling

When an integration fails:

1. The error is logged
2. Processing continues to the next app
3. A summary is shown at the end
4. Exit code is 1 if any apps failed

Use `--dry-run` to preview all changes before applying:

```bash
theman load my-profile --dry-run
```
