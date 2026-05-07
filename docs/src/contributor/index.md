@applies DOCS.CONTRIBUTOR_RUNBOOK_PAGE
@implements SPECIAL.DOCUMENTATION.CONTRIBUTOR.INDEX
# Contributor Reference

These pages are for maintainers changing Special itself. User-facing behavior
belongs in the public docs; implementation boundaries that affect releases,
proof, parsers, renderers, caches, or quality gates belong here.

Use this path before release work:

```sh
special docs --metrics
special arch SPECIAL.DOCUMENTATION.CONTRIBUTOR --metrics
special patterns DOCS.CONTRIBUTOR_RUNBOOK_PAGE --verbose
special health --metrics --target docs/src/contributor --within docs/src
```

Core references:

- [Release and Distribution](release.md)
- [Parser and Annotation Rules](parser.md)
- [Language Packs](language-packs.md)
- [Traceability and Kernel](traceability.md)
- [Rendering and Docs Output](rendering.md)
- [Cache Behavior](cache.md)
- [Quality Gates](quality.md)
