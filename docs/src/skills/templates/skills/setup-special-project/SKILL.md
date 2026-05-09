---
name: setup-special-project
description: 'Use this skill when configuring or validating Special in a project where the `special` binary is available. Choose a fresh-project or existing-project setup path, wire docs outputs, and run checks.'
---
@filedocuments spec SPECIAL.INIT.CREATES_SPECIAL_TOML

# Setup Special Project
@implements SPECIAL.DOCUMENTATION.SKILLS.FALLBACK.SETUP_SPECIAL_PROJECT
@applies DOCS.SKILL_MAIN_ENTRY

Use this skill when a project already has the `special` binary available and
needs repo setup, configuration review, or validation.

## When To Use
@applies DOCS.SKILL_TRIGGER_BOUNDARY_SECTION

Use this when `special --version` already works and the task is to configure or
check a repository. Do not use this to install or upgrade the binary.

## Workflow
@applies DOCS.SKILL_WORKFLOW_SECTION

1. Check the installed binary:

   ```sh
   special --version
   ```

2. Check whether the repo already has Special config:

   ```sh
   special
   special lint
   ```

3. If no `special.toml` exists and the user wants to adopt Special, run [`special init`](documents://spec/SPECIAL.INIT.CREATES_SPECIAL_TOML):

   ```sh
   special init
   ```

4. Choose the first inspection path:

   For an existing project, start with [health](documents://spec/SPECIAL.HEALTH_COMMAND.METRICS) and [pattern](documents://spec/SPECIAL.PATTERNS.METRICS) signals that work before heavy
   annotation:

   ```sh
   special health --metrics
   special patterns --metrics
   ```

   For a fresh project or new slice, start with the first [claim](documents://spec/SPECIAL.SPEC_COMMAND) and [ownership](documents://spec/SPECIAL.MODULE_COMMAND)
   boundary:

   ```sh
   special specs
   special arch
   ```

5. If public docs output is configured, check `special.toml` for
   [`[[docs.outputs]]`](documents://spec/SPECIAL.CONFIG.SPECIAL_TOML.DOCS_PATHS) entries and use:

   ```sh
   special docs build
   special docs --metrics
   ```

   For docs claim audits, use [docs metrics](documents://spec/SPECIAL.DOCS_COMMAND.METRICS) as the inventory and [trace](documents://spec/SPECIAL.TRACE_COMMAND.DOCS) for the
   focused relationship packet:

   ```sh
   special docs --metrics --verbose --target docs/src
   special trace docs --target docs/src/guide.md
   ```

6. If Codex [MCP integration](documents://spec/SPECIAL.MCP_COMMAND) is needed, verify the server through the plugin or
   with a minimal JSON-RPC client. The server command is:

   ```sh
   special mcp
   ```

7. Finish setup by running:

   ```sh
   special lint
   special docs --metrics
   ```

## Configuration Notes
@applies DOCS.SKILL_RESULT_DISPOSITION_SECTION

- Keep generated docs outputs in `ignore` if they should not be rediscovered by
  Special.
- Prefer [`[[docs.outputs]]`](documents://spec/SPECIAL.CONFIG.SPECIAL_TOML.DOCS_PATHS) for repeatable docs output paths.
- Use `[health] ignore-unexplained` only for generated or fixture-heavy paths
  that should not count as unexplained code.
- Use `[toolchain]` only when the project needs an explicit tool manager
  contract.

Do not install or update the `special` binary from this skill. If the binary is
missing or stale and this came from the Codex plugin, use the plugin bootstrap
install/update skill.
