# Backlog

## 0.7.2 Lean Kernel And Analyzer Hardening

- Move traceability's shared reasoning into a production Lean FFI kernel. Language packs should emit canonical trace facts; Lean should own closure, projection, support roots, exact item kernels, and summary invariants.
- Keep the FFI boundary small: one canonical fact input schema and one structured traceability result output schema. Avoid exposing a broad fine-grained Lean API to each language pack.
- Upgrade TypeScript and Go analyzers from heuristic tree-sitter metric walkers toward stronger language-native front ends. They should produce canonical facts with parser/tool-backed identities, spans, call edges, type information, and item metadata.
- Keep Rust `syn` plus tool-backed analysis, but route it through the same canonical adapter shape as TypeScript and Go so parity is schema and fact-conformance based rather than reimplemented kernel behavior.
- Reframe parity tests around adapter/schema conformance plus shared Lean-kernel outputs. Each pack should prove that it emits comparable canonical facts; the Lean kernel should be tested once for the shared reasoning.
- Add analyzer regressions for nested callable attribution, same-line item identity collisions, parameter-name/type shadowing, direct boolean-operator detection, and language-specific branch constructs.
- Treat generated, minified, and dense one-line source as supported analyzer input when the language parser can identify source items honestly.
