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
for status, specs, architecture, patterns, docs, lint, and health.
It can also return [docs output](documents://spec/SPECIAL.MCP_COMMAND.DOCS_OUTPUT)
and report a
[plugin version notice](documents://spec/SPECIAL.MCP_COMMAND.PLUGIN_VERSION_NOTICE)
when the installed plugin and binary disagree.

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

The [plugin source](documents://spec/SPECIAL.DISTRIBUTION.CODEX_PLUGIN.SOURCE_LAYOUT)
supplies workflow skills and MCP configuration. It does not replace the native
binary; plugin setup should still guide users to
[Homebrew](documents://spec/SPECIAL.DISTRIBUTION.HOMEBREW.INSTALLS_SPECIAL) or
[Cargo](documents://spec/SPECIAL.DISTRIBUTION.CRATES_IO.BINARY_NAME) when
`special` is missing.

@implements SPECIAL.DOCUMENTATION.PUBLIC.AGENTS.SKILLS
## Bundled Skills

When the plugin path is unavailable, use the bundled skills surface:

```sh
special skills
special skills install --destination project
```

Repo-local installs write `.agents/skills/<skill-id>/SKILL.md`. Global installs
write to `$CODEX_HOME/skills` or `~/.codex/skills`.
The bundled command supports
[project destinations](documents://spec/SPECIAL.SKILLS.COMMAND.INSTALL_SUBCOMMAND.PROJECT_DESTINATION),
[global destinations](documents://spec/SPECIAL.SKILLS.COMMAND.INSTALL_SUBCOMMAND.GLOBAL_DESTINATION),
[custom destinations](documents://spec/SPECIAL.SKILLS.COMMAND.INSTALL_SUBCOMMAND.CUSTOM_DESTINATION),
and [progressive-disclosure references](documents://spec/SPECIAL.SKILLS.COMMAND.BUNDLES_REFERENCES_FOR_PROGRESSIVE_DISCLOSURE).
