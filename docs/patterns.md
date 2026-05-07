# Patterns

Patterns are Special's surface for adopted repeated implementation structures.
Use them for shapes the project wants to recognize and review across the codebase.

Primary command:

```sh
special patterns
```

Primary annotations:

```text
@pattern CACHE.SINGLE_FLIGHT_FILL
Use one in-flight fill per cache key when concurrent callers request the same
expensive value.
```

Apply the pattern where the structure appears:

```ts
// @applies CACHE.SINGLE_FLIGHT_FILL
async function loadOrFillCache(key: string): Promise<Value> {
  return fills.getOrCreate(key, () => rebuildValue(key));
}
```

Inspect usage:

```sh
special patterns CACHE.SINGLE_FLIGHT_FILL --verbose
special patterns --metrics
```

Pattern metrics are advisory fit checks for declared applications. Raw repeated
source shapes and possible missing applications appear in
`special health --metrics`, because
health owns uncaptured analysis queues. A good pattern is identifiable by
structure; a principle like "write clear docs" is not a Special pattern.

For the opinionated admission bar, see [Patternizing Code and Docs](patternizing.md).
