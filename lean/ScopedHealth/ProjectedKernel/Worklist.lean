import ScopedHealth.ProjectedContractClosure
import ScopedHealth.ProjectedKernel.Base

/-
Reverse-closure worklist and exactness proof.

The executable worklist is a finite saturation procedure over the graph nodes
named by the input edge map. The proof separates the calculation into three
mathematical facts about the returned list:

* soundness: every retained node is reachable from a target seed;
* seed inclusion: every target seed remains retained;
* closedness: every reverse predecessor of a retained node is retained.

Together these facts make membership in `reverseClosureNodes` equivalent to
mathematical reachability.
-/

namespace SpecialProofs
namespace ScopedHealth
namespace ProjectedKernel

def extendsReverseClosure (edges : EdgeMap) (nodes : List String) (candidate : String) : Bool :=
  nodes.any (fun item => contains (reverseNeighbors edges item) candidate)

/--
Compute the reverse closure by scanning the finite graph node set until a full
pass reaches a fixed point.

`remaining` is the part of the current pass still to inspect. `skipped` holds
nodes that were not predecessors of the current closure earlier in the pass.
When a node is added, the skipped nodes are put back into `remaining` because
the new node may make them reachable. Termination is structural: every add
removes one graph node from the not-yet-retained graph-node pool, while ordinary
scanning shortens the current pass.
-/
def reverseClosureWork
    (edges : EdgeMap)
    (nodes remaining skipped : List String) : List String :=
  match remaining with
  | [] => nodes
  | candidate :: rest =>
      if contains nodes candidate then
        reverseClosureWork edges nodes rest skipped
      else if extendsReverseClosure edges nodes candidate then
        reverseClosureWork edges (insertUnique candidate nodes) (rest ++ skipped) []
      else
        reverseClosureWork edges nodes rest (candidate :: skipped)
termination_by (remaining.length + skipped.length, remaining.length)
decreasing_by
  · simp_wf
    apply Prod.Lex.left
    omega
  · simp_wf
    apply Prod.Lex.left
    omega
  · simp_wf
    simpa [Nat.add_assoc, Nat.add_comm, Nat.add_left_comm] using
      (Prod.Lex.right (rest.length + 1 + skipped.length) (by omega : rest.length < rest.length + 1))

/--
The executable reverse closure used by the projected graph kernel.

The proof chain below shows that this list contains every seed, contains only
reachable nodes, is closed under reverse predecessors, and is therefore exactly
the mathematical reachability predicate.
-/
def computedReverseClosure (edges : EdgeMap) (targets : List String) : List String :=
  reverseClosureWork edges targets (edgeNodeIds edges) []

def reverseClosureNodes (edges : EdgeMap) (targets : List String) : List String :=
  computedReverseClosure edges targets

def seedPredicate (targets : List String) : String → Prop :=
  fun item => contains targets item = true

/-! ## Finite List Algebra -/

theorem contains_iff_mem {items : List String} {item : String} :
    contains items item = true ↔ item ∈ items := by
  unfold contains
  rw [List.any_eq_true]
  constructor
  · intro h
    rcases h with ⟨candidate, hmem, heq⟩
    have hcandidate : candidate = item := LawfulBEq.eq_of_beq heq
    simpa [hcandidate] using hmem
  · intro h
    exact ⟨item, h, by simp⟩

theorem contains_true_mem {items : List String} {item : String}
    (h : contains items item = true) :
    item ∈ items := by
  exact contains_iff_mem.1 h

theorem contains_of_mem {items : List String} {item : String}
    (h : item ∈ items) :
    contains items item = true := by
  exact contains_iff_mem.2 h

theorem contains_insertUnique_iff {items : List String} {item candidate : String} :
    contains (insertUnique item items) candidate = true ↔
      candidate = item ∨ contains items candidate = true := by
  rw [contains_iff_mem]
  unfold insertUnique
  by_cases hitem : contains items item = true
  · simp [hitem]
    constructor
    · intro h
      exact Or.inr (contains_of_mem h)
    · intro h
      rcases h with hcandidate | hcandidate
      · subst hcandidate
        exact contains_true_mem hitem
      · exact contains_true_mem hcandidate
  · simp [hitem]
    constructor
    · intro h
      rcases h with hcandidate | hcandidate
      · exact Or.inl hcandidate
      · exact Or.inr (contains_of_mem hcandidate)
    · intro h
      rcases h with hcandidate | hcandidate
      · exact Or.inl hcandidate
      · exact Or.inr (contains_true_mem hcandidate)

