# Rendering and Docs Output

Special builds typed domain documents before rendering text, JSON, or HTML.
Rendering changes should preserve the command contract first, then update text,
JSON, HTML, docs, and tests together.

## Output Parity

Text, JSON, and HTML do not need identical structure. They do need the same
command-owned information unless a product contract documents an exception.
Use the shared projection layer for renderer policy, then test the relevant
command family across formats so text cannot silently summarize data that JSON
or HTML still exposes, or vice versa.

The current parity contract is
output parity.

## Generated Docs

`special docs build` writes generated markdown from configured docs source and
strips authoring annotations from the output. The build surface is governed by
docs output,
directory output,
configured output mappings,
overwrite safety, and
authoring-line stripping.

## Docs Metrics

Docs metrics are the docs-native resource view. Keep relationship inventory and
generated-page reachability in docs:
metrics,
relationship inventory,
and interconnectivity.
Broad public-docs coverage also belongs to docs metrics:
coverage and
docs-source declaration scoping.
Focused relationship-chain review belongs to
`special trace`.

Preserve this command boundary in renderers and MCP descriptors: focused
resource commands inspect their own surface, health reports broad inferred
repo signals and gaps, and trace follows the relevant resource chain.

## MCP Output

The MCP server exposes bounded render and docs-output tools for agents. Keep
the tool list, docs output tool, and binary-version notice in sync with CLI
behavior:
MCP command,
MCP tools,
MCP docs output, and
plugin version notice.
