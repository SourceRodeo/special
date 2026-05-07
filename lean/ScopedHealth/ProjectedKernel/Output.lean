import ScopedHealth.ProjectedKernel.Worklist

/-
Output selection and projected-contract bridge.

This module defines target selection, support-root filtering, and internal-edge
filtering over the proven reverse-closure node set. It also attaches the
executable keep predicate to the abstract projected-contract theorems.

There are three obligations:

* support-root filtering is exactly intersection with the proven closure;
* kept-callee filtering is exactly restriction to emitted node ids;
* the executable keep predicate satisfies the projected-contract boundary.
-/

namespace SpecialProofs
namespace ScopedHealth
namespace ProjectedKernel

def supportRootNodes (supportRootIds reachable : List String) : List String :=
  reachable.foldl
    (fun roots caller =>
      if contains supportRootIds caller then insertUnique caller roots else roots)
    []

def supportRootsFor (input : KernelInput) (target : String) : List String :=
  supportRootNodes input.supportRootIds (reverseClosureNodes input.edges [target])

def supportedProjectedTargetsLoop
    (input : KernelInput)
    (items targets : List String) : List String :=
  match items with
  | [] => targets
  | item :: rest =>
      let roots := supportRootsFor input item
      let targets := if roots.isEmpty then targets else insertUnique item targets
      supportedProjectedTargetsLoop input rest targets

def supportedProjectedTargets (input : KernelInput) : List String :=
  supportedProjectedTargetsLoop input input.projectedItemIds []

def targetIds (input : KernelInput) : List String :=
  match input.explicitTargetIds with
  | some targets => targets
  | none => supportedProjectedTargets input

/-! ## Executable Output Predicates -/

def calleeIdsFor (edges : EdgeMap) (caller : String) : List String :=
  edges.foldl
    (fun callees edge =>
      if edge.fst == caller then unionUnique callees edge.snd else callees)
    []

def internalCalleeIds (edges : EdgeMap) (nodeIds : List String) (caller : String) : List String :=
  (calleeIdsFor edges caller).filter (fun callee => contains nodeIds callee)

def internalEdgePredicate
    (edges : EdgeMap)
    (nodeIds : List String)
    (caller callee : String) : Prop :=
  contains nodeIds caller = true ∧
    contains nodeIds callee = true ∧
      contains (calleeIdsFor edges caller) callee = true

def projectedPredicate (input : KernelInput) : String → Prop :=
  fun item => contains input.projectedItemIds item = true

def targetPredicate (targets : List String) : String → Prop :=
  fun item => contains targets item = true

def exactKeepPredicate (input : KernelInput) (targets : List String) : String → Prop :=
  fun item =>
    projectedPredicate input item ∨
      Reachable (reverseRelation input.edges) (targetPredicate targets) item

def supportRootPredicate (input : KernelInput) : String → Prop :=
  fun item => contains input.supportRootIds item = true

/-! ## Projected-Contract Boundary -/

/--
The executable keep predicate inhabits the abstract projected-contract boundary.

For a concrete input and target list, `keep` is exactly projected output items
plus the reverse closure of those targets.
-/
def projectedKernelBoundary
    (input : KernelInput)
    (targets : List String) :
    ProjectedContractClosureBoundary (reverseRelation input.edges) where
  target := targetPredicate targets
  projected := projectedPredicate input
  keep := exactKeepPredicate input targets
  keep_exact := by
    intro item
    rfl

/-! ## Kept-Callee Filter Exactness -/

/--
`internalCalleeIds` is exactly the original callee list restricted to emitted
node ids.
-/
theorem internalCalleeIds_correct
    {edges : EdgeMap} {nodeIds : List String} {caller callee : String} :
    contains (internalCalleeIds edges nodeIds caller) callee = true ↔
      contains nodeIds callee = true ∧ contains (calleeIdsFor edges caller) callee = true := by
  rw [contains_iff_mem]
  unfold internalCalleeIds
  rw [List.mem_filter]
  constructor
  · intro h
    exact ⟨h.2, contains_of_mem h.1⟩
  · intro h
    exact ⟨contains_true_mem h.2, h.1⟩

/--
When the caller is retained, the internal-edge predicate is exactly membership
in the executable kept-callee list.
-/
theorem internalEdgePredicate_correct
    {edges : EdgeMap} {nodeIds : List String} {caller callee : String}
    (hcaller : contains nodeIds caller = true) :
    contains (internalCalleeIds edges nodeIds caller) callee = true ↔
      internalEdgePredicate edges nodeIds caller callee := by
  unfold internalEdgePredicate
  rw [internalCalleeIds_correct]
  constructor
  · intro h
    exact ⟨hcaller, h.1, h.2⟩
  · intro h
    exact ⟨h.2.1, h.2.2⟩

