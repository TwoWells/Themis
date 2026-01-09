# TheMan

**TheMan** is a theme orchestrator CLI for Linux. It manages switching system themes across multiple
applications by coordinating profiles, palettes, and integrations.

TheMan acts as a "General Contractor" for desktop theming—it doesn't generate colors, but manages
the _who, what, and when_ of applying themes.

## Features

- **Profile-based theming** - Define profiles that include color palettes and app-specific settings
- **Palette inheritance** - System palettes (nord, dracula, etc.) can be extended by user palettes
- **Multiple integration types** - Templates, symlinks, commands, and scripts
- **Safety-first** - Generates hidden partials (`.theman.conf`) that users manually include
- **Dry-run mode** - Preview changes without modifying files
- **XDG compliant** - Respects `XDG_CONFIG_HOME` and `XDG_STATE_HOME`

## Quick Start

```bash
# Initialize configuration
theman init

# Load a profile
theman load my-profile

# Check current status
theman status

# Preview changes without applying
theman load my-profile --dry-run
```

## How It Works

1. **Enroll** applications in `theman.yaml` with their integration type
2. **Create profiles** that define variables (colors, fonts, settings)
3. **Run `theman load`** to apply the profile to all enrolled apps

TheMan never overwrites your main config files. Instead, it generates hidden partial files (like
`.theman.conf`) that you include in your app's configuration.

## Next Steps

- [Getting Started](./getting-started.md) - Installation and first profile
- [Guides](./guides/) - In-depth guides for profiles and integrations
- [Reference](./reference/) - CLI commands and configuration schema
