# District street

A 240 × 160 m cook-time district with seeded heights, facade materials,
residential entrances and shopfronts. The street-facing ground storey has
recessed doors, framed shop windows and a projecting stone base course;
rear and party walls use the separate windowed rule. These are authored
fixture rules, not an engine town or style preset.

Readers: og-cook's district integration test checks that the entrances,
shop glazing and plinth survive merging and every generated LOD. The L4
look capture drives this project's camera: street until tick 180, a
block overview until 240, then the courtyard. Capture all three at
1920×1080 and 390×844; a two-second street
walk fits before the camera switches. The target is Watch Dogs: Legion's
London streets. Later look heads address upper-storey relief,
brick repetition and kerb details on this same project. Courtyards now
use labelled brick paving, seeded plant-generator trees in stone beds
and slatted model-program benches. Their shared Poisson sites stay
behind the reserved street lots and carry authored colliders; no model
or surface is generated at runtime.

Textures are pinned by `og-assets.json`; install with the repository's
asset tool. The source image is generated at authoring time, and the
normal, roughness/metalness and occlusion maps come from `og materials
make`. No image generation or building generation runs in the game.

The camera reports street/block coverage and an above-road invariant
through `og.assert`. Its explicit 512 MiB streaming allowance admits the
whole look-capture area; this is not the texture-memory budget test.
Capture timing separately from screenshots and walk-frame readbacks.
Captures and measurements live outside Git.
