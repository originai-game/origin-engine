# Test fixtures

Small projects built for one feature each. A fixture is not a template and
ships to nobody: it exists so a test can drive one engine feature on the
real stack in a few seconds. Each has a README (what it exercises, which
tests read it), generated or `.ogt` assets only, one or two reference frames
at most and `og.assert` invariants for its behaviour. Budget: under 10 s
warm per fixture run.

`scripts/check.sh fixtures` runs `og test` in each fixture whose inputs
changed since it last passed (below); `--all` and wave-end run all of them.

| Fixture | Feature | Tests that read it | Inputs (crates and packages it exercises) | Warm |
|---|---|---|---|---|
| [destruction-yard](destruction-yard/README.md) | destruction: fracture in the cook, breaking on impulses, dust, the cross-host world hash | its `tests/yard.og.ts`; `packages/engine/tests/destruction.test.ts`; `crates/og-runtime/tests/destruction.rs` | og-destruct, og-physics, og-cook, og-runtime, og-render, packages/engine | 3.5–6 s + 3.7 s cook |
| [vehicle-terrain](vehicle-terrain/README.md) | og.Vehicle on terrain, HUD, eased camera, particles, decals, engine audio | its `tests/drive.og.ts`; `scripts/check.sh web-reference` | og-physics, og-render, og-ui, og-audio, og-cook, og-runtime, packages/engine, templates/styles/scifi | 4.4–5.2 s |

The inputs of a fixture, as the `fixtures` step keys its cache: the
fixture's own directory, the native closure of `og` (`og_crate_dirs -p
og-cli`), `packages/engine` and the style packs and looks it reads.