theorem contains_unionUnique_of_left {left right : List String} {item : String}
    (h : contains left item = true) :
    contains (unionUnique left right) item = true := by
  unfold unionUnique
  induction right generalizing left with
  | nil =>
      simpa using h
  | cons head tail ih =>
      simp [List.foldl_cons]
      apply ih
      exact (contains_insertUnique_iff).2 (Or.inr h)

theorem contains_unionUnique_iff {left right : List String} {item : String} :
    contains (unionUnique left right) item = true ↔
      contains left item = true ∨ contains right item = true := by
  unfold unionUnique
  induction right generalizing left with
  | nil =>
      simp [contains]
  | cons head tail ih =>
      simp [List.foldl_cons, ih]
      constructor
      · intro h
        rcases h with h | h
        · rcases (contains_insertUnique_iff).1 h with hhead | hleft
          · exact Or.inr (contains_of_mem (by simp [hhead]))
          · exact Or.inl hleft
        · exact Or.inr (contains_of_mem (List.mem_cons_of_mem head (contains_true_mem h)))
      · intro h
        rcases h with hleft | hright
        · exact Or.inl ((contains_insertUnique_iff).2 (Or.inr hleft))
        · have hmem : item ∈ head :: tail := contains_true_mem hright
          simp at hmem
          rcases hmem with hhead | htail
          · subst hhead
            exact Or.inl ((contains_insertUnique_iff).2 (Or.inl rfl))
          · exact Or.inr (contains_of_mem htail)

theorem contains_append_of_left {left right : List String} {item : String}
    (h : contains left item = true) :
    contains (left ++ right) item = true := by
  exact contains_of_mem (List.mem_append_left right (contains_true_mem h))

theorem contains_append_of_right {left right : List String} {item : String}
    (h : contains right item = true) :
    contains (left ++ right) item = true := by
  exact contains_of_mem (List.mem_append_right left (contains_true_mem h))

/-! ## Graph-Node Coverage -/

theorem contains_edgeNodeIds_step_of_nodes
    {nodes callees : List String} {caller item : String}
    (h : contains nodes item = true) :
    contains (unionUnique (insertUnique caller nodes) callees) item = true := by
  exact contains_unionUnique_of_left ((contains_insertUnique_iff).2 (Or.inr h))

theorem contains_edgeNodeIds_step_caller
    {nodes callees : List String} {caller : String} :
    contains (unionUnique (insertUnique caller nodes) callees) caller = true := by
  exact contains_unionUnique_of_left ((contains_insertUnique_iff).2 (Or.inl rfl))

theorem reverseNeighbors_subset_edgeNodeIds_from
    {target item : String} {edges : EdgeMap} {callers nodes : List String}
    (hsubset : ∀ candidate,
      contains callers candidate = true → contains nodes candidate = true)
    (hitem : contains
      (edges.foldl
        (fun callers edge =>
          let caller := edge.fst
          let callees := edge.snd
          if contains callees target then insertUnique caller callers else callers)
        callers)
      item = true) :
    contains
      (edges.foldl
        (fun nodes edge =>
          let nodes := insertUnique edge.fst nodes
          unionUnique nodes edge.snd)
        nodes)
      item = true := by
  induction edges generalizing callers nodes with
  | nil =>
      simpa using hsubset item hitem
  | cons edge rest ih =>
      simp [List.foldl_cons] at hitem ⊢
      by_cases hcallee : contains edge.snd target = true
      · apply ih
        · intro candidate hcandidate
          rcases (contains_insertUnique_iff).1 hcandidate with hcaller | hcallers
          · subst hcaller
            exact contains_edgeNodeIds_step_caller
          · exact contains_edgeNodeIds_step_of_nodes (hsubset candidate hcallers)
        · simpa [hcallee] using hitem
      · apply ih
        · intro candidate hcandidate
          exact contains_edgeNodeIds_step_of_nodes (hsubset candidate hcandidate)
        · simpa [hcallee] using hitem

