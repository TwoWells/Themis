---
title: Neovim
description: Editor theming via IPC socket commands
---

## The Challenge

Neovim instances are isolated processes. Changing a config file doesn't magically update running
instances unless they are configured to watch that file. Alternatively, one can use `nvr` (Neovim
Remote) to send commands to each instance.

## 2. Themis's Approach (Simple)

Generate a Lua file with the theme variables. User adds a filesystem watcher in their `init.lua` to
reload when this file changes.

## 3. Themis's Approach (Advanced / Script)

Use a custom script to loop through `nvr` servers.

**Script:** `scripts/update_nvim.sh`

```bash
#!/bin/bash
MODE=$THEMIS_MODE
# ... loop through nvr --serverlist and run :set background=...
```

**Configuration:**

```yaml
enroll:
  neovim_ipc:
    type: script
    path: "~/.config/themis/scripts/update_nvim.sh"
```
