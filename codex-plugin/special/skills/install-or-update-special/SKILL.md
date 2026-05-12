---
name: install-or-update-special
description: 'Use this skill when the Special binary, Codex plugin, or marketplace setup is missing, stale, or not on PATH. Guide the user through normal install/update channels without silently installing binaries.'
---

# Install Or Update Special

Special has two install surfaces: the native `special` binary
and the Codex plugin that exposes MCP tools
plus skills. The plugin does not replace the binary.

## When To Use

Use this when the native binary, Codex plugin, marketplace entry, or plugin
version is missing or stale. Do not use this for repo setup after install is
working.

## Workflow

1. Check whether `special` is available with `special --version`.
2. If `special` is missing, tell the user the normal binary install command:

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

5. If the Codex plugin or marketplace entry is missing, use the SourceRodeo
   marketplace path:

   ```sh
   codex plugin marketplace add SourceRodeo/codex-marketplace
   ```

   Then install or enable the Special plugin through Codex's available plugin
   UI or CLI surface for that Codex version.

6. If the plugin is stale, upgrade the SourceRodeo marketplace and then update
   the installed Special plugin through Codex's available plugin surface:

   ```sh
   codex plugin marketplace upgrade sourcerodeo
   ```

7. After install or update, verify the binary and MCP server:

   ```sh
   special --version
   special mcp
   ```

   For `special mcp`, only use an MCP client or a short JSON-RPC smoke test; do
   not expect ordinary prose on stdout.

Do not silently install or update binaries or plugins. Ask before running
package-manager or Codex plugin commands, and prefer the user's existing
toolchain, package manager, and Codex install surface.
