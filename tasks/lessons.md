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
