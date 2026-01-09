# Configuration Design

## Philosophy: The "Contractor" Model

TheMan acts as a contractor executing a renovation. The user provides a **Work Order**
(Configuration) that specifies:

1.  **The Blueprint:** What style are we applying? (The Profile)
2.  **The Scope:** Which rooms are we renovating? (Enrollment)
3.  **Specific Instructions:** Any custom deviations? (Overrides)

## 1. The Blueprint (The Profile)

A Profile is a **Superset** of variables required by the supported applications.

```yaml
# profiles/nord.yaml
metadata:
  name: Nord

# The "Superset" of variables
vars:
  mode: "dark"
  wallpaper: "~/Pictures/nord-mountains.jpg"
  accent_color: "#88C0D0"

  # Colors
  bg: "#2E3440"
  fg: "#D8DEE9"
```

## 2. The Scope (Enrollment)

The user should not be burdened with configuring applications they don't use. The `theman.yaml`
config simply lists which "Integrations" are active.

```yaml
# theman.yaml (User Config)

# 1. Global State
current_profile: "nord"

# 2. Enrollment (The Scope)
# Only these apps will be touched.
enroll:
  - gtk
  - foot
  - dunst
  # - qt (Commented out, so TheMan ignores QT completely)
```

## 3. Overrides (Specific Instructions)

Users can override global variables from the Profile for specific apps.

```yaml
# theman.yaml

overrides:
  # Global Override (Apply to all apps)
  global:
    wallpaper: "~/Pictures/my-custom-wall.png"

  # App-Specific Override
  foot:
    # Use a specific font size for Foot, ignoring the profile defaults
    font_size: 14
```

## 4. Resolution Logic

When TheMan updates an application (e.g., `foot`):

1.  **Check Enrollment:** Is `foot` in the `enroll` list? If no, skip.
2.  **Load Handler:** Load the integration logic for `foot`.
3.  **Resolve Variables:**
    - Start with **Profile** variables.
    - Merge **Global Overrides**.
    - Merge **App-Specific Overrides**.
4.  **Execute:** Pass the resolved variable set to the Handler.

## 5. Benefits

- **Minimal Config:** A user only lists the apps they use.
- **Safety:** TheMan doesn't touch config files for unenrolled apps.
- **Flexibility:** Presets provide the basics, but users have full control to tweak specific apps
  without breaking the global theme.
