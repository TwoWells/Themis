# Polybar Integration (X11)

## 1. Mechanism

Polybar runs as a daemon. It usually has a config file (`config.ini`) which can include other files.
It supports an IPC mechanism (`polybar-msg`) to restart or reload the bar.

## 2. TheMan's Approach

We use **Template + Include + Command**.

1.  Generate `~/.config/polybar/colors.ini`.
2.  Include it in `config.ini`: `include-file = ~/.config/polybar/colors.ini`.
3.  Send a restart command.

## 3. User Setup

**One-time:** Add include to `~/.config/polybar/config.ini`:

```ini
[global/wm]
include-file = ~/.config/polybar/colors.ini
```

## 4. Equivalent Configuration

```yaml
enroll:
  polybar:
    type: template
    input: "~/.config/theman/templates/polybar_colors.ini.j2"
    output: "~/.config/polybar/colors.ini"
    # Command to reload/restart
    reload_cmd: "polybar-msg cmd restart"
```
