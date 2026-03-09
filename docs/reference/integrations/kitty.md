---
title: Kitty
description: Terminal emulator integration using live config reload
---

## Mechanism

Kitty supports reloading configuration on the fly. It does not natively support "themes" in the
sense of swapping files without restarting, EXCEPT via:

1.  **Live Reload:** It monitors `kitty.conf` and included files for changes.
2.  **Socket:** `kitten @ set-colors` (requires `allow_remote_control`).

## 2. Themis's Approach

We use the **Include Pattern** combined with **Live Reload**. We generate a file containing _only_
the color/font variables, which `kitty.conf` includes. When Themis updates this file, Kitty detects
the file change and re-renders the terminal instantly. No signals needed.

## 3. User Setup

**One-time:** Add this line to `~/.config/kitty/kitty.conf`:

```ini
include .themis.conf
```

## 4. Equivalent Configuration

If this wasn't built-in, a user would define it in `themis.yaml` like this:

```yaml
enroll:
  kitty:
    type: template
    input: "~/.config/themis/templates/kitty.conf.j2"
    output: "~/.config/kitty/.themis.conf"
    # No reload command needed; Kitty watches the file.
```

## 5. Template Variables Used

- `bg`, `fg`
- `color0`..`color15`
- `font_family` (optional)
- `opacity` (optional)
