# HyprPolkitAgent (Systemd Service Example)

## 1. The Challenge
Some background services (like polkit agents) read their theme from environment variables when they start. To change the theme, you must:
1.  Update the environment variable definition.
2.  Import that variable into the `systemd --user` session.
3.  Restart the service.

## 2. TheMan's Approach
This requires a **Script Integration** because it involves multiple sequential steps involving system state.

## 3. Configuration Example

**Script:** `~/.config/theman/scripts/update_polkit.sh`
```bash
#!/bin/bash

# 1. Map TheMan vars to Polkit vars
export QT_QUICK_CONTROLS_MATERIAL_THEME="$1" # Passed as arg
export QT_QUICK_CONTROLS_STYLE="Material"

# 2. Import to Systemd
systemctl --user import-environment QT_QUICK_CONTROLS_MATERIAL_THEME QT_QUICK_CONTROLS_STYLE

# 3. Restart Service
systemctl --user restart hyprpolkitagent
```

**TheMan Config:**
```yaml
enroll:
  polkit:
    type: script
    path: "~/.config/theman/scripts/update_polkit.sh"
    # Pass the theme name (e.g., "Dark" or "Light")
    # Note: You might need a custom variable 'polkit_theme_name' in your profile
    # if it differs from the standard preset name.
    args: ["{{ polkit_theme_name | default(value='Dark') }}"]
```
