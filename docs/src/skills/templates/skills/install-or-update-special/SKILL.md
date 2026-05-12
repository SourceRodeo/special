---
name: install-or-update-special
description: 'Use this skill when the Special binary, Codex plugin, or marketplace setup is missing, stale, or not on PATH. Guide the user through normal install/update channels without silently installing binaries.'
---
@filedocuments spec SPECIAL.DISTRIBUTION.HOMEBREW.INSTALLS_SPECIAL

# Install Or Update Special
@implements SPECIAL.DOCUMENTATION.SKILLS.FALLBACK.INSTALL_OR_UPDATE_SPECIAL
@applies DOCS.SKILL_MAIN_ENTRY

Special has two install surfaces: the native [`special` binary](documents://spec/SPECIAL.DISTRIBUTION.HOMEBREW.INSTALLS_SPECIAL)
and the Codex plugin that exposes [MCP tools](documents://spec/SPECIAL.MCP_COMMAND.TOOLS)
plus skills. The plugin does not replace the binary.

## When To Use
@applies DOCS.SKILL_TRIGGER_BOUNDARY_SECTION

Use this when the native binary, Codex plugin, marketplace entry, or plugin
version is missing or stale. Do not use this for repo setup after install is
working.

## Workflow
@applies DOCS.SKILL_WORKFLOW_SECTION

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

7. After install or update, verify the binary and [MCP server](documents://spec/SPECIAL.MCP_COMMAND):

   ```sh
   special --version
   special mcp
   ```

   For [`special mcp`](documents://spec/SPECIAL.MCP_COMMAND), only use an MCP client or a short JSON-RPC smoke test; do
   not expect ordinary prose on stdout.

## What To Do With Results
@applies DOCS.SKILL_RESULT_DISPOSITION_SECTION

- If the binary is missing or stale, fix the binary install first.
- If the plugin is missing or stale, fix marketplace/plugin installation after
  the binary works.
- If MCP starts with a plugin-version notice, update the older side instead of
  ignoring the mismatch.
- Do not silently install or update binaries or plugins. Ask before running
  package-manager or Codex plugin commands.