theorem reverseNeighbors_subset_edgeNodeIds
    {edges : EdgeMap} {target item : String}
    (h : contains (reverseNeighbors edges target) item = true) :
    contains (edgeNodeIds edges) item = true := by
  unfold reverseNeighbors at h
  unfold edgeNodeIds
  exact reverseNeighbors_subset_edgeNodeIds_from
    (callers := [])
    (nodes := [])
    (by
      intro candidate hcandidate
      simp [contains] at hcandidate)
    h

theorem extendsReverseClosure_of_relation
    {edges : EdgeMap} {nodes : List String} {callee caller : String}
    (hcallee : contains nodes callee = true)
    (hrel : reverseRelation edges callee caller) :
    extendsReverseClosure edges nodes caller = true := by
  unfold extendsReverseClosure
  rw [List.any_eq_true]
  exact ⟨callee, contains_true_mem hcallee, hrel⟩

/-! ## Reverse-Closure Invariants -/

def reverseClosureSound (edges : EdgeMap) (targets nodes : List String) : Prop :=
  ∀ item, contains nodes item = true →
    Reachable (reverseRelation edges) (seedPredicate targets) item

def reverseClosureIncludesTargets (targets nodes : List String) : Prop :=
  ∀ item, contains targets item = true → contains nodes item = true

def reverseClosureClosedProp (edges : EdgeMap) (nodes : List String) : Prop :=
  ∀ callee caller,
    contains nodes callee = true →
    reverseRelation edges callee caller →
    contains nodes caller = true

/--
Worklist coverage invariant.

Every graph node is either already retained, still waiting in the current pass,
or skipped in the current pass. Closedness follows because every reverse
predecessor of a retained node is known to live somewhere in that executable
state.
-/
def worklistCoversGraphNodes
    (edges : EdgeMap)
    (nodes remaining skipped : List String) : Prop :=
  ∀ item,
    contains (edgeNodeIds edges) item = true →
      contains nodes item = true ∨
        contains remaining item = true ∨
          contains skipped item = true

/--
Stable-pass invariant.

Each node in `skipped` was inspected during the current pass without extending
the retained closure. If the pass ends with this invariant intact, there is no
missing reverse predecessor: any such predecessor would have extended the
closure when it was inspected.
-/
def skippedNodesDoNotExtendClosure
    (edges : EdgeMap)
    (nodes skipped : List String) : Prop :=
  ∀ item,
    contains skipped item = true →
      extendsReverseClosure edges nodes item = false

theorem bool_false_of_not_true {value : Bool}
    (h : ¬value = true) :
    value = false := by
  cases value <;> simp at h ⊢

/--
At the end of a stable pass, the retained set is a fixed point.

Every reverse predecessor of a retained node is a graph node. Coverage says the
predecessor is either retained or has already been inspected in this pass; the
stable-pass invariant rules out the inspected-but-unretained case.
-/
theorem closed_at_end_of_stable_pass
    {edges : EdgeMap} {nodes skipped : List String}
    (hcover : worklistCoversGraphNodes edges nodes [] skipped)
    (hskipped : skippedNodesDoNotExtendClosure edges nodes skipped) :
    reverseClosureClosedProp edges nodes := by
  intro callee caller hcallee hrel
  have hedge : contains (edgeNodeIds edges) caller = true :=
    reverseNeighbors_subset_edgeNodeIds hrel
  rcases hcover caller hedge with hnode | hrest
  · exact hnode
  · rcases hrest with hremaining | hskip
    · simp [contains] at hremaining
    · have hextends := extendsReverseClosure_of_relation hcallee hrel
      have hnot := hskipped caller hskip
      rw [hextends] at hnot
      simp at hnot

