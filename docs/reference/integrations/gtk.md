# GTK Integration

## 1. Mechanism

GTK apps (LibAdwaita, GTK3/4) do not read config files directly for theming. They rely on the
**DConf** database (GNOME Settings Daemon). Changes must be made via `gsettings` commands or IPC
calls. **Persistence:** Changes made via `gsettings` are written to disk (`~/.config/dconf/user`)
and persist across reboots.

## 2. TheMan's Approach

We use the **Exec** pattern. We run a series of `gsettings` commands to update the relevant keys in
the DConf database. These changes apply immediately to running applications.

## 3. User Setup

None. DConf is a system-level store (per user). No config file edits are required.

## 4. Equivalent Configuration

If this wasn't built-in, a user would define it like this:

```yaml
enroll:
  gtk:
    type: command # or 'script'
    commands:
      - "gsettings set org.gnome.desktop.interface gtk-theme '{{ preset.gtk_theme }}'"
      - "gsettings set org.gnome.desktop.interface icon-theme '{{ preset.icon_theme }}'"
      - "gsettings set org.gnome.desktop.interface cursor-theme '{{ preset.cursor_theme }}'"
      - "gsettings set org.gnome.desktop.interface font-name '{{ preset.font_name }}'"
      - "gsettings set org.gnome.desktop.interface color-scheme 'prefer-{{ mode }}'"
```

## 5. Required Profile Variables

- `gtk_theme` (e.g., "Adwaita")
- `icon_theme`
- `cursor_theme`
- `font_name` (e.g., "Cantarell 11")
- `mode` (light/dark)
