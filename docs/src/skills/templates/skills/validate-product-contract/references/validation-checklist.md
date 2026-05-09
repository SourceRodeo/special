@filedocuments spec SPECIAL.TRACE_COMMAND.SPECS

# Validation Checklist
@implements SPECIAL.DOCUMENTATION.SKILLS.FALLBACK.VALIDATE_PRODUCT_CONTRACT
@applies DOCS.SKILL_SUPPORT_REFERENCE

Use this checklist when deciding whether a claim is honestly supported:

1. Read the exact claim and keep the wording in view.
2. Confirm the support is attached to that exact claim, not just a nearby parent or child.
3. Confirm the support body is self-contained enough to judge locally.
4. Confirm the verify sits at the right abstraction boundary for the claim.
5. Use [`special trace specs --id SPEC.ID`](documents://spec/SPECIAL.TRACE_COMMAND.SPECS) when you need the claim and its support evidence in one packet.
6. Use [`special specs --metrics`](documents://spec/SPECIAL.SPEC_COMMAND.METRICS) when you need broader support or lifecycle counts.
7. Confirm the claim is current only if the support is genuinely good enough.

Good pairing:

```text
Claim: CSV exports include a header row with the selected column names.
Verify: A test that exports CSV with specific selected columns and asserts the first line is exactly those column names.
```