/-- Coverage is preserved when the head of `remaining` is already retained. -/
theorem coverage_after_retained_head
    {edges : EdgeMap} {nodes skipped : List String} {candidate : String} {rest : List String}
    (hcover : worklistCoversGraphNodes edges nodes (candidate :: rest) skipped)
    (hcontains : contains nodes candidate = true) :
    worklistCoversGraphNodes edges nodes rest skipped := by
  intro item hedge
  rcases hcover item hedge with hnode | hrest
  · exact Or.inl hnode
  · rcases hrest with hremaining | hskip
    · have hmem : item ∈ candidate :: rest := contains_true_mem hremaining
      simp at hmem
      rcases hmem with hcandidate | hrestMem
      · subst hcandidate
        exact Or.inl hcontains
      · exact Or.inr (Or.inl (contains_of_mem hrestMem))
    · exact Or.inr (Or.inr hskip)

/--
Coverage is preserved when a new predecessor is inserted.

The newly retained node covers the old head of `remaining`; the old skipped
nodes rejoin the pass through `rest ++ skipped`.
-/
theorem coverage_after_inserted_head
    {edges : EdgeMap} {nodes skipped : List String} {candidate : String} {rest : List String}
    (hcover : worklistCoversGraphNodes edges nodes (candidate :: rest) skipped) :
    worklistCoversGraphNodes edges (insertUnique candidate nodes) (rest ++ skipped) [] := by
  intro item hedge
  rcases hcover item hedge with hnode | hrest
  · exact Or.inl ((contains_insertUnique_iff).2 (Or.inr hnode))
  · rcases hrest with hremaining | hskip
    · have hmem : item ∈ candidate :: rest := contains_true_mem hremaining
      simp at hmem
      rcases hmem with hcandidate | hrestMem
      · exact Or.inl ((contains_insertUnique_iff).2 (Or.inl hcandidate))
      · exact Or.inr (Or.inl (contains_append_of_left (contains_of_mem hrestMem)))
    · exact Or.inr (Or.inl (contains_append_of_right hskip))

/-- Coverage is preserved when the head of `remaining` is skipped for this pass. -/
theorem coverage_after_unextended_head
    {edges : EdgeMap} {nodes skipped : List String} {candidate : String} {rest : List String}
    (hcover : worklistCoversGraphNodes edges nodes (candidate :: rest) skipped) :
    worklistCoversGraphNodes edges nodes rest (candidate :: skipped) := by
  intro item hedge
  rcases hcover item hedge with hnode | hrest
  · exact Or.inl hnode
  · rcases hrest with hremaining | hskip
    · have hmem : item ∈ candidate :: rest := contains_true_mem hremaining
      simp at hmem
      rcases hmem with hcandidate | hrestMem
      · exact Or.inr (Or.inr (contains_of_mem (by simp [hcandidate])))
      · exact Or.inr (Or.inl (contains_of_mem hrestMem))
    · exact Or.inr (Or.inr (contains_of_mem (List.mem_cons_of_mem candidate (contains_true_mem hskip))))

/-- With no skipped nodes, stable-pass evidence is immediate. -/
theorem skippedNodesDoNotExtendClosure_empty
    {edges : EdgeMap} {nodes : List String} :
    skippedNodesDoNotExtendClosure edges nodes [] := by
  intro item hitem
  simp [contains] at hitem

/-- Skipping an unextended node preserves stable-pass evidence. -/
theorem skippedNodesDoNotExtendClosure_cons
    {edges : EdgeMap} {nodes skipped : List String} {candidate : String}
    (hskipped : skippedNodesDoNotExtendClosure edges nodes skipped)
    (hextends : ¬extendsReverseClosure edges nodes candidate = true) :
    skippedNodesDoNotExtendClosure edges nodes (candidate :: skipped) := by
  intro item hitem
  have hmem : item ∈ candidate :: skipped := contains_true_mem hitem
  simp at hmem
  rcases hmem with hcandidate | hskipMem
  · subst hcandidate
    exact bool_false_of_not_true hextends
  · exact hskipped item (contains_of_mem hskipMem)

/-! ## Worklist Soundness And Closedness -/

theorem reachable_of_seed_contains
    {edges : EdgeMap} {targets : List String} {item : String}
    (h : contains targets item = true) :
    Reachable (reverseRelation edges) (seedPredicate targets) item := by
  exact ⟨item, h, Path.refl item⟩

theorem reverseClosureSound_initial {edges : EdgeMap} {targets : List String} :
    reverseClosureSound edges targets targets := by
  intro item hitem
  exact reachable_of_seed_contains hitem

