---
name: og-taste
description: Art direction for a game on Origin Engine — choosing a style from the premise (genre, mood, era, audience) out of a 37-style catalogue and binding it as a look (post stack, stylisation, material defaults, the UI style, and the art contract every asset is made to) with a palette and a type ramp; the look's mechanism (og.Stylise, og.MaterialStyle, extends, applying a look at a tick) and the art contract's fields; every asset request compiled from the look by the art brief (artBrief); composition and hierarchy on the play surface, colour, typography, motion, feedback and consistency; atmosphere (fog, sky, time of day, weather, water); the anti-patterns that make a game look like a web form; and the checklist to run (with og run captures and dump_ui) before calling a game done. Use when starting a game's look, building or reviewing any screen or HUD, or when a game "works but looks generic".
---

A game is judged in its first second, before anyone reads a word. The
engine's UI is correct by default and characterless by default: its
default style is the design language of the engine and origin-agent, and
the game's UI never borrows the Origin platform's look. This
skill is how to give a game a look of its own and keep it consistent. The
mechanics are og-ui's (styles, motion and sounds are og-ui §6) and the amounts of game feel
og-feel's; this is the taste.

## Start from the choices

Read the `create` block in `project.ogt` first. Its `look` is the chosen
catalogue slug; `mood` is the requested tone, and `ui` is an independent
style choice. An explicit UI choice wins over the look preset's `ui`
line. Creation writes `assets/looks/<id>.ogt` with `extends`, `mood` and
`ui`, and names it in `project.ogt`. The cook resolves the built-in pack.
Customize that look for this game; use `og style add` to edit a local pack.

Keep `view`, `platform` and `input` in the art brief: a phone portrait HUD
and a desktop first-person HUD need different compositions. Inspect both
the controls and the safe area in a captured frame. The mood configures the
look's mood layer. Music and language choices need their own assets.

## What works

## 1. Choose a style from the premise

Write the premise in one line, then answer four questions:

| Question | Pulls toward |
|---|---|
| **Genre**: what does the player do most? | action and arcade → bold, high contrast, fast motion; puzzle and idle → soft, readable, gentle motion; strategy and RPG → dense, ornamented, calm |
| **Mood**: how should a session feel? | tense → dark ground, one hot accent; cosy → warm light ground, rounded; eerie → desaturated, thin type |
| **Era or world**: when and where is it set? | 8- and 16-bit nostalgia → pixel; myth, castles, quests → fantasy; space, hacking, mechs → sci-fi; city nights, racing, rhythm → cyber; storybooks, notebooks, mysteries → storybook |
| **Audience**: who plays, on what? | children and casual phone players → cartoon (big targets, bright, bouncy); core desktop players → any, but denser |

Then choose the **look** from the style catalogue (§12): match the four
answers against each style's `premise` (genre first, then mood, then era
and audience), shortlist two, and take the one whose three `checks` the
game can hold on every frame. Some premises and where they land:

| Premise | Look (`extends`) | Why |
|---|---|---|
| a brisk 2D platformer, 16-bit nostalgia | `16bit-pixel` | energetic and bright: a 224-line grid, eight tones, a Bayer dither |
| a cosy farming or life sim seen from above | `topdown-tile-pixel` | cosy and curious: a tile-friendly grid, flat crisp tiles |
| a calm city or diorama builder | `miniature-diorama` | tilt-shift focus, a tabletop model |
| an exploration or flight game, serene | `faceted-low-poly` | flat facets, a soft vignette, AgX light |
| a fighting or character-action game | `anime-cel-3d` | toon bands, cool-tinted shadows, outlines that thin with distance |
| survival horror, lonely and uncanny | `low-resolution-3d` | early-console 3D: snapped vertices, affine textures, fog |
| an arcade shooter, rhythm or racing game | `neon-vector` | bright outlines that bloom on a dark scene; a large ground plane keeps its outline only where it ends, and with `outlineDistance` a line grows at most four times its width near the camera |
| a narrative puzzle, gentle and sad | `watercolor-wash` | translucent washes granulated into the paper, pigment pooled in thin rims where they meet |

