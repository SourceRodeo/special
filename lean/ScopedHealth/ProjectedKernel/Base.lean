import Lean

/-
Base definitions for the projected traceability kernel.

This module contains the parsed input shape, finite list-set operations, and the
interpretation of the input edge map as a reverse support relation. It contains
no JSON transport logic and no preservation theorem; it is the small vocabulary
shared by the executable kernel and its proof modules.
-/

namespace SpecialProofs
namespace ScopedHealth
namespace ProjectedKernel

abbrev EdgeMap := List (String × List String)

/--
The proof boundary for the executable kernel.

JSON decoding must produce this structure before the graph theorems apply.
Theorems reason about these fields directly, not about raw JSON syntax or object
behavior.
-/
structure KernelInput where
  projectedItemIds : List String
  explicitTargetIds : Option (List String)
  edges : EdgeMap
  supportRootIds : List String

def schemaVersion : Nat := 1

def contains (items : List String) (item : String) : Bool :=
  items.any (fun candidate => candidate == item)

def insertUnique (item : String) (items : List String) : List String :=
  if contains items item then items else item :: items

def unionUnique (left right : List String) : List String :=
  right.foldl (fun acc item => insertUnique item acc) left

def reverseNeighbors (edges : EdgeMap) (target : String) : List String :=
  edges.foldl
    (fun callers edge =>
      let caller := edge.fst
      let callees := edge.snd
      if contains callees target then insertUnique caller callers else callers)
    []

def edgeNodeIds (edges : EdgeMap) : List String :=
  edges.foldl
    (fun nodes edge =>
      let nodes := insertUnique edge.fst nodes
      unionUnique nodes edge.snd)
    []

def reverseRelation (edges : EdgeMap) : String → String → Prop :=
  fun callee caller => contains (reverseNeighbors edges callee) caller = true

end ProjectedKernel
end ScopedHealth
end SpecialProofs
