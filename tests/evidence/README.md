Milestone evidence lives here as files (docs/phase-1.md §4): tests/evidence/M<n>/.

## Naming

- One directory per milestone: `tests/evidence/M0/` … `tests/evidence/M7/`
  (the milestone numbers of docs/phase-1.md §2).
- One file per piece of evidence: `tests/evidence/M<n>/<slug>.<png|json|md>`.
  The slug is lowercase words joined by `-` and says what the file proves,
  not when it was made (`spike-frame-native.png`, `world-hash-3600-ticks.json`,
  `size-gate.json`). Re-capturing replaces the file under the same slug; Git
  keeps the history.
- `png` for frames and diffs, `json` for machine-readable results (hashes,
  timings, size reports), `md` for a written description of a run (what the
  frame shows, what was read in a dump).
- Every milestone directory has a `NOTES.md` with one line per file: what it
  is, the exact command or procedure that produced it, the revision and the
  date. A file without its line is not evidence.

## Filing a file

```
scripts/evidence.sh M0 spike-frame-native target/evidence/gfx/spike.actual.png \
  "OG_CAPTURE=1 cargo test -p og-tests gfx_spike on M4 Pro, Metal"
```

copies the file into `tests/evidence/M0/spike-frame-native.png` and appends
(or replaces) its line in `tests/evidence/M0/NOTES.md` with the revision
(`+dirty` when the tree had uncommitted changes outside `tests/evidence/`),
the UTC date and the file's sha256 prefix.

Scratch output that is not evidence (gate summaries, gfx diffs from a failing
run, perf reports) goes to `target/evidence/` (`$OG_EVIDENCE_DIR`), which CI
uploads as an artifact and Git ignores.
