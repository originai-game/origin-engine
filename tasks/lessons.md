# Lessons

## Shared timing does not merge authored languages

A shared easing type does not make CSS keywords valid GSAP names. Timeline
text accepts GSAP names and explicit parameterized functions; stylesheet
text accepts CSS keywords and functions. Test removed names at the timeline
parser boundary, including quoted names, while proving they still work in CSS.

## Creation checks must precede prepack

- After changing creation choices or bundled assets, exercise creation before
  prepack can refresh ignored copies. Keep a stale-copy regression test, and run
  the full-lane consumers that create games: no-renderer and the skeleton fixture.
  A quick pass alone does not verify those consumers.

## API alignment

- Rename a query family consistently: `castRay` and `castRayAll`, even
  when the all-hit extension is engine-owned. Document the semantic
  difference without keeping a conflicting verb in the same family.

## UI rebuild

- There are no existing games to migrate in this rebuild. Refuse obsolete save envelope versions with both versions in the error; do not add legacy extraction or conversion.
- Land named stores, registration and presentation checkpoints before retained scheduling. Document mounts, models, actions and snippets target the new parser IR, never an adapter for the deleted screen dialect.
- A performance claim needs executed-work counters as well as elapsed time. An idle retained UI evaluates no bindings and traverses no tree; keyed scrolling must include the end of the collection and return with key-held focus.

## 2026-11-07 — Language compatibility needs an upstream conformance gate

The owner rejected a bounded Yarn subset. A named language integration must
cover the upstream grammar and semantics, not only the examples initially
requested. Pin the upstream revision, inventory every fixture and expected
result, and report every failure. Do not rename a profile as complete until
the full corpus passes on both hosts, including checkpoint replay.

## 2026-11-07 — Pin a released language and reuse text range units

"Current Yarn 3" means the latest released version, v3.2.2, not development
main. Import the corpus from that tag and document later differences rather
than silently adopting them. Check the consumer's range representation before
proposing a span ABI: og-ui uses Unicode scalar indices. Convert upstream
UTF-16 expectations in tests and include an emoji before a span to catch a
unit mismatch.

## UI documents: familiar syntax includes familiar semantics

When adopting web markup, IDs must be unique across the document and HTML
names take precedence where their meaning matches (`img`, `step`, `disabled`).
Do not carry sibling-local engine IDs into CSS selectors. A translated UI
also includes interpolated units and counts: use named ICU parameters, not
translated labels surrounding English string concatenation. Validate these
contracts at cook time and test the rejected forms.

## Reference equality is not visual review

Inspect each new or changed reference at readable resolution before committing.
Use populated screen fixtures, and label placeholder coverage separately. When
a contact sheet suggests a defect, inspect the full-size frame and its draw
commands before diagnosing it: the save/load Back label was present in both the
capture and reference, despite appearing absent in the scaled contact sheet.

Visual regression fixtures must reproduce the dimensions that exposed the bug.
A square-cell sheet cannot verify a short-side constraint: render tall, wide,
full-bleed and small-icon slots at their real aspect ratios, and keep labels
outside the slots so they do not shift the mark being reviewed.

## Keep active implementation running across notifications

A CI notification does not end an unfinished implementation request. Handle the
notification, then continue the active work without requiring another prompt.
Protect verified prerequisites in local logical commits; do not confuse their
tests with the requested integrated rendering proof or land unfinished adapters.

## 2026-11-08 — Never `git stash` in a lane

Every worktree shares one stash (`refs/stash` lives in the common git
dir). `git stash` on a clean lane saves nothing, and the `git stash pop`
after it applies another lane's entry. To test a base revision, commit or
check out a detached revision instead; never stash.

## 2026-11-09 — Both generators after an IDL or schema change

`og api gen --wiki` writes only the wiki and guides; the `.d.ts` and
`api.schema.json` come from `og api gen`. Rebuild `og` first
(`cargo build -p og-cli`), run both, and check every crate that builds an
og-ui struct literal (`cargo check --workspace --all-targets`), not og-ui
alone: og-runtime lists `Element`'s fields exhaustively.

## 2026-11-09 — Never act on a lane `take` refused

`scripts/lane.sh take lane-13 …; cd lane-13 && git reset --hard …` ran the
reset after `take` refused because another stream (C12) had taken the
lane seconds before: its uncommitted edits to four tracked files were
lost. Chain a take with `&&` and check the printed path, or better, land
from your own lane: `scripts/land.sh` needs the worktree clean only while
it runs, so stop editing there until it returns instead of borrowing a
second lane.

- 2026-11-09 (P1b, lane-11): never `git stash` / `git stash pop` in a lane. The stash list is shared by every worktree, so a stash with nothing to save makes `pop` apply another lane's entry. With untracked work, rebase with `git rebase origin/main` directly, or commit first.

## 2026-11-10 — Squash against the merge base, never against a fetched main

`git reset --soft origin/main` after a fetch keeps your tree but moves HEAD
to the new main: if main moved, the next commit stages a revert of every
change that landed meanwhile. Squash with
`git reset --soft "$(git merge-base HEAD origin/main)"`, then rebase.

## 2026-11-10 — A state sheet must capture offscreen states

A scrollable page is not complete visual evidence when only its first
viewport is captured. UI sheet and pack review must capture every scroll
page, including the desktop 150% and phone cells. A CSS test pack must
also distinguish each button variant through its class selector: matching
states alone do not prove that primary, secondary, quiet, ghost and danger
variants reach the cascade.

The same rule applies to semantic tones: neutral, info, success, warning,
danger and accent badges need different fills and inks from their own
selectors, or a broken tone selector disappears in a uniform sample.

A locale label on a review cell is not proof of localization. Load the
sheet's translated string tables before its document, install the locale's
font pack, and inspect translated glyphs and RTL geometry in the output.
English fallback under a Chinese or Arabic cell label is missing coverage.
