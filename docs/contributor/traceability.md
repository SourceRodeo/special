# Traceability and Kernel

Traceability answers whether implementation is connected to proof without
pretending every command, process boundary, or framework entrypoint is a valid
proof path. The health command owns the repo-wide traceability surface:
traceability,
default visible summary,
JSON traceability,
and deterministic ordering.

## Process Boundaries

Special intentionally does not trace through process launches, command-line
entrypoints, generated entrypoints, route handlers, or framework dispatch as
preferred proof paths. That is a design pressure toward facades and modules that
tests can call directly. The current product contract is
boundary non-penetration.

## Untraced Implementation

Untraced implementation is a review queue, not a lint failure. Health keeps
unexplained implementation
visible, can include evidence detail,
and allows configured
ignore-unexplained
paths for generated or fixture-heavy code.

## Lean Kernel Boundary

The scoped traceability proof boundary is the Lean projected traceability
kernel. Production must use the Lean kernel, while the Rust reference kernel is
only an explicit debug/test oracle:
Lean kernel selection.
The executable theorem surface must connect reverse-closure computation to
mathematical reachability through
executable reverse closure.

Release builds also embed the Lean kernel for host-native artifacts through
GitHub release Lean kernel packaging.

