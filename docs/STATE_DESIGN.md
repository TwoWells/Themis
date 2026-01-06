# State Management Design

## Overview
"TheMan" must remember its previous actions. This state is used to:
1.  Report status to the user (or status bars).
2.  Enable intelligent toggling (e.g., "switch to light mode of the *current* theme").
3.  Detect drift (optional future feature).

## 1. State Location
Following XDG State Home standards:
`~/.local/state/theman/state.json`

## 2. State Schema
The state file is a JSON object updated *after* a successful `load` operation.

```json
{
  "last_run": "2023-10-27T14:30:00Z",
  "success": true,
  
  "current": {
    "preset": "nord",
    "mode": "dark",
    
    # Store the exact variable set used?
    # Pros: Debugging, "Redo" capability.
    # Cons: Large file size.
    # Decision: Store metadata only for now.
    "checksum": "a1b2c3d4..." 
  },
  
  "history": [
    # Optional: Keep a small history of previous themes?
    # Useful for a "theman undo" feature.
  ]
}
```

## 3. The "Toggle" Logic
A common user request is "Toggle Light/Dark".
TheMan does not have a hardcoded `toggle` command logic. Instead, it relies on the state.

**Scenario:** User runs `theman toggle` (or `theman load --toggle` TBD).

1.  Read `state.json`.
2.  Identify current preset: `nord`.
3.  Identify current mode: `dark`.
4.  Look for a corresponding "inverse" preset?
    *   *Option A (Naming Convention):* Look for `nord-light` if current is `nord-dark`.
    *   *Option B (Variable Toggle):* Load the *same* preset `nord`, but inject `mode: light` as an override.
    
**Decision:** **Option B** is more robust for the Declarative model.
If `theman load nord --mode light` is run, the preset `nord.yaml` is loaded, but the `mode` variable is forced to `light`. The preset's internal logic (if using Jinja2 in the preset itself? No, presets are static YAML) ...

**Refinement on Presets & Modes:**
Since Presets are static YAML, we cannot "compute" new values inside the YAML based on a flag.
*Solution:* A Preset can define "Variants" or we simply rely on separate files (`nord-light.yaml`, `nord-dark.yaml`).

**Revised Toggle Strategy:**
The `toggle` command is just a shortcut.
If `state.preset` == `nord-dark`, try loading `nord-light`.
If `state.preset` == `nord`, and it has no obvious suffix, `toggle` might fail or require configuration mapping.

*Alternative:* The `theman.yaml` config can define pairs:
```yaml
toggles:
  nord-dark: nord-light
  gruvbox-dark: gruvbox-light
```
This is simple and explicit.

## 4. Concurrency & Locking
Since TheMan is a "Run-Once" CLI, race conditions are rare but possible (e.g., two scripts triggering it simultaneously).
*   **Lockfile:** `~/.local/state/theman/lock`
*   If the lock exists and is fresh (< 10 seconds), fail or wait.
*   This prevents corrupted state if the user mashes the button.