`og new <dir> --premise "<one line>"` does this match
for a new game and records the slug in the `create` block (`--look <slug>`
names it instead). Author `assets/looks/<game>.ogt` extending that preset
(§12), add `look
"looks/<game>.ogt"` to project.ogt (the cook reads the style's art
contract from it) and leave `GameDesc.look` out, or name the same file.
The entry also names the palette rules, the asset specs and the UI style
its preset carries (the look's `ui` line, §12), so a game that extends a
preset is dressed by it from the first frame. The game's UI follows the
game's art, never the Origin platform's: without a style it draws in the
engine's default style, neutral by design. The built-in
styles (`og style list`) are starting points, not the
only looks:

| Style | Premises it fits | Signature | Start look (§12) |
|---|---|---|---|
| `pixel` | platformers, roguelikes, retro and arcade shooters, jam games | square frames, 2 px bevels, hard shadows, a bitmap face | `16bit-pixel` |
| `cartoon` | casual, match-3, party, kids, idle | thick outlines, cream panels, squash on press, pops | `bold-comic-illustration` |
| `fantasy` | RPGs, card battlers, quest and dungeon games | parchment, gold corners and flourishes, a serif display | `illuminated-manuscript` |
| `scifi` | space, strategy, simulation, mech | flat near-black glass, 1 px hairlines, one ice-blue accent, a thin sans, mono telemetry | `cinematic-realism` |
| `storybook` | narrative, visual novels, puzzle, cosy, mystery, word games | soft paper sheets, ink lines, ornaments, floaty motion | `pencil-hatching` |
| `cyber` | racing, rhythm, cyberpunk, hacking | 45° cut frames on black, acid yellow with signal red and cyan, bold condensed type, dense data, snappy motion | `neon-vector` |

```text
ui "fantasy"                     # in the game's look: a built-in style by slug
ui "styles/harbour/style.ogt"    # or the game's own
```

Make it the game's own by changing the style file, never by passing colours
to individual calls: write the built-in into the game's
`assets/styles/<game>/` with `og style eject fantasy <game>` and name it in
the look (`ui "styles/<game>/style.ogt"`). Nothing in it needs rewriting: the ids in a
style file are read relative to the file first (`atlas "atlas.png"`, `body
"Newsreader-Medium.ttf"`), then from `assets/`. A style is complete: it
names every palette role and the four radii (`small`, `control`, `bar`,
`panel`), so nothing of it comes from anywhere else. Then change `palette`
(start with `primary`, `backdrop`, `panel`, `ink`), `radius`, `ramp` and
`fonts` (any OFL font beside the style file), and redraw the atlas if the
frames should change (`og style pack`, og-ui §6). A game that shows Chinese,
Japanese or Korean adds a `fallback` font to its style (or its string
table's `fonts` line, og-i18n). For a quick re-colour without a copy,
`og.ui.configure(world, { primaryColor })` derives the primary's hover,
pressed and soft tones and the ink on it. The art direction of the world
(sprites, backgrounds) and the style must agree: a pixel-art world in a
cyber UI reads as two games.

**The look is the contract.** Once chosen, the look is more than the
frame: its art blocks (§12) say what every asset of the game is (the
palette its colours sit on, one pixel size or texel density, the line, the
shape language, the motion step), and the cook makes every asset meet them
(og-build §8). So every asset request goes through the **art brief**, never
a prompt written by hand: `artBrief(look, kind, subject)` compiles the
game's look and its catalogue entry into the request for one asset, the
prompt (the style's direction, the palette as named colours with hex, the
sizes, the line, the shape language, what to avoid), the reference images
(the style's preview, then the game's accepted assets) and the generator's
parameters (transparent and size for an image; the face budget and texture
size for a model). In a game it is the engine package's
`knowledge/scripts/brief.ts`:

```sh
bun node_modules/@origingame/engine/knowledge/scripts/brief.ts character "the courier, running, side-on" --project . \
  --reference assets/sprites/keeper.png
```

The kinds are `sprite`, `tileset`, `portrait`, `backdrop`, `prop`,
`character`, `texture` and `icon`; a prop or character of a 3D game is a
model, of a 2D game a sprite. Hand the generator the brief's prompt,
references and parameters as they are, adding only what the subject needs
(a pose, a material, a mood), never a palette, a size or a style of your
own: asking for one asset never redesigns the style. An accepted asset
goes back in as a `--reference` of the next request, so the game's own
assets hold the style as much as the preview does. When an asset comes
back off the contract (a colour off the palette, a soft edge on pixel art,
too many triangles), fix the request or the asset; do not loosen the look
to fit it.

## 2. Composition and hierarchy

- **What the eye reads first**: decide the one thing per screen (the score on
  results, Play on the title, the health on a HUD) and make it the biggest,
  the brightest or the only thing in the primary colour. If two things compete,
  shrink one.
- **The play surface is the hero**: a HUD sits in the corners and edges
  (anchors `topLeft`, `topRight`, `bottom`), never across the middle. Keep
  the centre third free during play.
- **HUD weight**: at most three HUD groups visible at once; each is a HUD box
  (`frame: "hud"`) with an icon and a number, not a panel with labels. Move rarely needed
  information behind pause.
- **Safe area**: everything interactive inside `og.ui.frameViewport().safeArea`
  plus the density gutter; anchors do this for you, absolute positions do not.
- **Phone vs desktop**: on `phonePortrait` the thumb zone is the bottom third:
  primary actions go there, touch controls at least 44 CSS px apart; titles
  shrink one step (`xl2` instead of `xl3`). On desktop, menus sit
  centred or left, never full-width.
- **Grouping**: spacing does the grouping. Related things are `space.sm`
  apart, groups `space.lg` or more. Do not draw a box around everything.

## 3. Colour

- **Limited palette**: a ground (`backdrop`), one or two surfaces (`panel`,
  `panelRaised`), one ink (`ink`) with a muted ink (`inkMuted`), **one
  primary**, and the status colours. That is the whole palette; a style
  declares these and the roles derived from them.
- **One primary**, used for the one primary action per screen, earned stars,
  the player's own row and focus. If everything is primary, nothing is.
- **Value before hue**: check the screen in greyscale. Text, the primary
  button and the player character must separate from their ground by value
  alone (body text at least 4.5:1; the packs' test enforces it for theirs:
  every label ink, every button label in every state, a text field's
  placeholder, which draws in `inkFaint` on the field's frame, and a
  primary badge, `highlight` on `primarySoft` over a panel (the results
  screen's "New best"), and a status badge in each of `success`, `warn`,
  `danger` and `info` (the tone's own ink on its 16 % tint), so a copied
  style whose `inkFaint`, `control`, `highlight`, `primarySoft` or a status
  colour moves must keep those pairs at 4.5:1 too. The status colours are
  also the meters', toasts' and status dots' colours: on a cream panel they
  are deep (the cartoon style's warn is `#7b5102`, not a bright amber that
  read 1.63:1), on a dark one light (a red under a dark style's panel is a
  salmon such as `#ff8080`). Make them for the panels; a badge placed on
  the other side of the scale (a light style's badge straight on its dark
  stage) moves its own ink towards white or black until it reads there,
  and the neutral badge does the same on its `fill`, so neither needs a
  second palette. A game's own style is held to the same bars: the
  engine's style-pack contrast checks measure each ink against its ground.
  Fix contrast through value before changing hue: deepen an orange fill
  under a cream label or a plum fill under a light placeholder.
  Also read the primary and secondary labels from a rendered frame,
  where thin strokes may never reach their colour: a pixel font's one-pixel
  strokes fell between pixels and took half their ink, so a pixel face is
  imported with `pixel <grid>` (its pixels to the em) and drawn at whole
  multiples of its grid, every font pixel whole screen pixels; the pixel
  style sets Fusion Pixel 12 at 12, 24, 36, 48 and 72 px, and a copy of
  it with a DotGothic16 face still needs `textWeight body 0.5`).
- **Read the drawn contrast**: `og ui check` measures every live text's
  contrast on the frame's pixels and holds it to nothing. As a rule of
  thumb, body text at 4.5:1 or more reads for everyone (WCAG AA), large or
  display text and HUD readouts over a UI ground at 3:1 or more, and 7:1
  in the high-contrast setting; under 3:1 most players strain. Text in a
  deliberately dimmed ink (a placeholder, a disabled control) and text
  over the game scene are not measured: judge them in the frame. A low
  number on a CJK label at body size usually wants a heavier weight or a
  larger step, not a new hue (og-build §5).
- Saturated colour on large areas tires the eye: keep saturation for small
  things (the primary, icons, effects) and keep grounds calmer.
- Status colours mean status: never use `danger` red as decoration.

## 4. Typography

- **Display vs body**: a display face for titles, the score and big numbers
  (characterful: serif, rounded, italic, pixel), a body face for everything
  read (plain, open, legible at 13–15 px). Never the display face for
  paragraphs.
- **Numerals**: scores, timers, prices and stats in the numeric role
  (`font: "mono"`), so digits do not jitter as they count; big scores in the
  display face. Count them with `og.ui.counter`, never by setting text.
- **A ramp, not sizes**: use the style's steps (`xl3`, `xl`, `body`,
  `meta`); three or four steps per screen at most. Each step carries its
  tracking (display sizes tighten; a style's `ramp` line
  takes it as its fourth number after the CJK leading), and it is applied
  to Latin text only.
- Headings get `heading: true` so the style's flourish frames them.
- **Sizes that read**: `og ui check` reports every text's size (CSS px)
  and how far it grows at 150 % and 200 % text. Usually: 12 px or more on
  a phone, about 18 px at 1080p on desktop for anything read in play, a
  dense HUD's readouts (strategy, MMO, simulation; `dense: true` in
  og.ui.props) down to 12 px at 1080p; and every text growing with the
  player's text scale (a text that reads 100 % in a 200 % cell has a fixed
  size). Controls a thumb presses want 44×44 CSS px on touch (its
  `touch-target` measure); a link inline in a sentence is the exception.

## 5. Motion

- Every change of state moves; nothing pops in or out without a transition
  unless reduced motion is on.
- **Durations 120 to 300 ms** for UI (the style's `durations`), up to 700 ms
  for a screen transition, longer only for rewards. **Never linear**: ease
  out on arrival, ease in on exit.
- **Anticipation and follow-through**: a press squashes before the action
  (`press` in the style), a reward overshoots and settles (`pop`, `motion.pop`).
- **Stagger**: a panel's controls arrive one after another (`motion.stagger`
  30–60 ms), not all at once.
- **Screens change through a transition** (`og.ui.screenTransition`): fade
  for calm games, wipe or slide for brisk ones, iris for arcade and retro,
  dissolve for dreamlike ones. One kind per game.
- Respect `reducedMotion` (the settings template passes it): everything
  becomes a fade; do not add motion that ignores it.

## 6. Feedback

Every input is answered **on the same tick** (og-feel §1) by a visual and a
sound: hover (the style's lift or highlight), press (squash, the `press`
sound), success (a toast, a burst, `success`), refusal (a disabled button
shakes and plays `error` by itself; a wrong answer or a locked door shakes
its element with `og.ui.shake` and the `error` sound; never silently ignore
a press, and never shake the camera for it). Hand the UI its sounds with
`og.ui.configure(world, { sounds: { hover, press, open, close, success, error } })`
or the style's `sound` lines. Results land with a count-up, stars with a
`og.ui.burst`, rewards with a pop.

## 7. Consistency

- **One style per game.** Every screen, dialog and HUD uses it.
- **One radius scale**: the style's `radius`; never pass radii by hand.
- **One shadow direction**: the style's shadows all fall the same way (down);
  in-world shadows should agree.
- **One icon set**: the style's `icons`; do not mix icon styles.
- **One transition kind** for screen changes, one entrance for dialogs.

## 8. Atmosphere

Fog, sky, time of day and weather set the mood of a 3D (and 2D) scene more
cheaply than any model; use them on purpose, one look per area.

- **Fog is depth**: distance fog with `density` 0.005–0.02 per metre makes
  the far plane recede; keep the nearest 10 m clear (`start` 6–10). Height
  fog (`heightDensity` 0.03–0.1 below the ground's height) fills valleys and
  swamps. Match the fog's `color` to the sky's horizon or the scene floats.
  Those numbers are for a camera near the ground. Fog distances count from
  the camera, not from the player: with a high or top-down camera
  (shooters, strategy, isometric) the whole play area sits 20–60 m away,
  so distance fog starts beyond the far edge of the play area (`start` past
  its farthest visible point) or stays off. Depth there comes from value
  (the ground darker or lighter than the subjects, farther layers lower in
  contrast) and from height fog below the play plane (`height` under the
  ground or sea, so it fills what lies beneath), never from fog over the
  play area: the play area keeps full contrast.
- **One sky per area**: a clear noon (`og.Sky` zenith deep blue, horizon
  pale), an overcast grey (low contrast, `sunSize` 0), a warm dusk with
  long shadows (`og.DayNight` at 16.5–17.25 h: at the default `tilt` the
  sun is 20° to 10° up, its light turning warm from about 17 h). Later the
  sun's light on level ground sinks into the sky's and its shadows fade
  (17.5 h: under 7° up), and at 18 h it sets whatever the `tilt` (which
  only lowers the noon sun); 17.5–18.2 h is a glowing horizon without
  sun shadows. Relief shadows the play area sooner: a valley's western
  ridge puts the floor in shade by about 17 h, so check the hours on a
  frame of the real ground (turn `azimuth` to set the sun across the
  valley). A day-night cycle wants `speed` slow enough
  that a player notices it over minutes (0.02–0.05 game hours per second),
  never per encounter.
- **Weather is a beat, not wallpaper**: rain or snow for a level's mood or a
  story moment; `intensity` 0.5–1; dust for deserts; mist for mornings and
  graveyards. Keep the play area readable: if enemies vanish in the fog,
  lower the density.
- **Water reads by its edges**: foam where it meets rocks and posts
  (`foamWidth` 0.2–0.5), a shallow colour close to the bed's and a deep one
  dark and saturated; `waveStrength` 0.2–0.5 for ponds, 0.6–1 for seas.
- **2D**: a `Water2D` lip a shade lighter than the surface; metaball fluids
  in one bright colour with a lighter `rim`, drawn under the foreground.
- Prove it: captures at the area's time of day; at a ground-level camera
  the far third of the frame lower in contrast than the near third, and the
  HUD unchanged by the fog. At every camera, capture the play area with the
  fog on and off: its contrast is the same in both (within a few per cent),
  or the fog is over the play area and moves out (`start` farther,
  `density` lower).

## 9. Anti-patterns: "it looks like a web form"

Each of these is a defect, not a style choice:

- grey rounded rectangles on a grey ground (the token look left as is);
- the default sans everywhere, display text at body weight;
- a centred stack of equal buttons with no primary;
- no ornament, no texture, no shadow: flat boxes floating on the game;
- no motion: screens and dialogs pop in, numbers jump;
- no sound: buttons are silent;
- the HUD as a toolbar: a full-width bar of labelled boxes across the top;
- labels instead of icons on the HUD ("Health: 72/100" instead of a heart
  and a bar);
- every colour saturated, or every button in the primary colour;
- a settings-page layout (left-aligned form rows) for a title screen.

## 10. The checklist before a game is done

Run each item; each is checkable from a scripted run.

1. **The style is the one you chose**: `og.ui.theme(world).style` names it
   (empty is the default style; log it from a command, or read it in a
   playtest through a `facts` field). A game with a look of its own usually
   reads better with a UI style to match (§1); the default and the built-ins
   ship as they are too.
2. **Title, pause and results screens captured**: `og run --headless --commands
   "0.5:show:title;1.5:screenshot:title.png;…"` (with the game's own commands);
   look at each capture in greyscale: the one thing per screen reads first.
3. **One primary per screen**: in a `dump_ui` capture, at most one button per
   screen uses the primary variant (the template's first item, or the one
   you marked); every other is secondary or quiet.
4. **HUD in the corners**: on the gameplay capture, `dump_ui` shows every HUD
   element's rectangle outside the central third of the surface, inside the
   safe area.
5. **Touch targets**: run with the phone viewport (`og.ui.previewViewport`,
   `og run --size 1170x2532`) and read `og ui check`'s `touch-target`
   measures (44 × 44 CSS px is the usual size, §4) and its `safe-area`
   findings.
6. **Text legible**: read `og ui check`'s `text-size` and `contrast`
   measures against §3 and §4 and in the frames.
7. **Motion present**: captures 100 ms apart after `show:` differ (an
   entrance), and a score `dump_ui` text read twice during the results
   differs, then settles on the final value.
8. **Screen changes transition**: a capture halfway through a screen change
   shows the cover (`og.ui.screenTransition` answers `covering` or
   `revealing`).
9. **Sound on input**: after a scripted press, `dump_world` lists an
   `og.AudioSource` on the ui bus (bus 3).
10. **Reduced motion**: with `reducedMotion: true` the captures show fades
    only, and nothing is missing.
11. **CJK**: switch the language to Chinese in settings and capture: no
    missing-glyph boxes (the style's or the string table's fallback font covers it).
12. **None of section 9** is visible in any capture.
13. **The frames are read**: every capture above opened and described,
    and `og ui check .` run over the declared screens with each number it
    measured read against §2 to §4 in the frames (§11, og-build §5).

## 11. Numbers cannot judge a style

No engine command scores taste. `og ui check` measures what can be
measured on every declared screen (contrast on the drawn pixels, text
sizes, overlaps, cut and offscreen text, focus), `og test` checks what the
playtests assert, and the frames show the rest; you judge them against
§2 to §10 (og-build §5 has the loop). A measured number is evidence for a
frame, not a verdict: read the frames before changing anything, and fix
the cause in the style or the template, never per call.

## 12. Style catalogue: looks

A style is chosen once per game and then held by data, not by habit: the
**look** (`assets/looks/<game>.ogt`, an `.ogt` of `type look`) binds the
frame's whole treatment (tone mapping and exposure, the post stack, the
stylisation primitives, material defaults such as toon shading and
outlines, fog and sky, the UI style and the antialiasing) in one diffable
file. The engine ships the primitives and the presets; the taste is in
which one you pick and how you tune it.

**Pick from the catalogue.** `looks/catalogue.json` (this skill's file:
`skill {name: "og-taste", file: "looks/catalogue.json"}`)
describes 37 styles in eight groups (`category`, named by `categories`:
pixel art, retro 3D, hand-drawn and painted, ink and comic, toon and
stylised 3D, low-poly, toy and handmade 3D, flat and graphic, realistic).
For each: a one-line descriptor, three
checks a critic can judge on a captured frame, the premises it fits
(genre, mood, era, audience, as §1 asks them), the preset it maps to, the
UI style (`ui`), palette constraints, asset specs (logical resolution, sprite
scale, face budget, outline width, texture density) and an art-bible
direction. Match the premise's four answers against `premise`, shortlist
two, and choose by the checks you can actually hold for the whole game.

**A style is the assets, not a filter.** The look's render resources
treat the frame; what makes most styles is in the assets: facets, voxels,
a palette every texel sits on, one pixel size, a line drawn into every
sprite. `look.layers` says what a style needs: `frame` (the look's render
resources), `conform` (the assets meet the look's art contract),
`runtime` (sprite outline, palette swap, pixel-art rotation, camera
presets), `authored` (art no conformance makes). Make every asset inside
the style (the art bible, §1) and look at it in the game, never only at
the frame.

**Read `look.support` before promising a style.** `full`: every layer the
style needs but `authored` is in the engine. `partial`: what `missing`
names is not yet; until then the assets must hold the style by hand (the
art brief of §1 writes the palette, the sizes and the line into each
request; check each asset that comes back); say so to whoever asked for
it. `content`: the look lives in authored art (painted backdrops,
frame-by-frame animation, hand-placed pixels); the engine frames it with
the look, but the assets carry it, so budget for them. `preview` (`previews/<slug>.webp` in the engine
package's `knowledge/`) shows the style as a game.

**Read `look.profiles` before matching.** It names the project profiles a
style draws for: `["2d"]` for flipbooks, hand-placed pixels and paper
cut-outs, `["3d"]` for lit meshes, toon materials and voxels, both for a
post treatment that frames either. `create --premise` matches only among the
styles compatible with the selected view (2D for `side-2d` and `top-down-2d`,
3D otherwise);
`--look` names any style and wins.

**Read `look.views` and `look.mood`.** `views` names the views a style
suits (`side-2d`, `top-down-2d`, `isometric-2.5d`, `third-person-3d`,
`first-person-3d`, the creation choice's words), only those its profiles
draw: shortlist among the styles that suit the game's view. `mood` is the
mood the preset is drawn in, one of `bright`, `dark`, `warm`, `cold`,
`soft` and `high-contrast`. A game asks another with one line in its look,
`mood "dark"` (author this from the `create` block's mood): the cook lays the mood's
grading, light and, for a palette-bound style, its palette over the
preset, and your own fields still win (§13). The art brief reads it, so
every asset is requested lit and coloured for the mood; change the mood
before the assets are made, not after.

**Budget for a `content` style's art.** `look.authored` lists the art such
a style cannot get from conformance, each an art brief kind and what it
must be (painted backdrops, frame-by-frame flipbooks, hand-modelled clay).
`artBrief` writes that into every request of the kind (an `Authored`
line), and `authoredArt(look)` lists them, so request each one the game
needs through the brief and accept it only when it is that art.

**Write the look.** Start from the style's preset (this skill's file:
`skill {name: "og-taste", file: "looks/presets/<slug>.ogt"}`) and change
only what the game needs; your file wins field by field. A style whose `look.presetReady`
is false has no preset in this engine yet: leave out `extends` and set the
blocks yourself from the entry's `primitives`.

```text
type look
version 1
---
name "Harbour at dusk"
extends "anime-cel-3d"          # the preset's slug (looks/presets/<slug>.ogt)
ui "styles/harbour/style.ogt"   # the game's own UI style (the preset names a built-in)
RenderSettings {
  toneMapping agx
  exposure 1.1
}
MaterialStyle {
  shadowTint "#3b3566"
  shadowTintAmount 0.6
}
Fog {
  enabled true
  color "#9aa7c7"
  density 0.012
}
```

**The pixel grid** (`Stylise.pixelHeight`, `pixelUpscale`) scales the
frame up by a whole number, so pick the height for the output you ship
at. The height sets the scale: `pixelUpscale` when height × scale fits
the output's height, else (and for 0) the largest that does. The width
is the aspect's columns, at most the output's width over the scale, so
at most the last column gives way, never the scale: 240 lines at 1280 ×
720 are 426 columns at 3x (1278 × 720, a 1-pixel border each side), at
1920 × 1080 427 columns at 4x (1708 × 960, a 106/60-pixel border). A
height that divides the output fills it: 180 (4x), 240 (3x) or 360 (2x)
at 720p; 270 (4x) or 360 (3x) at 1080p. The `16bit-pixel` preset's 224
lines draw at 3x inside a 43/24-pixel border at 720p. Size the camera to
the grid: with 16-pixel tiles, 240 lines are 15 tiles tall.

The engine ships canvases and palettes a look names without the game
owning a copy: `Stylise.paintCanvas "@canvas/paper"` (also `linen`, a
woven canvas, and `watercolor`, cold-pressed paper) and `Stylise.palette
"@palettes/handheld4"` (also `lcd2`, `dusk16`, `bright32` and `1bit`). The
cook puts the ones a look names into the bundle; a palette or canvas of
the game's own is a path under `assets/` as before.

**Fit the pixel grid to the screens.** `Stylise.pixelHeight` sets the
whole-number scale from the output's height alone, the width follows the
aspect, and what the grid does not cover is a border in
`Camera.background` (the rule is `Stylise.pixelHeight`'s doc,
§13). So choose the height by the screens the game meets, not by the era
alone. The scale per output (✓ fills it, ≈ fills it but for one column, b
leaves a border):

| `pixelHeight` | 1280×720 | 1920×1080 | 2560×1440 | 3840×2160 | phone 1600×720 | phone 1688×780 | phone 1830×824 |
|---|---|---|---|---|---|---|---|
| 90 | 8 ✓ | 12 ✓ | 16 ✓ | 24 ✓ | 8 ✓ | 8 b | 9 b |
| 135 | 5 b | 8 ✓ | 10 b | 16 ✓ | 5 b | 5 b | 6 b |
| 144 | 5 ✓ | 7 b | 10 ✓ | 15 ✓ | 5 ✓ | 5 b | 5 b |
| 160 | 4 b | 6 b | 9 ≈ | 13 b | 4 b | 4 b | 5 b |
| 180 | 4 ✓ | 6 ✓ | 8 ✓ | 12 ✓ | 4 ✓ | 4 b | 4 b |
| 192 | 3 b | 5 b | 7 b | 11 b | 3 b | 4 b | 4 b |
| 216 | 3 b | 5 ✓ | 6 b | 10 ✓ | 3 b | 3 b | 3 b |
| 224 | 3 b | 4 b | 6 b | 9 b | 3 b | 3 b | 3 b |
| 240 | 3 ≈ | 4 b | 6 ≈ | 9 ≈ | 3 ≈ | 3 b | 3 b |
| 270 | 2 b | 4 ✓ | 5 b | 8 ✓ | 2 b | 2 b | 3 b |
| 360 | 2 ✓ | 3 ✓ | 4 ✓ | 6 ✓ | 2 ✓ | 2 b | 2 b |

- 180 is the height to reach for (90 or 360 for a coarser or finer grid):
  it fills every 16:9 size. 144 suits 720p and 1440p, 216 and 270 1080p
  and 4K; 240 fills 720p but leaves a 106/60-pixel border at 1080p.
- 224 (`16bit-pixel`) and 192 (`8bit-pixel`) leave a border everywhere:
  write `pixelHeight 180` (or 216) in the game's look, which wins over the
  preset field by field, unless the border is part of the look.
- A phone's output is its page's CSS size times the device pixel ratio,
  capped at 2 (`publish { dpr_cap 2 }` in project.ogt; the columns above
  are 800×360, 844×390 and 915×412 CSS pixels in landscape), so most
  phones leave a border whatever the height. Give it a colour of the
  palette (`Camera.background`), or letterbox the page to 16:9
  (`publish { fit letterbox aspect "16:9" }`).
- With 16-pixel tiles, 180 lines are 11¼ tiles tall: size the camera and
  the rooms to the grid, not the other way round.
- Prove it on frames: `og run . --headless --size 1280x720` and again at
  `1920x1080`, a screenshot each, and read the border.

With a moving 3D camera, lay the paper and the strokes on the scene
rather than over the screen: `Stylise.paintCanvasSpace surface` (grain
per metre by `paintCanvasWorldScale`), `hatchingSpace surface` and
`halftoneSpace surface` stick to the surfaces and wrap round forms, so
nothing slides over the world as the camera moves; `screen` (the
default) suits 2D, fixed cameras and a deliberate printed-sheet feel.
When things move (characters, vehicles, a turning prop), take
`hatchingSpace object` (and `halftoneSpace object`, `paintCanvasSpace
object`): the marks are laid in each object's own space, so a walking
figure carries its strokes instead of wading through them (terrain keeps
the world's). `hatchingDepth` (0 to 1) gathers the strokes along
silhouettes and creases as `inkDepth` inks them: the contour shading of
an inked page. Pixel sprites standing in a 3D set take
`Sprite.dropShadow`; on a deck or a step give them `dropShadowHeight`,
and over an og.Terrain their shadows follow the slope, and bend into a
hollow or over a ridge narrower than themselves, by themselves.

Depth bands (`Stylise.depthBands`, the layered-silhouette, black-figure
and luminous-layered presets) cut the frame by distance from the camera,
so they need a camera that looks *across* the world: a side-scroller, a
stage, a three-quarter view down a valley or a street, a high view over
a landscape, where near, middle and far ground lie at clearly different
depths. Set `depthBandsNear` where the nearest layer you want starts and
`depthBandsFar` where the farthest layer ends (the far colour begins one
band before it), measured along the view on a real frame. They do not
work top-down or in a steep overhead camera: the whole play area sits at
about one depth and falls in one band, so it goes one flat colour and
the characters vanish into it; use a tinted shadow side, fog or a
palette there instead, or the bands only for a high establishing shot
(a title, the results). `depthBandsFlatten` 1 gives flat cut-paper
layers; 0 keeps the lit scene and only tints each layer (its shading and
hues show through), so characters stay readable; 0.3–0.6 is a painted
middle ground.

Name it in the game (`og.game({ look: "looks/harbour.ogt", … })`) and it
is applied after the first scene spawns and before `start` runs (so
`start` may still change any field, and reads the look's UI style from
`og.render.look(world)?.style`); switch looks at a tick with
`og.load.look` and `og.render.applyLook` (a flashback, a photo mode, an
area's mood). A look's `MaterialStyle` reaches every lit material that
leaves the field at its default; a material that must stay as made (a
photographic hero prop in a toon set, a product shot, a gem) takes
`styled: false` in `og.render.standardMaterial` and is drawn exactly as
declared. Physical materials are safe under a toon look: lacquer
(`clearcoat`), glass and jelly (`transmission`), velvet (`sheen`) keep
their shading and take only the rim, the outline and the shadow tint, so
a toon look never switches them off; set `toon: true` on one yourself to
flatten it too. A look replaces each resource it names whole and leaves the
others alone, so an area look that names only `Fog` and `Sky` keeps the
game's grade. A copy of a preset under the preset's own name
(`assets/looks/<slug>.ogt` with `extends "<slug>"`) overrides it for every
look of the game that extends that slug. Colours may be written as sRGB
`"#rrggbb"`; the cook stores them linear. A look's top-level `antialias` takes `tier`,
`off`, `msaa4` or `taa` (FXAA is `GameDesc.render`'s, og-render §3), and an
`Environment` block sets an environment map (`map "env/dusk.ogt"`) as
part of the look. The mechanics (what each resource's fields do, presets,
the built-in canvases) are §13, and light and exposure og-render §3.

**The art contract: what every asset holds.** Beside the frame's
resources, a look carries seven art blocks that turn the entry's
`palette` and `assets` prose into numbers every asset is made to, 2D and
3D alike (§14 has the fields). Each preset already holds its
style's values; read them in `templates/looks/<slug>.ogt` (the comment
above them says where each number comes from) and override only what the
game decides differently.

| Block | Holds | Where it comes from in the entry |
|---|---|---|
| `ArtPalette` | `colors` (a palette asset), `strict`, `maxColors`, `rampSteps`, `accents` (palette indices kept for signals), `dither` | `palette`: the colour count, the ramps, the indexed palette of a handheld or 1-bit style |
| `ArtPixels` | `unit` (pixels per metre), `logicalHeight`, `spriteSizes`, `rotation`, `rotationSteps`, `snap` | `assets`: the logical resolution, sprite scale and tile size of pixel styles |
| `ArtTexels` | `density` (texels per metre), `maxSize`, `filter`, `mips` | `assets`: texture density, the small nearest-filtered textures of early 3D |
| `ArtShape` | `propFaces`, `characterFaces`, `normals`, `normalAngle`, `bevel`, `voxel`, `silhouette` | `assets`: the face budget, facets or soft forms, clay bevels, the voxel edge |
| `ArtLine` | `width`, `color`, `place`, `distanceFalloff` | `assets`: the outline width and where it lies |
| `ArtMotion` | `stepFps`, `maxFrames`, `squash` | animation on twos or threes, rubber-hose squash |
| `ArtCamera` | `projection`, `fixedAngle`, `pitch`, `yaw`, `fovMin`, `fovMax`, `pixelPerfect` | a style that rules perspective in or out, a fixed isometric or side-on view |

```text
extends "topdown-tile-pixel"
ArtPalette {
  colors "palettes/meadow.hex"   # the game's own palette: every asset sits on it
  accents 2 9                    # the player and danger; scenery stays off them
}
ArtPixels {
  spriteSizes 16                 # one tile size, no 24-pixel stragglers
}
```

Every field is no constraint at zero, so an override to `0` (or `false`,
or the empty list) releases a constraint the preset sets. project.ogt's
`look "looks/<game>.ogt"` names the look the cook reads the contract from;
author that line after creating the look. The art brief (§1) writes the contract's numbers into
every asset request (the palette as names and hex, the sizes, the line, the
face budget), the cook conforms each asset to them and lists what it
changed (og-build §8), and you check each asset against them in the game,
never only the frame.

**Expand the art bible.** Fill the catalogue's `artBibleTemplate` with the
chosen entry and the project's premise, camera and platform, and keep the
result with the game. The bible keeps the brief's template: under the
filled text, write the brief of each kind the game will need (the command
of §1 with its kind and a placeholder subject, and the parameters it
printed), so every later request starts from the same text and numbers.
Asking for one asset never redesigns the style; when the look changes,
run the briefs again and replace them in the bible. Then hold the game to the
style's three `checks` on real frames (`og run --headless --commands` with
screenshots of the title, play and results), as §10 holds the UI, and
make a playtest fact of `og.render.look(world)?.name` so a later change
cannot drop the look unnoticed. The platform's own wizard does the same
in its style step (`docs/platform-asks.md` in the engine repository): the
catalogue entry chosen from the premise, the look written, and the
entry's three checks handed to the critic that reads the frames.

| Slug | Group | Style | Support | UI |
|---|---|---|---|---|
| `expressionist-painting` | Hand-drawn and painted | Expressionist paint | content | `storybook` |
| `bold-comic-illustration` | Ink and comic | Bold comic illustration | full | `fantasy` |
| `watercolor-wash` | Hand-drawn and painted | Watercolour wash | full | `storybook` |
| `pixel-3d-hybrid` | Retro 3D | Pixel sprites in 3D sets | full | `fantasy` |
| `faceted-low-poly` | Low-poly, toy and handmade 3D | Faceted low poly | full | `storybook` |
| `sculpted-clay` | Low-poly, toy and handmade 3D | Sculpted clay | content | `cartoon` |
| `8bit-pixel` | Pixel art | Early console pixels | full | `pixel` |
| `voxel-worlds` | Low-poly, toy and handmade 3D | Voxel worlds | full | `pixel` |
| `illuminated-manuscript` | Hand-drawn and painted | Manuscript page | content | `storybook` |
| `rubber-hose-animation` | Hand-drawn and painted | Rubber-hose cartoon | content | `cartoon` |
| `pencil-hatching` | Ink and comic | Pencil and hatching | full | `storybook` |
| `gothic-contour` | Hand-drawn and painted | Gothic ornament | content | `fantasy` |
| `lush-painted-worlds` | Hand-drawn and painted | Lush opaque painting | content | `fantasy` |
| `decorative-storybook` | Hand-drawn and painted | Storybook ornament | content | `cartoon` |
| `classical-painting-collage` | Hand-drawn and painted | Old-master collage | content | `storybook` |
| `detailed-doodle` | Hand-drawn and painted | Fine-line doodle worlds | content | `storybook` |
| `black-figure-pottery` | Hand-drawn and painted | Black-figure pottery | content | `fantasy` |
| `heavy-ink-shadows` | Ink and comic | Heavy ink and shadow | full | `fantasy` |
| `monochrome-pixel` | Pixel art | One-ink pixels | full | `pixel` |
| `16bit-pixel` | Pixel art | Late console pixels | full | `pixel` |
| `detailed-pixel` | Pixel art | Dense illustrative pixels | full | `fantasy` |
| `luminous-pixel` | Pixel art | Glowing pixels on dark ground | full | `cyber` |
| `topdown-tile-pixel` | Pixel art | Top-down tile worlds | full | `fantasy` |
| `low-resolution-3d` | Retro 3D | Low-resolution 3D | full | `scifi` |
| `coarse-pixel` | Pixel art | Minimal low-count pixels | full | `pixel` |
| `anime-cel-3d` | Toon and stylised 3D | Anime cel-shaded 3D | full | `cyber` |
| `inked-comic-3d` | Toon and stylised 3D | Inked comic 3D | full | `scifi` |
| `paper-craft` | Low-poly, toy and handmade 3D | Paper craft | content | `storybook` |
| `miniature-diorama` | Low-poly, toy and handmade 3D | Miniature diorama | content | `cartoon` |
| `painterly-3d` | Toon and stylised 3D | Painterly 3D | content | `fantasy` |
| `cinematic-realism` | Realistic | Cinematic realism | full | `scifi` |
| `muted-cinematic-3d` | Realistic | Muted theatrical light | full | `scifi` |
| `layered-silhouette` | Flat and graphic | Layered silhouettes | full | `cyber` |
| `minimal-geometric` | Flat and graphic | Minimal geometry | full | `cyber` |
| `neon-vector` | Flat and graphic | Neon vector lines | full | `cyber` |
| `monochrome-dither-3d` | Retro 3D | Monochrome dithered 3D | full | `pixel` |
| `luminous-layered-painting` | Hand-drawn and painted | Luminous layered depths | content | `fantasy` |

**What looks like a defect but is the style.** Each entry's `intended`
says, in one sentence that never names the style, which traits of its
frames look like defects and are the style: hard one-pixel steps and
aliased edges in the pixel styles, dither in the dithered ones, broken
edges and uneven marks in the painted ones, crushed-looking blacks in
the ink and silhouette ones. Every critic reads it beside the three
`checks` (visual-acceptance §5 says how to hand both to a blind
critic), so no critic holds a
game for being in its own style, and every critic reads the same traits
the same way (visual-acceptance §5, §6).

Use the catalogue as a vocabulary, never as a list of games to copy: every
design, character and composition stays original (the platform's rule),
and the descriptors deliberately name no game or studio.

## 13. Looks: the mechanism

A **look** is an `.ogt` of `type look` under `assets/looks/` that sets the
whole treatment of the frame at once: the resources `RenderSettings`,
`PostEffects`, `Stylise`, `MaterialStyle`, `Fog`, `Sky` and `Environment`
(one block each, fields by name), the UI style (`ui`) and the `antialias`;
and, in the same file, the art contract the assets hold (below).
Choosing one for a premise is §1 and §12; this is the mechanism.

```text
type look
version 1
---
name "Harbour"
extends "anime-cel-3d"           # a built-in preset (templates/looks/<slug>.ogt) or another look's id
mood "cold"                      # bright, dark, warm, cold, soft or high-contrast; unset, the preset's own
ui "styles/harbour/style.ogt"     # the game's own UI style, or a built-in by slug (ui "fantasy")
Stylise {
  ink true
  inkWidth 1.5
}
MaterialStyle {
  shadowTint "#3b3566"           # sRGB written, stored linear
  shadowTintAmount 0.6
}
```

```ts
export default og.game({ look: "looks/harbour.ogt", start, systems });   // GameDesc.look: applied after the first scene, before start

const flashback = must(og.load.look(world, "looks/flashback.ogt"));      // at any tick
must(og.render.applyLook(world, flashback));
og.render.look(world)?.name;                                              // the look last applied (LookInfo)
```

- Applying a look replaces each resource it names **whole** (the fields it
  sets, the rest their defaults; `PostEffects.flash` and `shakeTrauma`
  keep theirs) and leaves the others alone, then configures the UI with
  its style. Write per-game tweaks in `start` after it, or in the look. A
  system that applies one declares those resources among its writes.
- `extends` flattens at cook time, field by field (a field written wins);
  `assets/looks/<slug>.ogt` of the game's own replaces the preset of that
  slug for every look that extends it. The cook refuses an unknown field,
  a value out of range and an asset not in the bundle, naming the line.
- `mood` asks the preset for a mood other than its own: the cook lays the
  mood's layer (`templates/looks/moods/`) over the preset before the
  looks above it. A number there moves the preset's value by its distance
  from the field's default (`dark`'s `exposure 0.72` takes 0.8 to 0.52), so
  the style keeps its own grading, moved; a bool, colour or palette
  replaces. A mood sets the grading, exposure, ambient light and, for
  `monochrome-pixel` and `monochrome-dither-3d`, the palette, never the art
  contract's sizes or lines. A flashback in another mood is a second look
  with `extends` the game's look and its own `mood` line.
- `og.load.lookInfo` (and `og.render.look`) answer `LookInfo`: `name`,
  `extends`, `mood`, `resources`, `style`, `antialias`. A look's `antialias` (tier,
  off, msaa4 or taa) is the renderer's when `GameDesc.render` names none.
- **`og.Stylise`** moves the frame away from the photographic, every
  primitive off at zero: the pixel grid (`pixelHeight`, `pixelUpscale`,
  `pixelSnap`), fewer tones or a fixed palette through a dither
  (`posterize`, `palette` from `og.load.palette` of a `.hex` or `.gpl`,
  `dither`, `ditherStrength`), `halftone`, `hatching`, painterly
  smoothing (`paintRadius`, `paintEdgeDarkening`, `paintBleed`,
  `paintCanvas`), the watercolour wash over it (`watercolorEdgeDarkening`
  rims where the painted regions meet, `watercolorGranulation` and
  `watercolorTurbulence` thinning and thickening the pigment about the
  paper, `watercolorDryBrush`, `watercolorWobble`), `ink` lines (with `inkDepth` for silhouettes and
  creases), `depthBands`, and early-3D `vertexSnap`, `affineTexturing`
  and `depthBits`. The UI and screen sprites stay unstylised.
- **`og.MaterialStyle`** sets every lit material that leaves the field at
  its default: `toon` (with a `gradientMap`), `shadowTint` and
  `shadowTintAmount`, `shadowDarkness` (1: every shadow a solid black
  mass, no sun and no ambient light there), `rimIntensity`, `outlineWidth` (with
  `outlineDistance` to thin lines with depth) and `flatShading`. Unlit and
  shader materials, sprites and the UI are untouched, and so is a material
  made with `styled: false` (exactly as declared under any look). The
  style's `toon` passes by a material with a physical lobe (`clearcoat`,
  `sheen`, `transmission`, `iridescence`, `anisotropy`, an `ior` other
  than 1.5), which keeps its physical shading and takes the rest; give it
  `toon: true` itself to make it toon anyway. An outline is a hull
  outline: each object's silhouette and its creases against other objects
  (the ground under it), never the edges between its own faces, so `.vox`
  models and flat-shaded meshes keep clean faces; for lines on every
  crease, `Stylise.inkDepth`.
- Built-in assets a look may name without owning a copy:
  `"@canvas/paper"`, `"@canvas/linen"`, `"@canvas/watercolor"` for
  `Stylise.paintCanvas`, and `"@palettes/lcd2"`, `"@palettes/handheld4"`,
  `"@palettes/dusk16"`, `"@palettes/bright32"`, `"@palettes/1bit"` for
  `Stylise.palette`.
- `RenderCapabilities.stylise` (`og.render.capabilities(world)`) names the
  primitive groups this device draws; a look using another still applies,
  without that primitive. A look in a streaming group applies once the
  group is resident (og-game §6).

## 14. The art contract: the fields

The render resources treat the frame; seven more resources of the `art`
module, set in the same look, say what the game's **assets** must be
(docs/art.md in the engine repository): `og.ArtPalette` (the palette the
texels sit on: `colors` a palette asset or `"@palettes/<name>"`, `strict`
indexed colour, `maxColors` per asset, `rampSteps` per hue, `accents`
kept for signals by palette index, `dither`), `og.ArtPixels` (2D: `unit`
logical pixels per metre, `logicalHeight`, `spriteSizes` the allowed cell
sides, `rotation` free, nearest, cleanEdge or prerotated with
`rotationSteps`, `snap`), `og.ArtTexels` (3D: `density` texels per metre,
`maxSize`, `filter`, `mips`), `og.ArtShape` (`propFaces` and
`characterFaces` triangle budgets, `normals` smooth, flat or angle past
`normalAngle`, `bevel`, `voxel` edge, `silhouette` thinnest feature),
`og.ArtLine` (`width`, `color`, `place` outside, inside or both,
`distanceFalloff`), `og.ArtMotion` (`stepFps` for animation on twos,
`maxFrames` per clip, `squash`) and `og.ArtCamera` (`projection`,
`fixedAngle` with `pitch` and `yaw`, `fovMin`/`fovMax`, `pixelPerfect`).
Every field, with its range and unit, is in `og api show art`.

```text
extends "16bit-pixel"            # the preset carries the style's contract
ArtPalette {
  colors "palettes/harbour.hex"   # the game's own palette, under assets/
  maxColors 32
  accents 3 7                     # palette indices kept for the player and danger
}
ArtPixels {
  unit 16                         # 16-pixel tiles one metre wide
  spriteSizes 16 32
}
ArtMotion {
  stepFps 12                      # on twos
}
```

- Every field is **no constraint at zero** (0, false, the first variant,
  an empty list): a look without these blocks constrains nothing, and a
  game made before them cooks and draws as it did. The presets carry each
  catalogue style's values (§12); the game's look overrides them
  field by field, as the frame's. Lists take one line (`spriteSizes 16
  32`), and the cook checks every field as it checks the frame's.
- `project.ogt`'s `look "looks/<id>.ogt"` names the look whose contract
  the cook reads (author it explicitly after creation); the cook's
  report on that look lists the art resources it sets. Without the line
  the cook reads no contract.
- They are resources, so game code reads the contract too (a system lists
  them among its `reads`): `must(world.resource(og.ArtPixels)).unit` to
  snap a sprite, `og.ArtPalette`'s `accents` to colour the player. Setting them changes neither
  the frame nor the simulation; `og.load.lookInfo(...).resources` names
  them among the look's blocks.
- A system that applies a look writes only the art resources it declares
  among its writes; the others keep their values, so a system declaring
  only the frame's resources applies a preset with a contract unchanged.
