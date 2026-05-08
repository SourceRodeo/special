# Agents

## MCP

Agents should use Special through controlled tools when possible and the native
binary directly when necessary.

Start the MCP server:

```sh
special mcp
```

The server exposes bounded tools
for status, specs, architecture, patterns, docs, lint, and health.
It can also return docs output
and report a
plugin version notice
when the installed plugin and binary disagree.

Verify the native binary first:

```sh
special --version
special lint
```

## Codex Plugin

Install the SourceRodeo marketplace, then install the Special plugin from that
marketplace:

```sh
codex plugin marketplace add SourceRodeo/codex-marketplace
```

The plugin source
supplies workflow skills and MCP configuration. It does not replace the native
binary; plugin setup should still guide users to
Homebrew or
Cargo when
`special` is missing.

## Bundled Skills

When the plugin path is unavailable, use the bundled skills surface:

```sh
special skills
special skills install --destination project
```

Repo-local installs write `.agents/skills/<skill-id>/SKILL.md`. Global installs
write to `$CODEX_HOME/skills` or `~/.codex/skills`.
The bundled command supports
project destinations,
global destinations,
custom destinations,
and progressive-disclosure references.
