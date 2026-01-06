# Profile Design (The Blueprint)

## Overview
A **Profile** is a named collection of variables that defines a system state. It is the "Source of Truth" for color palettes, font choices, and asset paths.

*   **Old Term:** Preset
*   **New Term:** Profile

## 1. File Structure
Profiles are stored as YAML (or TOML) files in the user's config directory:
`~/.config/theman/profiles/*.yaml`

## 2. Schema

```yaml
# profiles/nord.yaml

metadata:
  name: "Nord"
  description: "Standard Nord Dark Theme"

# Inheritance (Optional)
# If set, this profile starts with the variables from 'dark.yaml'
extends: "dark" 

# The core dictionary of variables
vars:
  # ...
```
## 3. Usage
```bash
theman load nord       # Loads profiles/nord.yaml
theman load dark       # Loads profiles/dark.yaml
theman load work-mode  # Loads profiles/work-mode.yaml
```
