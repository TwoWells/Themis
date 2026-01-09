# Waybar Integration

## 1. Mechanism

Waybar is styled via `style.css`. It supports live reloading if the config changes, OR via
`SIGUSR2`. However, often we only want to change specific colors, not the whole CSS layout.

## 2. TheMan's Approach

We use **Template + Include + Signal**.

1.  Generate `~/.config/waybar/colors.css`.
2.  Import this in `style.css`.
3.  Send `SIGUSR2` to reload the bar without restarting the process.

## 3. User Setup

**One-time:** Add import to `~/.config/waybar/style.css`:

```css
@import "colors.css";

/* Use the variables defined in colors.css */
window#waybar {
  background-color: @base;
  color: @text;
}
```

## 4. Equivalent Configuration

```yaml
enroll:
  waybar:
    type: template
    input: "~/.config/theman/templates/waybar_colors.css.j2"
    output: "~/.config/waybar/colors.css"
    # The crucial reload step
    reload_signal: SIGUSR2
    # OR manual command:
    # reload_cmd: "pkill -SIGUSR2 waybar"
```