theorem reverseClosureIncludesTargets_initial {targets : List String} :
    reverseClosureIncludesTargets targets targets := by
  intro item hitem
  exact hitem

/--
If a graph node directly extends a sound retained set, then that node is
reachable from the original target set.
-/
theorem reachable_of_extendsReverseClosure
    {edges : EdgeMap} {targets nodes : List String} {candidate : String}
    (hsound : reverseClosureSound edges targets nodes)
    (hextends : extendsReverseClosure edges nodes candidate = true) :
    Reachable (reverseRelation edges) (seedPredicate targets) candidate := by
  unfold extendsReverseClosure at hextends
  rw [List.any_eq_true] at hextends
  rcases hextends with ⟨node, hnode, hrel⟩
  exact reachable_step (hsound node (contains_of_mem hnode)) hrel

/-- Adding a reachable node preserves the soundness invariant. -/
theorem reverseClosureSound_insertReachable
    {edges : EdgeMap} {targets nodes : List String} {candidate : String}
    (hsound : reverseClosureSound edges targets nodes)
    (hcandidate : Reachable (reverseRelation edges) (seedPredicate targets) candidate) :
    reverseClosureSound edges targets (insertUnique candidate nodes) := by
  intro item hitem
  rcases (contains_insertUnique_iff).1 hitem with hcandidate_eq | hnodes
  · subst hcandidate_eq
    exact hcandidate
  · exact hsound item hnodes

/-- The worklist never introduces a node outside the mathematical closure. -/
theorem reverseClosureWork_sound
    {edges : EdgeMap} {targets nodes remaining skipped : List String}
    (hsound : reverseClosureSound edges targets nodes) :
    reverseClosureSound edges targets (reverseClosureWork edges nodes remaining skipped) := by
  fun_induction reverseClosureWork edges nodes remaining skipped with
  | case1 =>
      exact hsound
  | case2 nodes skipped candidate rest hcontains ih =>
      exact ih hsound
  | case3 nodes skipped candidate rest hnotContains hextends ih =>
      exact ih
        (reverseClosureSound_insertReachable
          hsound
          (reachable_of_extendsReverseClosure hsound hextends))
  | case4 nodes skipped candidate rest hnotContains hextends ih =>
      exact ih hsound

/-- The worklist never drops an original target seed. -/
theorem reverseClosureWork_includes
    {edges : EdgeMap} {targets nodes remaining skipped : List String}
    (hincludes : reverseClosureIncludesTargets targets nodes) :
    reverseClosureIncludesTargets targets (reverseClosureWork edges nodes remaining skipped) := by
  fun_induction reverseClosureWork edges nodes remaining skipped with
  | case1 =>
      exact hincludes
  | case2 nodes skipped candidate rest hcontains ih =>
      exact ih hincludes
  | case3 nodes skipped candidate rest hnotContains hextends ih =>
      exact ih
        (by
          intro item htarget
          exact (contains_insertUnique_iff).2 (Or.inr (hincludes item htarget)))
  | case4 nodes skipped candidate rest hnotContains hextends ih =>
      exact ih hincludes

/--
The worklist result is closed under reverse predecessors.

At the end of a pass, any reverse predecessor of a retained node is a graph
node. By coverage, that predecessor is either already retained or was skipped
in the pass. A skipped predecessor would contradict pass stability, because it
directly extends the retained closure. Therefore all reverse predecessors are
retained.
-/
theorem reverseClosureWork_closed
    {edges : EdgeMap} {nodes remaining skipped : List String}
    (hcover : worklistCoversGraphNodes edges nodes remaining skipped)
    (hskipped : skippedNodesDoNotExtendClosure edges nodes skipped) :
    reverseClosureClosedProp edges (reverseClosureWork edges nodes remaining skipped) := by
  fun_induction reverseClosureWork edges nodes remaining skipped with
  | case1 =>
      exact closed_at_end_of_stable_pass hcover hskipped
  | case2 nodes skipped candidate rest hcontains ih =>
      exact ih
        (coverage_after_retained_head hcover hcontains)
        hskipped
  | case3 nodes skipped candidate rest hnotContains hextends ih =>
      exact ih
        (coverage_after_inserted_head hcover)
        skippedNodesDoNotExtendClosure_empty
  | case4 nodes skipped candidate rest hnotContains hextends ih =>
      exact ih
        (coverage_after_unextended_head hcover)
        (skippedNodesDoNotExtendClosure_cons hskipped hextends)

