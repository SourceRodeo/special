@applies DOCS.CONTRIBUTOR_RUNBOOK_PAGE
@implements SPECIAL.DOCUMENTATION.CONTRIBUTOR.TRACEABILITY
# Traceability and Kernel

Traceability answers whether implementation is connected to proof without
pretending every command, process boundary, or framework entrypoint is a valid
proof path. The health command owns the repo-wide traceability surface:
[traceability](documents://spec/SPECIAL.HEALTH_COMMAND.TRACEABILITY),
[default visible summary](documents://spec/SPECIAL.HEALTH_COMMAND.TRACEABILITY.DEFAULT_VISIBLE),
[JSON traceability](documents://spec/SPECIAL.HEALTH_COMMAND.JSON.TRACEABILITY),
and [deterministic ordering](documents://spec/SPECIAL.HEALTH_COMMAND.TRACEABILITY.DETERMINISTIC_ORDERING).

## Process Boundaries

Special intentionally does not trace through process launches, command-line
entrypoints, generated entrypoints, route handlers, or framework dispatch as
preferred proof paths. That is a design pressure toward facades and modules that
tests can call directly. The current product contract is
[boundary non-penetration](documents://spec/SPECIAL.HEALTH_COMMAND.TRACEABILITY.BOUNDARY_NON_PENETRATION).

## Untraced Implementation

Untraced implementation is a review queue, not a lint failure. Health keeps
[unexplained implementation](documents://spec/SPECIAL.HEALTH_COMMAND.TRACEABILITY.UNEXPLAINED)
visible, can include [evidence detail](documents://spec/SPECIAL.HEALTH_COMMAND.TRACEABILITY.UNEXPLAINED.EVIDENCE),
and allows configured
[ignore-unexplained](documents://spec/SPECIAL.HEALTH_COMMAND.TRACEABILITY.IGNORE_UNEXPLAINED)
paths for generated or fixture-heavy code.

## Lean Kernel Boundary

The scoped traceability proof boundary is the Lean projected traceability
kernel. Production must use the Lean kernel, while the Rust reference kernel is
only an explicit debug/test oracle:
[Lean kernel selection](documents://spec/SPECIAL.HEALTH_COMMAND.TRACEABILITY.LEAN_KERNEL).
The executable theorem surface must connect reverse-closure computation to
mathematical reachability through
[executable reverse closure](documents://spec/SPECIAL.HEALTH_COMMAND.TRACEABILITY.LEAN_KERNEL.EXECUTABLE_REVERSE_CLOSURE).

Release builds also embed the Lean kernel for host-native artifacts through
[GitHub release Lean kernel packaging](documents://spec/SPECIAL.DISTRIBUTION.GITHUB_RELEASES.LEAN_KERNEL).

