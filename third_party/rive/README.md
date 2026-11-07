# Rive format and runtime semantics

The Rive import (`docs/ui.md` §11.2) reads `.riv` files at cook time and
never embeds the Rive runtime. What it takes from rive-runtime
(https://github.com/rive-app/rive-runtime, revision
`4f864426617abcac999a4d642f764b5623b9cb76`, MIT, `LICENSE` here):

| What | Source | Where |
|---|---|---|
| The format's type keys, their parents, and property keys with field types and defaults | `include/rive/generated/**/*_base.hpp`, `core_registry.hpp` | `crates/og-cook/src/cook/riv/defs.rs`, written by `scripts/gen-rive-defs.py` |
| The binary layout: fingerprint, version, table of contents, objects | `runtime_header.hpp`, `core/binary_reader.cpp` and https://rive.app/docs/runtimes/advanced-topic/format | `riv/read.rs` |
| Which objects take an artboard id (`claimsArtboardSlot`, `readNullObject`) | `file.cpp`, `importers/*` | `riv/convert.rs` |
| Path construction, rounded corners, the cubic vertices and the parametric shapes | `shapes/path.cpp`, `shapes/cubic_*_vertex.cpp`, `rectangle.cpp`, `ellipse.cpp`, `triangle.cpp`, `polygon.cpp`, `star.cpp` | `crates/og-ui/src/rive/player.rs` (`outline`, `build`, `ideal`) |
| Keyframe sampling, the cubic ease solver and the value interpolator, colour lerp | `animation/keyed_property.cpp`, `cubic_interpolator_solver.cpp`, `cubic_value_interpolator.cpp`, `shapes/paint/color.cpp` | `player.rs` (`sample`, `solve`, `sample_color`) |

The pinned major is 7. A new revision is a deliberate update of the
importer: rerun the generator against it and update this file.
