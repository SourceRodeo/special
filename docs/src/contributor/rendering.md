@applies DOCS.CONTRIBUTOR_RUNBOOK_PAGE
@implements SPECIAL.DOCUMENTATION.CONTRIBUTOR.RENDERING
# Rendering and Docs Output

Special builds typed domain documents before rendering text, JSON, or HTML.
Rendering changes should preserve the command contract first, then update text,
JSON, HTML, docs, and tests together.

## Generated Docs

`special docs build` writes generated markdown from configured docs source and
strips authoring annotations from the output. The build surface is governed by
[docs output](documents://spec/SPECIAL.DOCS_COMMAND.OUTPUT),
[directory output](documents://spec/SPECIAL.DOCS_COMMAND.OUTPUT.DIRECTORY),
[configured output mappings](documents://spec/SPECIAL.DOCS_COMMAND.OUTPUT.CONFIG),
[overwrite safety](documents://spec/SPECIAL.DOCS_COMMAND.OUTPUT.SAFETY), and
[authoring-line stripping](documents://spec/SPECIAL.DOCS_COMMAND.OUTPUT.AUTHORING_LINES).

## Docs Metrics

Docs metrics are the docs-native graph view. Keep relationship inventory,
generated-page reachability, target coverage, and target support audit in docs,
not health:
[metrics](documents://spec/SPECIAL.DOCS_COMMAND.METRICS),
[relationship inventory](documents://spec/SPECIAL.DOCS_COMMAND.METRICS.RELATIONSHIPS),
[coverage](documents://spec/SPECIAL.DOCS_COMMAND.METRICS.COVERAGE),
[interconnectivity](documents://spec/SPECIAL.DOCS_COMMAND.METRICS.INTERCONNECTIVITY),
[target support audit](documents://spec/SPECIAL.DOCS_COMMAND.METRICS.TARGET_AUDIT), and
[docs-source declaration scoping](documents://spec/SPECIAL.DOCS_COMMAND.METRICS.COVERAGE.DOCS_SOURCE_DECLARATIONS).

## MCP Output

The MCP server exposes bounded render and docs-output tools for agents. Keep
the tool list, docs output tool, and binary-version notice in sync with CLI
behavior:
[MCP command](documents://spec/SPECIAL.MCP_COMMAND),
[MCP tools](documents://spec/SPECIAL.MCP_COMMAND.TOOLS),
[MCP docs output](documents://spec/SPECIAL.MCP_COMMAND.DOCS_OUTPUT), and
[plugin version notice](documents://spec/SPECIAL.MCP_COMMAND.PLUGIN_VERSION_NOTICE).

