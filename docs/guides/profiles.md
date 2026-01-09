# Profiles and Palettes

Profiles are the core of TheMan's theming system. A profile defines variables that get applied to
all your enrolled applications.

## Profile Structure

A profile is a YAML file in `~/.config/theman/profiles/`:

```yaml
# profiles/my-dark.yaml
vars:
  bg: "#2e3440"
  fg: "#eceff4"
  accent: "#88c0d0"
  font_family: "JetBrains Mono"
  transparency: 0.95
```

Variables can be any type: strings, numbers, booleans, or even arrays.

## Palettes

Palettes are reusable sets of color variables. They're stored in:

- User palettes: `~/.config/theman/palettes/`
- System palettes: `/usr/share/theman/palettes/`

A palette has the same structure as a profile:

```yaml
# palettes/nord.yaml
vars:
  bg: "#2e3440"
  fg: "#eceff4"
  color0: "#3b4252"
  color1: "#bf616a"
  # ... more colors
```

## Inheritance with `include`

Profiles can include palettes to inherit their variables:

```yaml
# profiles/my-nord.yaml
include: nord # Includes palettes/nord.yaml

vars:
  # Override or add variables
  font_family: "Fira Code"
  transparency: 0.9
```

The inheritance chain works like this:

1. Load the included palette's variables
2. Override with the profile's variables
3. Override with any config overrides

### Chained Inheritance

Palettes can include other palettes:

```yaml
# palettes/base.yaml
vars:
  font_family: "Monospace"
  font_size: 12

# palettes/nord.yaml
include: base
vars:
  bg: "#2e3440"
  fg: "#eceff4"

# profiles/my-theme.yaml
include: nord
vars:
  font_family: "JetBrains Mono"  # Overrides base
```

Result: `font_size: 12` (from base), `bg: #2e3440` (from nord), `font_family: JetBrains Mono` (from
profile)

## User vs System Palettes

When you `include: nord`, TheMan searches:

1. `~/.config/theman/palettes/nord.yaml` (user palette)
2. `/usr/share/theman/palettes/nord.yaml` (system palette)

User palettes take precedence, allowing you to customize system palettes.

## Config Overrides

You can override variables in `theman.yaml` without modifying profiles:

```yaml
# theman.yaml
enroll:
  kitty:
    type: template
    # ...

overrides:
  global:
    font_size: 14 # Applied to all apps

  kitty:
    font_size: 16 # Only for kitty
```

Override precedence (highest to lowest):

1. App-specific overrides (`overrides.kitty`)
2. Global overrides (`overrides.global`)
3. Profile variables
4. Included palette variables

## Circular Inheritance Detection

TheMan detects and prevents circular includes:

```yaml
# This will error: "Circular include detected"
# palettes/a.yaml
include: b

# palettes/b.yaml
include: a
```

## Best Practices

1. **Use palettes for colors** - Keep color schemes in palettes for reuse
2. **Use profiles for settings** - App-specific settings go in profiles
3. **Name profiles by purpose** - `work.yaml`, `gaming.yaml`, `presentation.yaml`
4. **Keep variables flat** - Nested objects can't be passed to templates easily
