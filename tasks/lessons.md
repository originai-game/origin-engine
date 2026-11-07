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
