---
name: setup-special-project
description: Use this skill when configuring, validating, or updating Special in a project where the `special` binary is available. Initialize config, inspect specs, wire docs outputs, and run checks.
---

# Setup Special Project

Use this skill when a project already has the `special` binary available and
needs repo setup, configuration review, or validation.

## Workflow

1. Check the installed binary:

   ```sh
   special --version
   ```

2. Check whether the repo already has Special config:

   ```sh
   special
   special lint
   ```

3. If no `special.toml` exists and the user wants to adopt Special, run:

   ```sh
   special init
   ```

4. Inspect the current contract state:

   ```sh
   special specs
   special arch
   special patterns
   special docs
   special health
   ```

5. If public docs output is configured, check `special.toml` for
   `[[docs.outputs]]` entries and use:

   ```sh
   special docs --output
   ```

6. If Codex MCP integration is needed, verify the server through the plugin or
   with a minimal JSON-RPC client. The server command is:

   ```sh
   special mcp
   ```

7. Finish setup by running:

   ```sh
   special lint
   special docs
   ```

## Configuration Notes

- Keep generated docs outputs in `ignore` if they should not be rediscovered by
  Special.
- Prefer `[[docs.outputs]]` for repeatable docs output paths.
- Use `[health] ignore-unexplained` only for generated or fixture-heavy paths
  that should not count as unexplained code.
- Use `[toolchain]` only when the project needs an explicit tool manager
  contract.

Do not install or update the `special` binary from this skill. If the binary is
missing or stale and this came from the Codex plugin, use the plugin bootstrap
install/update skill.
