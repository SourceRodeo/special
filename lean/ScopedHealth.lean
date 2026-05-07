/-
Lean traceability kernel
========================

This package contains the production scoped traceability kernel and the formal
contract that justifies graph narrowing. The executable JSON adapter lives in
`ScopedHealth.KernelCli`; this root imports both the theorem surface and the
operational kernel used by the released binary.

How to read this package
------------------------

Read the files in this order:

1. `Closure.lean`
   defines paths, reachability from a target set, and relation restriction.
2. `ReverseClosure.lean`
   proves that a kept graph preserves per-target reverse reachability when it
   keeps all nodes reachable from that target set.
3. `SupportWitness.lean`
   lifts the same fact to reachable support roots.
4. `ProjectedContractClosure.lean`
   packages the shared Rust/TypeScript/Go/Python item-level production contract.
5. `FileContractClosure.lean`
   adds the TypeScript file-loading projection.
6. `ProjectedKernel.lean`
   re-exports the executable projected-kernel module family:
   `ProjectedKernel.Base` defines `KernelInput` and graph interpretation,
   `ProjectedKernel.Worklist` proves exact reverse closure,
   `ProjectedKernel.Output` proves support-root and kept-callee filtering plus
   projected-contract preservation, and `ProjectedKernel.Transport` adapts the
   proven kernel to stdin/stdout JSON.
7. `KernelCli.lean`
   adapts the projected kernel to stdin/stdout JSON.

The proof boundary
------------------

The formal graph kernel starts at `KernelInput`. From that parsed input, Lean
proves the target selection, reverse-closure node list, support-root filter,
kept-callee filter, and projected-contract preservation theorems.

`parseInput`, `outputJson`, `run`, and `KernelCli.lean` are transport adapters.
They keep the released binary small and direct. They form a separate protocol
boundary: this package proves the mathematics of narrowing after parsing has
produced `KernelInput`; protocol behavior such as malformed JSON, Lean object
decoding, and duplicate object keys is validated by tests and can be formalized
independently if that boundary ever becomes the theorem object.

The narrowing problem
---------------------

The product behavior for `special health PATH` is conceptually simple: analyze
the repository, then show only the traceability results relevant to `PATH`.

Doing that literally for every scoped request is often too expensive, and some
language tools do not expose results at exactly the display boundary. TypeScript
and Go naturally load files or packages, Rust analysis may require context items
outside the requested path, and Python derives a parser-backed graph over files.
The implementation therefore uses a narrowed kernel: keep the requested output
items plus the trace graph needed to compute their support evidence.

The danger is that "enough surrounding graph" is easy to get subtly wrong. If
we keep too little, a scoped run can lose support roots or reverse callers that
full analysis would have found. If we keep unrelated material, results may be
slow or confusing, but the more serious correctness failure is losing evidence.

This Lean package states the condition under which narrowing is
traceability-preserving. It is the audit contract that lets scoped execution
stand in for "full analysis, then filter" for the traceability evidence named
below.

Objects
-------

Let `α` be the type of analyzed items. Let `R : α -> α -> Prop` be the
backward trace relation: `R callee caller` means that `caller` is direct
evidence for, or directly supports, `callee`. In `KernelInput`, edges are
stored as direct `caller -> callee` lists; `reverseRelation` interprets that
finite input as the incoming-caller relation used by the theorem.

`Path R a b` is the reflexive-transitive closure of `R`. Thus `Path R target x`
means that `x` is in the reverse support/caller closure of `target`.

`Reachable R Target x` means that `x` is reachable from at least one item in the
target predicate `Target`.

`Induced Keep R` is `R` restricted to edges whose endpoints both satisfy
`Keep`. This models running traceability over a narrowed kept graph.

The shared production contract
------------------------------

`ProjectedContractClosureBoundary R` has three predicates:

* `target`    -- the supported projected items chosen as semantic roots
* `projected` -- the output items the user asked to see
* `keep`      -- the item set retained by the narrowed kernel

Its only semantic assumption is:

    keep x <-> projected x \/ Reachable R target x

In words: the narrowed kernel keeps exactly the requested projected outputs plus
the full reverse closure of the supported projected targets.

What is proven
--------------

For every `target` item satisfying `boundary.target target`:

1. Reverse closure is unchanged:

       ReachableFrom (Induced boundary.keep R) target =
       ReachableFrom R target

   So every item that can support `target` in the full graph can still support
   it after narrowing, and narrowing introduces no new support path.

2. Support-root witnesses are unchanged for any root predicate `IsRoot`:

       SupportRootsFor (Induced boundary.keep R) IsRoot target =
       SupportRootsFor R IsRoot target

   So the kept graph preserves exactly the same reachable roots, such as tests
   or specs, for every supported projected target.

`ProjectedKernel.Worklist` then attaches this abstract theorem to the executable
closure calculation. The worklist starts from the selected target ids, closes
over the finite input graph, and produces exactly the mathematical reverse
closure:

      contains (reverseClosureNodes edges targets) x = true <->
      Reachable (reverseRelation edges) (seedPredicate targets) x

This connects the emitted `node_ids` list to the same reachability predicate
used by the graph theorem.

`ProjectedKernel.Output` proves the executable support-root filter against the
same relation:

      contains (supportRootsFor input target) root = true <->
      supportRootPredicate input root /\ Path (reverseRelation input.edges) target root

The `internalCalleeIds` helper used by the transport encoder is also proven to
keep exactly the original callees that appear in the emitted node set.

TypeScript has one extra execution-layer theorem,
`FileContractClosureBoundary`, because its analyzer loads files rather than
individual items. If the kept file set is exactly the projected files plus the
files that own the reverse closure of the target items, then the same two
item-level preservation results hold when `Keep x` is defined as "the owner file
of `x` is kept".

Transport and upstream boundaries
---------------------------------

The kernel proof starts at `KernelInput`, so parser correctness,
language-server correctness, quality metrics, cache correctness, JSON parser and
object semantics, duplicate JSON object key behavior, and language-pack fact
construction are upstream or transport contracts. This package proves the graph
calculation performed after parsing: reverse-closure `node_ids`, support-root
filtering, the kept-callee helper used inside `internal_edges`, and the
preservation contract those outputs inhabit.
-/

import ScopedHealth.Closure
import ScopedHealth.ReverseClosure
import ScopedHealth.SupportWitness
import ScopedHealth.ProjectedContractClosure
import ScopedHealth.FileContractClosure
import ScopedHealth.ProjectedKernel
