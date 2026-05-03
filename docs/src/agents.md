@filedocuments spec SPECIAL.MCP_COMMAND
@filedocuments spec SPECIAL.MCP_COMMAND.TOOLS
@filedocuments spec SPECIAL.MCP_COMMAND.DOCS_OUTPUT
@filedocuments spec SPECIAL.SKILLS.COMMAND.HELP
# Agents and MCP

Special is designed to give coding agents controlled access to repo truth without
making them scrape arbitrary files first.

## MCP Server

```sh
special mcp
```

The server speaks stdio MCP. It exposes controlled tools for status, overview,
specs, architecture, patterns, docs validation, docs output, lint, and health.

The first MCP surface is intentionally inspection-heavy. It does not make broad
source edits or mutate project configuration. Docs output is exposed as a bounded
write path because it follows the same explicit output safety policy as the CLI.

## Codex Plugin

Codex users should install the SourceRodeo marketplace and then install the
Special plugin:

```sh
codex plugin marketplace add SourceRodeo/codex-marketplace
```

The plugin includes:

- a Special workflow skill
- install/update guidance for the native binary
- setup/configuration guidance
- MCP configuration for `special mcp`

## Bundled Skills

For users who do not use plugins, the binary can print or install bundled skills:

```sh
special skills
special skills install
```

Repo-local installs write `.agents/skills/<skill-id>/SKILL.md`. Global installs
write to `$CODEX_HOME/skills` or `~/.codex/skills`.