/-! ## Support-Root Filter Exactness -/

theorem supportRootNodes_fold_exact
    {supportRootIds reachable roots : List String} {item : String} :
    contains
        (reachable.foldl
          (fun roots caller =>
            if contains supportRootIds caller then insertUnique caller roots else roots)
          roots)
        item = true ↔
      contains roots item = true ∨
        (contains supportRootIds item = true ∧ contains reachable item = true) := by
  induction reachable generalizing roots with
  | nil =>
      simp [contains]
  | cons head tail ih =>
      simp [List.foldl_cons]
      by_cases hsupportHead : contains supportRootIds head = true
      · rw [if_pos hsupportHead, ih, contains_insertUnique_iff]
        constructor
        · intro h
          rcases h with hinsert | htail
          · rcases hinsert with hitem | hroot
            · exact Or.inr ⟨by simpa [hitem] using hsupportHead, contains_of_mem (by simp [hitem])⟩
            · exact Or.inl hroot
          · exact Or.inr ⟨htail.1, contains_of_mem (List.mem_cons_of_mem head (contains_true_mem htail.2))⟩
        · intro h
          rcases h with hroot | hreachable
          · exact Or.inl (Or.inr hroot)
          · have hmem : item ∈ head :: tail := contains_true_mem hreachable.2
            simp at hmem
            rcases hmem with hhead | htail
            · exact Or.inl (Or.inl hhead)
            · exact Or.inr ⟨hreachable.1, contains_of_mem htail⟩
      · rw [if_neg hsupportHead, ih]
        constructor
        · intro h
          rcases h with hroot | htail
          · exact Or.inl hroot
          · exact Or.inr ⟨htail.1, contains_of_mem (List.mem_cons_of_mem head (contains_true_mem htail.2))⟩
        · intro h
          rcases h with hroot | hreachable
          · exact Or.inl hroot
          · have hmem : item ∈ head :: tail := contains_true_mem hreachable.2
            simp at hmem
            rcases hmem with hhead | htail
            · have hsupportItem : contains supportRootIds item = true := hreachable.1
              subst hhead
              contradiction
            · exact Or.inr ⟨hreachable.1, contains_of_mem htail⟩

theorem supportRootNodes_correct
    {supportRootIds reachable : List String} {item : String} :
    contains (supportRootNodes supportRootIds reachable) item = true ↔
      contains supportRootIds item = true ∧ contains reachable item = true := by
  unfold supportRootNodes
  rw [supportRootNodes_fold_exact]
  simp [contains]

/--
The executable support-root filter is exact for a single target.

It returns precisely the configured support roots that are reachable from that
target in the interpreted reverse relation.
-/
theorem supportRootsFor_correct
    {input : KernelInput} {target root : String} :
    contains (supportRootsFor input target) root = true ↔
      supportRootPredicate input root ∧ Path (reverseRelation input.edges) target root := by
  unfold supportRootsFor supportRootPredicate
  rw [supportRootNodes_correct]
  have hexact := reverseClosureNodes_correct (edges := input.edges) (targets := [target])
  constructor
  · intro h
    exact ⟨h.1, (reachable_singleton_seed_iff).1 ((hexact root).1 h.2)⟩
  · intro h
    exact ⟨h.1, (hexact root).2 ((reachable_singleton_seed_iff).2 h.2)⟩

/-! ## Preservation Bridge -/

/--
The executable keep predicate has the same per-target reverse closure as the
full graph for every selected target.
-/
theorem executable_target_reverse_closure_preserved
    (input : KernelInput)
    (targets : List String)
    {target : String}
    (htarget : targetPredicate targets target) :
    ReachableFrom
        (Induced (exactKeepPredicate input targets) (reverseRelation input.edges))
        target =
      ReachableFrom (reverseRelation input.edges) target := by
  exact reachable_from_eq_of_projected_contract_closure_boundary
    (projectedKernelBoundary input targets)
    htarget

/--
The executable keep predicate has the same support-root witnesses as the full
graph for every selected target.
-/
theorem executable_target_support_roots_preserved
    (input : KernelInput)
    (targets : List String)
    (isRoot : String → Prop)
    {target : String}
    (htarget : targetPredicate targets target) :
    SupportRootsFor
        (Induced (exactKeepPredicate input targets) (reverseRelation input.edges))
        isRoot
        target =
      SupportRootsFor (reverseRelation input.edges) isRoot target := by
  exact support_roots_for_eq_of_projected_contract_closure_boundary
    (projectedKernelBoundary input targets)
    isRoot
    htarget

end ProjectedKernel
end ScopedHealth
end SpecialProofs
