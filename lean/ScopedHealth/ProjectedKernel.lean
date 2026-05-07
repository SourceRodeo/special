import ScopedHealth.ProjectedKernel.Transport

/-
Executable projected traceability kernel
========================================

This public module re-exports the projected traceability kernel.

Module map
----------

1. `ProjectedKernel.Base`
   Defines `KernelInput`, finite list-set operations, and the interpreted
   reverse relation. This is the formal graph proof boundary.
2. `ProjectedKernel.Worklist`
   Computes the reverse closure and proves `reverseClosureNodes_correct`.
3. `ProjectedKernel.Output`
   Defines target selection, support-root filtering, internal-edge filtering,
   and the projected-contract preservation bridge.
4. `ProjectedKernel.Transport`
   Adapts raw stdin/stdout JSON to the proven `KernelInput` kernel. This is the
   protocol boundary around the graph theorem, not part of the graph theorem
   itself.

Proof map
---------

The central executable theorem is `reverseClosureNodes_correct`: membership in
the emitted node list is exactly reachability from the selected target set in
the interpreted reverse relation. `supportRootsFor_correct` and
`internalCalleeIds_correct` prove the output filters over that exact closure,
and `executable_target_*_preserved` connects the executable keep predicate to
the abstract projected-contract preservation theorem. The transport module then
serializes those proven graph outputs.
-/
