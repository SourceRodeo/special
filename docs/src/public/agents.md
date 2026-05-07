@applies DOCS.AGENT_SETUP_PATH
# Agents

@implements SPECIAL.DOCUMENTATION.PUBLIC.AGENTS.MCP
## MCP

Agents should use Special through controlled tools when possible and the native
binary directly when necessary.

Start the MCP server:

```sh
special mcp
```

The server exposes [bounded tools](documents://spec/SPECIAL.MCP_COMMAND.TOOLS)
for status, overview, specs, architecture, patterns, docs, lint, and health.

Verify the native binary first:

```sh
special --version
special lint
```

@implements SPECIAL.DOCUMENTATION.PUBLIC.AGENTS.PLUGIN
## Codex Plugin

Install the SourceRodeo marketplace, then install the Special plugin from that
marketplace:

```sh
codex plugin marketplace add SourceRodeo/codex-marketplace
```

The plugin supplies workflow skills and MCP configuration. It does not replace
the native binary; plugin setup should still guide users to Homebrew or Cargo
when `special` is missing.

@implements SPECIAL.DOCUMENTATION.PUBLIC.AGENTS.SKILLS
## Bundled Skills

When the plugin path is unavailable, use the bundled skills surface:

```sh
special skills
special skills install --destination project
```

Repo-local installs write `.agents/skills/<skill-id>/SKILL.md`. Global installs
write to `$CODEX_HOME/skills` or `~/.codex/skills`.
