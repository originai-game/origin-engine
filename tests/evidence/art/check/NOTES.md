# A6 `art/check`: `og style check`, the style measured

Stream A6 of docs/art.md (§2.5, the A6 row of §3), branch `art/check`
from `4a178bdd`, with origin/main fast-forwarded to `a6feb7cc` (A5) before
the skill edit. Tooling only: nothing in the cook, the runtime or the
frame changes, so no reference frame moves and no metal frame needs
recording.

## What landed

| Commit | What |
|---|---|
| `9cb2763f` | og-cli `style_check.rs` (new): `og style check [project] [--title|--play|--results <commands>] [--size WxH] [--no-frames] [--critic] [--json]`; the `style` line of `og doctor --project`; `og build --publish`'s warnings; unit tests and two CLI tests |
| `659086a0` | og-build §8 describes the real command (the "coming" clause dropped); og-game §2 names project.ogt's `screens`; CHANGELOG `## Unreleased` line; `changelog.json` `unreleased.added` entry `og style check` |
| this commit | these notes, `frames-lavapipe.jpg` |

## How it measures

The project is cooked (`og cook`'s own call), and every asset is
measured from `cooked/`. The check decodes the blobs and does not read
the cook's notes or `.conform.ogt`. The contract is the cooked look named
by project.ogt's `look` line, which the cook has already flattened. Its
`ArtPalette.colors` is read from the cooked palette blob.

- **Sprites.** A sprite is any colour texture of a `profile "2d"`
  project, or one cooked as pixel art. The check measures:
  - distinct opaque colours;
  - the palette hit rate: the share of opaque texels within ΔE00 2 of a
    palette colour. CIEDE2000 is og compare's (`compare.rs`), checked
    against Sharma's data there;
  - the pixel grid, per axis. A period k holds when at most 25 % of the
    positions off its grid carry a colour boundary. At least three
    on-grid positions must carry one, two of them one logical pixel
    apart. The half pixels are the off-grid positions that carry a
    boundary;
  - the outline width: the rings of one dark colour (within ΔE00 15 of
    `ArtLine.color`) in from clear texels, per tile for a tile set, where
    a cell's edge is not clear.

  The project's grid is its sprites' most common k. Each sprite must sit
  on that grid with no half pixels.
- **Fixed layouts.** These are textures that a 2D rig or a UI style names,
  found from their sources. Their line is not judged, because the cook
  draws none into them. A UI atlas's grid is not judged either, because
  the UI draws at its own scale.
- **Lossy textures** (ETC1S and UASTC): colours are measured but not
  judged.
- **3D textures**: size, nearest or linear, and mip levels, against
  `ArtTexels`.
- **Models.** The check measures:
  - triangles drawn: each primitive's LOD 0 times the nodes that draw it,
    against `propFaces`, or `characterFaces` for a model with a skin;
  - the flat-normal share: corners whose normal is within 1° of their
    face's normal. `normals flat` needs 99 %;
  - texel density: the area-weighted median, p10 and p90 in texels per
    metre, against `ArtTexels.density` within ×2;
  - the colours of untextured faces;
  - the density spread across all models.
- **Opt-outs.** These are the cook report's `off` entries (`conform off`,
  project.ogt's `conform { off }`, data textures). They are listed, and
  each asset is still measured and judged.
- **Frames.** The screens come from project.ogt's new `screens { title
  "..." play "..." results "..." }` block, or from the flags. A
  `screenshot` is added at a script's last step when it has none. Each is
  run with `og run --headless --no-check --commands` into
  `og-out/style/<screen>/` and measured:
  - colours, and the fewest colours that cover 95 % of the pixels;
  - the palette hit rate;
  - edge density, the share of hard one-pixel steps, and mean contrast;
  - the frame's grid.
- **`--critic`.** Prints a one-line prompt, the catalogue entry's three
  `checks` as written (the entry is found through the look's `extends`
  chain), and the frames' paths. It never prints the name, the slug or the
  look id; a test asserts this.
- **Exit status.** 0 when every asset meets the contract, 3 when one does
  not, and 1 when the cook or a capture fails. A project with no `look`
  line prints that there is no contract and exits 0.
- **Doctor and publish.**
  - `og doctor --project` adds one finding with code `style`: `info` when
    the assets meet the contract or the project has no look, `warning`
    when one does not. It measures `cooked/` as it is, without cooking,
    and takes the opt-outs from `cook-report.json`. A warning never makes
    doctor exit 1. Doctor's JSON `warnings` now counts only warnings, not
    `info` findings.
  - `og build --publish` prints the off-contract assets after its cook as
    `og build: warning: style: ...`, at most eight lines plus "and N
    more". It never refuses a build.

## Runs

These runs use copies of the templates with a `look` line added to
project.ogt, the branch's dev `og`, and a dev addon through
`OG_NATIVE_ADDON=/tmp/og-dev.node` (copied from
`target/debug/libog_host_node.so`). Frames were taken at 640×360 on
lavapipe and read. The montage is `frames-lavapipe.jpg`: top row, game-2d
play and the paused screen; bottom, game-3d at 2 s.

**game-2d with 16bit-pixel** (`look "looks/origin-game-2d.ogt"`, which
extends 16bit-pixel). The `screens` block is title `0.5:screenshot`, play
`3.5:screenshot`, results `1:key_down:Escape;1.05:key_up:Escape;2:screenshot`.
5 assets were measured, none off the contract, exit 0:

- courier: 16 colours, grid 1, line 1;
- coin: 4 colours, grid 1, line 1;
- city tiles: 19 colours, grid 1, line 1, measured per tile;
- keeper rig parts: 17 colours, grid 1, line not judged;
- the pixel UI atlas: 6 colours, line and grid not judged.

All are within maxColors 64, and the project grid is 1 texel per logical
pixel. One opt-out: `@styles/pixel/icons.png` (an SDF: data, not
conformed). There is no palette hit rate, because the preset sets no
`colors`. Frames:

- title and play: 487 colours, 95 % of the pixels in 10, edges on 18 % of
  pixels, 43 % hard, contrast 0.15. They match because the template opens
  straight into play;
- results (paused): 877 colours, 95 % in 21, edges 2.8 %, 46 % hard,
  contrast 0.27.

In the frames, the courier and coins carry the cook's 1-px inside line,
the rooftops' crenellations are lined and the HUD is crisp. `--critic`
printed the three 16bit-pixel checks (parallax rates, 4 to 6 tones, edges
on the grid when the camera moves) and the three frame paths, and nothing
else.

Before the grid and line rules were settled, the first run judged three
of the five off. That run showed two false positives, now fixed:

- Tile sets have clear gutters (`spacing 2`), which counted every tile
  edge as a silhouette. The line is now measured per cell.
- A share-of-boundaries grid test took 1-texel art for 2× art. The
  templates' sprites put 50 % to 77 % of their boundaries on a period of 2
  by chance. The test now counts off-grid positions that carry a
  boundary.

**game-3d with faceted-low-poly** (`look "looks/origin-game.ogt"`).
4 assets were measured, 1 off, exit 3:

- dropship: 96 faces against propFaces 200, 100 % flat, 15 baked colours;
- walker (skinned): 108 faces against characterFaces 1500, 100 % flat,
  1 colour;
- paper UI atlas: 31 colours against maxColors 32;
- **`textures/stone.png`: 512×512 against maxSize 64, linear against
  `filter nearest`, 10 mip levels against `mips none`.**

`textures/stone.png` is a real finding. A scene material uses it, not a
glTF model, so neither A2 nor A3 conforms it. The coordinator should
decide whether A3's texture pass should cover textures that scenes'
materials name.

The play frame at 2 s is the title card over the flat-shaded diorama:
2,738 colours, 95 % in 742, edges 3.4 %, 8.5 % hard, contrast 0.35.
`og doctor --project` reported `warning style: 1 of 4 asset(s) off the art
contract of looks/origin-game.ogt (textures/stone.png), 1 opted out of
conformance`. `og build --publish` printed the two `style:` warnings and
went on to stop at the missing web host
(`packages/engine/dist/web-publish/origin-engine.js`), which this orb
does not build under the stream rules.

## Tests

Expectations are derived by hand in each test, with asymmetric fixtures.

`style_check::tests` (og-cli bin, 7 passed):

- `an_on_palette_sprite_hits_and_one_three_delta_e_off_misses`: palette
  grey 128 and red. Grey 136 is ΔE00 2.95 off, worked out by hand (L*
  53.585 against 56.708, S_L 1.0583). The sprite has 5 grey, 3 red and a
  clear texel. On-palette gives 1.0; off gives 3/8 and fails `strict`;
  grey 129 (≈0.95) hits. Without `strict`, the rate is measured but not
  judged.
- `a_two_texel_sprite_with_a_half_pixel_column_is_found`: an 8×4 logical
  sprite drawn at 2×. Clean, it gives k = (2, 2) with no half pixels.
  With column 13 taking column 14's colour, it gives half_columns `[13]`,
  and 1× art gives (1, 1). Against a project grid of 2, the clean sprite
  holds and the half pixel fails; against 1, both fail. This test failed
  on the first detector, the boundary-share version.
- `a_300_face_prop_is_over_200_and_within_500`: 100 triangles drawn by 3
  nodes measure 300, not 100. They are over 200 and within 500, and a
  character is held to characterFaces. Tilting 30 of 100 triangles'
  normals by 10° gives flat share 0.7, which fails `normals flat`.
- `the_outline_width_is_the_rings_of_one_dark_colour`: 1-, 2- and 0-ring
  sprites. A tile with a hole, cut by a clear gutter, gives 1 per cell
  and 0 without cells.
- `the_critic_reads_the_checks_and_frames_and_never_the_style`: text and
  JSON, with no slug, name or `looks/`.
- `the_project_grid_is_the_sprites_most_common` and
  `a_script_gets_a_screenshot_at_its_last_step`.

`tests/cli.rs` (2 passed):

- `style_check_without_a_look_says_there_is_no_contract_and_exits_0`.
- `style_check_measures_cooked_sprites_and_lists_opt_outs`: a real cook
  of a strict two-colour 2D project. `on.png` gives hit 1.0 and is ok.
  `off.png`, with `conform off` and a quarter of its texels grey 136,
  gives hit 0.75, is not ok, is opted out and is listed. Exit 3.

Other checks:

- `cargo clippy -p og-cli --tests -- -D warnings`, `cargo fmt --check`:
  clean.
- `bun run validate` in packages/knowledge: valid.
- Not run, per the stream rules: scripts/check.sh, test lanes, og test,
  web-reference, wasm or release builds.

## Shared files touched

- `crates/og-cli/src/main.rs` (the `mod` line and the usage text),
  `style.rs` (the `check` subcommand), `doctor_project.rs` (the style
  finding, and `warnings` counting warnings), `bundle.rs` (the publish
  warnings), `tests/cli.rs` (two tests appended).
- `packages/knowledge/skills/og-build/SKILL.md` §8,
  `packages/knowledge/skills/og-game/SKILL.md` §2 (one table cell).
- `CHANGELOG.md`, `packages/engine/changelog.json` (unreleased).
- og-cli reads `packages/knowledge/looks/catalogue.json` through
  `include_str!`, so the catalogue's `checks` are compiled into `og`.

## For the coordinator

- project.ogt gains `screens { title play results }`, which only `og style
  check` reads. The cook and the runtime ignore unknown blocks (both runs
  above cooked and ran with it).
- `og doctor --project --json` counts `warnings` differently: `info`
  findings are no longer warnings.
- While writing the results script, `1:key:Escape` ran without an error
  but pressed nothing. The command channel's action is
  `key_down`/`key_up`. I did not check whether `og run` should refuse an
  unknown action; it may be worth a look.
- `textures/stone.png` in templates/game is outside conformance (above).
