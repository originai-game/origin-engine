# Lessons

## 2026-10-24 — Development-key capability contract

The owner defines the platform development key as full-featured. When an
engine acceptance flow fails with that key, verify and fix the development
credential contract rather than narrowing it to a single-operation exception
or substituting an unrelated account key. Keep ordinary platform/channel
credentials distinct; full development capability does not imply upgrading
every platform key.

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
