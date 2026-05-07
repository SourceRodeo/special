import Lean
import ScopedHealth.ProjectedKernel.Output

/-
JSON transport adapter for the projected traceability kernel.

The formal graph proof starts at `KernelInput`. This module stays deliberately
thin: decode stdin JSON to `KernelInput`, run the proven graph/output functions,
and encode the result as stdout JSON.
-/

namespace SpecialProofs
namespace ScopedHealth
namespace ProjectedKernel

open Lean

def stringArrayJson (items : List String) : Json :=
  Json.arr <| items.reverse.map Json.str |>.toArray

/-- Normalize protocol arrays into the finite list-set representation used by the kernel. -/
def uniqueStringList (items : List String) : List String :=
  items.foldl (fun unique item => insertUnique item unique) []

def stringArrayFromJson (json : Json) : Except String (List String) := do
  let items ← json.getArr?
  let strings ← items.toList.mapM fun item => item.getStr?
  return uniqueStringList strings

def optionalStringArrayFromJson (json : Json) : Except String (Option (List String)) := do
  match json with
  | Json.null => pure none
  | _ => return some (← stringArrayFromJson json)

def edgeMapFromJson (json : Json) : Except String EdgeMap := do
  let object ← json.getObj?
  object.toList.mapM fun (caller, callees) => do
    return (caller, ← stringArrayFromJson callees)

def parseInput (json : Json) : Except String KernelInput := do
  let schemaVersion ← (← json.getObjVal? "schema_version").getNat?
  if schemaVersion != ProjectedKernel.schemaVersion then
    throw s!"unsupported traceability kernel schema version {schemaVersion}"
  return {
    projectedItemIds := ← stringArrayFromJson (← json.getObjVal? "projected_item_ids")
    explicitTargetIds := ← optionalStringArrayFromJson
      (← json.getObjVal? "preserved_reverse_closure_target_ids")
    edges := ← edgeMapFromJson (← json.getObjVal? "edges")
    supportRootIds := ← stringArrayFromJson (← json.getObjVal? "support_root_ids")
  }

def internalEdgesJson (edges : EdgeMap) (nodeIds : List String) : Json :=
  let edgeJson :=
    edges.foldl
      (fun entries edge =>
        let caller := edge.fst
        if contains nodeIds caller then
          let keptCallees := internalCalleeIds edges nodeIds caller
          (caller, stringArrayJson keptCallees) :: entries
        else
          entries)
      []
  Json.mkObj edgeJson.reverse

def outputJson (input : KernelInput) : Json :=
  let targetIds := targetIds input
  let nodeIds := reverseClosureNodes input.edges targetIds
  Json.mkObj [
    ("schema_version", toJson schemaVersion),
    ("reference", Json.mkObj [
      ("contract", Json.mkObj [
        ("projected_item_ids", stringArrayJson input.projectedItemIds),
        ("preserved_reverse_closure_target_ids", stringArrayJson targetIds)
      ]),
      ("exact_reverse_closure", Json.mkObj [
        ("target_ids", stringArrayJson targetIds),
        ("node_ids", stringArrayJson nodeIds),
        ("internal_edges", internalEdgesJson input.edges nodeIds)
      ])
    ])
  ]

/--
Transport entrypoint for the standalone executable.

The graph theorem begins after `parseInput` returns `KernelInput`. This function
is intentionally a small adapter from stdin JSON to the proven list kernel and
back to stdout JSON.
-/
def run (stdin : String) : Except String String := do
  let input ← parseInput (← Json.parse stdin)
  return (outputJson input).compress

end ProjectedKernel
end ScopedHealth
end SpecialProofs
