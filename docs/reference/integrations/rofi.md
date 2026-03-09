---
title: Rofi
description: Application launcher theming with rasi files
---

## Mechanism

Rofi uses `.rasi` files (CSS-like syntax). It loads a main configuration file (`config.rasi`). This
file can import a theme using `@theme "name"`.

## 2. Themis's Approach

We use the **Template** pattern. We generate a `.rasi` file containing the colors and save it as
`~/.config/rofi/themis.rasi`.

## 3. User Setup

**One-time:** Edit `~/.config/rofi/config.rasi` to point to the generated theme.

```css
configuration {
  /* ... settings ... */
}
@theme "themis";
```

## 4. Equivalent Configuration

```yaml
enroll:
  rofi:
    type: template
    input: "~/.config/themis/templates/rofi.rasi.j2"
    output: "~/.config/rofi/themis.rasi"
    # Rofi reads config at startup; no reload needed unless the daemon is running (rare).
```