theorem reachable_contained_of_closed
    {edges : EdgeMap} {targets nodes : List String}
    (hincludes : reverseClosureIncludesTargets targets nodes)
    (hclosed : reverseClosureClosedProp edges nodes)
    {item : String}
    (hreachable : Reachable (reverseRelation edges) (seedPredicate targets) item) :
    contains nodes item = true := by
  rcases hreachable with ⟨seed, hseed, path⟩
  induction path with
  | refl =>
      exact hincludes seed hseed
  | tail path hedge ih =>
      exact hclosed _ _ ih hedge

/--
A sound, seed-containing, closed set is exactly the reverse closure.

Soundness gives the forward implication. Seed inclusion plus closedness gives
the reverse implication by induction on the reachability path.
-/
theorem reverseClosureExact_of_sound_closed
    {edges : EdgeMap} {targets nodes : List String}
    (hsound : reverseClosureSound edges targets nodes)
    (hincludes : reverseClosureIncludesTargets targets nodes)
    (hclosed : reverseClosureClosedProp edges nodes) :
    ∀ item,
      contains nodes item = true ↔
        Reachable (reverseRelation edges) (seedPredicate targets) item := by
  intro item
  constructor
  · exact hsound item
  · exact reachable_contained_of_closed hincludes hclosed

/-! ## Exactness Of The Executable Reverse Closure -/

theorem computedReverseClosure_sound
    {edges : EdgeMap} {targets : List String} :
    reverseClosureSound edges targets (computedReverseClosure edges targets) := by
  unfold computedReverseClosure
  exact reverseClosureWork_sound
    (reverseClosureSound_initial (edges := edges) (targets := targets))

theorem computedReverseClosure_includes_targets
    {edges : EdgeMap} {targets : List String} :
    reverseClosureIncludesTargets targets (computedReverseClosure edges targets) := by
  unfold computedReverseClosure
  exact reverseClosureWork_includes
    (reverseClosureIncludesTargets_initial (targets := targets))

/-- The executable fixed point is closed under reverse predecessors. -/
theorem computedReverseClosure_closed
    {edges : EdgeMap} {targets : List String} :
    reverseClosureClosedProp edges (computedReverseClosure edges targets) := by
  unfold computedReverseClosure
  apply reverseClosureWork_closed
  · intro item hedge
    exact Or.inr (Or.inl hedge)
  · intro item hitem
    simp [contains] at hitem

/--
The emitted reverse-closure node list is mathematically exact.

This is the central executable theorem: reading membership in
`reverseClosureNodes edges targets` is the same proposition as reachability from
the target set in the interpreted reverse relation.
-/
theorem reverseClosureNodes_correct
    {edges : EdgeMap} {targets : List String} :
    ∀ item,
      contains (reverseClosureNodes edges targets) item = true ↔
        Reachable (reverseRelation edges) (seedPredicate targets) item := by
  unfold reverseClosureNodes
  exact reverseClosureExact_of_sound_closed
    (computedReverseClosure_sound (edges := edges) (targets := targets))
    (computedReverseClosure_includes_targets (edges := edges) (targets := targets))
    (computedReverseClosure_closed (edges := edges) (targets := targets))

theorem reachable_singleton_seed_iff
    {edges : EdgeMap} {target item : String} :
    Reachable (reverseRelation edges) (seedPredicate [target]) item ↔
      Path (reverseRelation edges) target item := by
  constructor
  · intro h
    rcases h with ⟨seed, hseed, path⟩
    have hseed_eq : seed = target := by
      have hmem : seed ∈ [target] := contains_true_mem hseed
      simpa using hmem
    subst hseed_eq
    exact path
  · intro path
    exact ⟨target, contains_of_mem (by simp), path⟩

end ProjectedKernel
end ScopedHealth
end SpecialProofs
