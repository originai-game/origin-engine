# Lessons

## 2026-10-24 — Development-key capability contract

The owner defines the platform development key as full-featured. When an
engine acceptance flow fails with that key, verify and fix the development
credential contract rather than narrowing it to a single-operation exception
or substituting an unrelated account key. Keep ordinary platform/channel
credentials distinct; full development capability does not imply upgrading
every platform key.

## 2026-10-25 — Regenerate copied assets before recording references

An ignored game-local atlas can survive a source-pack change and make a
reference recording irreproducible on a clean clone. Regenerate the game's
assets from the committed source before recording. Rumpus's frame suite now
compares its atlas and icons with the source pack and refuses both comparison
and recording when either copy is stale.

## 2026-10-25 — Measurement revision identity

`git merge --ff-only origin/main` succeeding does not mean local main equals
origin/main: local main can be ahead. Before measuring a coordinator's revision,
compare both full object IDs and pin the measurement worktree to the requested
commit. Preserve unrelated local commits and never identify them as remote head.

## 2026-10-25 — Internal flags are not public CLI contracts

Before calling an unknown-option exit a defect, inspect the public parser and
the internal caller. `og run --device` invokes device.ts's internal preflight
before building; users do not pass `--preflight` to og run. Preserve observed
exit codes while distinguishing invalid invocations from product failures.

## 2026-11-03 — A serial rerun does not fix gate contention

When a timeout disappears outside the normal lane, inspect shared fixtures
and concurrency across steps before changing its timeout. Per-process GPU
limits add together across test runners. Declare the shared adapter in the
gate scheduler, bound worker counts, and verify through the normal lane.

Do not edit a shell script while it is running. Bash can resume reading at
an old byte offset after a long loop, so an otherwise valid edit can break
the active invocation. Wait for the gate to exit before editing its script.

## 2026-11-04 — Land ready heads in batches

Stack the ready heads, regenerate once, and run quick once for the stack.
Run the targeted Metal checks together; return only a failing head to its
author. Heads arriving during the gate join the next stack. Record approved
references in the next stack instead of putting them behind more code heads.

## 2026-11-04 — Capture controls identify effective state

Extract diagnostic scenes from the named revision instead of reusing a
previously patched copy. Record effective AA, rendered-frame history and
selection state beside the screenshot; equal simulation ticks do not prove
equal temporal history. The site384 copy retained `antialias off`, making
its grain comparison against a TAA reference inconclusive.

Treat a checker finding's element attribution as a claim to verify when
the pixels disagree. Repeated `Text` IDs and identical strings can belong
to different elements; preserve the element index or full path. Compare
the drawn ink colour with the sampled ground before trusting a pixel vote
that can mistake a light background for dark text.
