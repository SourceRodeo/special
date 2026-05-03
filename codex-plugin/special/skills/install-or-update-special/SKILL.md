---
name: install-or-update-special
description: 'Use this skill when the Special Codex plugin is installed but the `special` binary is missing, too old, or not on PATH. Guide the user through normal install or update channels without silently installing binaries.'
---

# Install Or Update Special

This plugin provides Codex integration. It does not install the native
`special` binary by itself.

## Workflow

1. Check whether `special` is available with `special --version`.
2. If `special` is missing, tell the user the normal install command:

   ```sh
   brew install sourcerodeo/homebrew-tap/special
   ```

3. If the user wants a direct Rust install instead, use:

   ```sh
   cargo install special-cli
   ```

4. If `special` is present but stale, guide the user to update through the same
   channel they used to install it, usually:

   ```sh
   brew upgrade sourcerodeo/homebrew-tap/special
   ```

5. After install or update, verify:

   ```sh
   special --version
   special mcp
   ```

   For `special mcp`, only use an MCP client or a short JSON-RPC smoke test; do
   not expect ordinary prose on stdout.

Do not silently install or update binaries. Ask before running package-manager
commands, and prefer the user's existing toolchain and package manager.
