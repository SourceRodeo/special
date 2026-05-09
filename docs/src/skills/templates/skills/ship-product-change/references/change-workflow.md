@filedocuments spec SPECIAL.TRACE_COMMAND.SPECS

# Change Workflow
@implements SPECIAL.DOCUMENTATION.SKILLS.FALLBACK.SHIP_PRODUCT_CHANGE
@applies DOCS.SKILL_SUPPORT_REFERENCE

Use this checklist when shipping a change that should keep the product contract honest.

1. Identify the exact behavior that changed.
2. If the claim does not exist yet, add it before or alongside the implementation.
3. Keep the claim present tense and narrow enough that one verify can honestly support it.
4. Use [`special specs SPEC.ID --verbose`](documents://spec/SPECIAL.SPEC_COMMAND.VERBOSE) to inspect existing support when Special is configured.
5. Tighten weak verifies until a reviewer can judge the claim from the test or attestation body.
6. If the work is not ready to ship, keep the claim planned instead of pretending it is current.
7. If ownership changed, update [`@module`](documents://spec/SPECIAL.MODULE_COMMAND.MARKDOWN_DECLARATIONS) / [`@implements`](documents://spec/SPECIAL.MODULE_PARSE.IMPLEMENTS.ITEM_SCOPE).
8. If the implementation follows a recurring approach, update [`@pattern`](documents://spec/SPECIAL.PATTERNS.DEFINITIONS) / [`@applies`](documents://spec/SPECIAL.PATTERNS.SOURCE_APPLICATIONS).
9. Run the focused Special checks that match the surfaces you touched.
10. Keep the contract focused on stable, externally meaningful invariants and avoid verifies that overfit transient details.

Good examples:

- Spec: `CSV exports include a header row with the selected column names.`
- Verify: exercise the export path and assert the first CSV row is the selected header row.
- Spec: `special skills install writes project-local installs under .agents/skills/.`
- Verify: run the install command and assert the destination directory and `SKILL.md` exist there.

Bad examples:

- Spec: `The help copy says "Select install destination".`
- Verify: assert an exact instructional paragraph in a bundled skill file.
- Spec: `The command calls helper parse_destination before install_bundled_skills.`

If the true contract is a side effect, interaction assertions can be part of the proof. Otherwise, prefer end-state and output checks over call-order checks.

Example verify shape:

```python
# @verifies EXPORT.CSV.HEADERS
def test_csv_export_includes_selected_column_headers():
    csv_text = export_orders_csv(columns=["order_id", "status"])
    assert csv_text.splitlines()[0] == "order_id,status"
```
