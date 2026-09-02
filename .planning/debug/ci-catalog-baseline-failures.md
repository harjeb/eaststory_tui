---
status: awaiting_human_verify
trigger: "CI cargo test --lib has 4 failures: stale-looking M5/M6/M7 catalog count assertions and shared behavior ledger status validation."
created: 2026-09-02T10:06:31+08:00
updated: 2026-09-02T10:29:00+08:00
---

## Current Focus

hypothesis: Verified locally: the minimal consistency patch resolves all four failures and preserves the stronger ledger invariants.
test: Await CI or user confirmation from a full checkout after local complete-fixture verification passed.
expecting: Hosted CI reproduces the local result with zero test or Clippy failures.
next_action: Parent agent should review the patch, update the Beads issue, and commit/push as requested; archive this session only after hosted CI or user confirmation.

## Symptoms

expected: `cargo test --all-targets --all-features --locked` passes, including all catalog and ledger integrity tests.
actual: 163 tests pass and 4 fail.
errors: M5 count mismatch 38 vs 21 at src/content.rs:1631; M6 count mismatch 31 vs 23 at src/content.rs:831; M7 count mismatch 51 vs 50 at src/content.rs:1115; behavior ledger contains at least one status outside verified/deferred/excluded at src/items.rs:651.
reproduction: In a complete checkout, run `cargo test --all-targets --all-features --locked`; focused lib tests reproduce independently.
started: Surfaced after strict Clippy errors were fixed and CI advanced to tests; previously reproduced in the isolated checkout.

## Eliminated

- hypothesis: The JSON ledger summaries are corrupt and should be reverted to the old test values.
  evidence: Status counts recomputed from every disposition equal the current summaries exactly and total 59, 81, and 183; commit 6f60acb also added the corresponding runtime behavior and tests.
  timestamp: 2026-09-02T10:16:00+08:00

- hypothesis: `adapted` is an accidental status in the shared item ledger.
  evidence: Commit 6f60acb deliberately changed obj.dust from deferred to adapted when corpse dissolution was implemented; the summary and entry counts both report exactly one adapted item.
  timestamp: 2026-09-02T10:16:00+08:00

## Evidence

- timestamp: 2026-09-02T10:06:31+08:00
  checked: Initial repository state
  found: `es2-utf8` is intentionally dirty and `.clippy-validation` is untracked; neither is owned by this debug task.
  implication: Investigation and verification must preserve those paths and may use a complete isolated checkout if needed.

- timestamp: 2026-09-02T10:16:00+08:00
  checked: Exact failing assertions
  found: M5 fails at adapted expected 21/current 38; M6 at adapted expected 23/current 31; M7 at adapted expected 50/current 51; shared items rejects the sole adapted obj.dust entry.
  implication: The failures are stale status expectations, not catalog room counts.

- timestamp: 2026-09-02T10:16:00+08:00
  checked: Git history at commit 6f60acb
  found: The M8 completion commit intentionally promoted 17 M5 flags, 8 M6 flags, 1 M7 flag, and obj.dust from deferred to adapted, while adding corresponding ambient movement, justice, dynamic quest, corpse lifecycle, and dust runtime behavior.
  implication: Reverting ledger status data would contradict implemented behavior; tests must follow the completed milestone.

- timestamp: 2026-09-02T10:16:00+08:00
  checked: Mechanical status recomputation from ledger dispositions
  found: M5 computes verified 14/adapted 38/deferred 6/excluded 1; M6 computes verified 17/adapted 31/deferred 15/excluded 13/alias 4/source_noop 1; M7 computes verified 33/adapted 51/deferred 50/excluded 38/alias 10/source_noop 1. All equal declared summaries and totals.
  implication: Current ledger summaries are authoritative and internally consistent.

- timestamp: 2026-09-02T10:16:00+08:00
  checked: M6 behavior_flag_audit narrative
  found: Its prose says adapted 25/deferred 21, while both dispositions and structured by_status fields compute adapted 31/deferred 15.
  implication: The prose is a separate stale data defect and should be corrected to prevent contradictory audit evidence.

- timestamp: 2026-09-02T10:24:00+08:00
  checked: Four focused regression tests in complete temporary fixture
  found: All three `static_catalogs_and` tests and `shared_behavior_ledger_covers_every_dynamic_obj_item` pass.
  implication: The original failures are directly resolved.

- timestamp: 2026-09-02T10:27:00+08:00
  checked: `cargo test --all-targets --all-features --locked` in complete temporary fixture
  found: 167 library tests and 21 importer tests pass; all other targets pass with zero failures.
  implication: No adjacent regression is observed across the complete test suite.

- timestamp: 2026-09-02T10:28:00+08:00
  checked: `cargo clippy --all-targets --all-features --locked -- -D warnings`
  found: Clippy completes successfully with zero warnings.
  implication: The patch satisfies the strict CI lint gate.

- timestamp: 2026-09-02T10:25:00+08:00
  checked: `cargo run --quiet --locked --bin es2-audit -- --output target\\m9-release-metadata-debug`
  found: M9 acceptance passes for 552 source rooms, 556 runtime locations, and 220 task rows.
  implication: Migration release metadata remains valid.

- timestamp: 2026-09-02T10:26:00+08:00
  checked: Direct test execution in primary working tree
  found: Compilation is blocked only because the intentionally dirty es2-utf8 checkout lacks quest source files; the same tree overlaid on a complete fixture passes.
  implication: Verification must be reported from the complete fixture until the user restores the submodule.

## Resolution

root_cause: Commit 6f60acb updated M8 implementation and ledger dispositions/summaries but omitted the matching M5/M6/M7 assertions and shared-ledger allowed-status list; its M6 narrative used an intermediate, incorrect count.
fix: Updated M5/M6/M7 exact status counts, admitted the implemented adapted status while checking every shared item status count against its summary, and corrected the M6 audit narrative.
verification: Focused regressions pass; full all-target/all-feature test suite passes (167 lib + 21 importer); strict Clippy passes; cargo fmt and git diff --check pass; M9 audit passes.
files_changed: [src/content.rs, src/items.rs, migration/overrides/m6-npcs.json]
