<!-- Generated from the effect parsers' line tables (crates/og-bundle/src/fx.rs, effect.rs and vfx.rs) by `og api gen`. Do not edit: `og api check` fails on any difference. -->
# The effect files

Every line the three effect formats take, from the tables their parsers read: an emitter (`type particles`), an effect material (`type effect`) and a composed effect (`type vfx`). A parser refuses any other line and names the ones it takes. Each file starts `type <name>`, `version <n>` and `---`; a line left out takes its default. What the formats are for and how a game loads them is `loading §5` and `loading §11`; ready-made effects are `effect-packs`, pixel effects (`type vfx2d`) `pixel-vfx`.

## 1. Emitters (type particles)

`type particles`, `version 1`: one particle emitter, loaded with og.load.emitter or named by a vfx `emitter` element. `<min> [<max>]` ranges are uniform random per particle; one number is both ends. A curve is 1 to 8 `<t> <value>` pairs over the particle's life (t 0 to 1). Every line runs the same in compute and on the CPU path.

| Line | Default | Does |
|---|---|---|
| `maxParticles <n>` | `256` | The ring of particles, 1 to 1048576: the newest overwrite the oldest. |
| `shape point \| sphere <radius> [surface] \| cone <angle°> <radius> \| box <x> <y> <z> \| spline <x> <y> <z> ... [closed] \| mesh box \| sphere \| tetrahedron \| "<model id>"` | `cone 15 0.05` | Where particles are born and which way they start: a box by its half extents, a spline through 2 to 8 Catmull-Rom points, emitted evenly along it and along its tangent, a mesh's surface (a model's first mesh, in the emitter's space), area weighted and along its normals (a statue crumbling to embers, a sword's glow). |
| `spread <degrees>` | `0` | Random deviation from the shape's direction; 180 is every direction. |
| `rate <per second>` | `40` | Particles born per second. |
| `burst <time> <count> [<cycles> <interval>]` |  | Particles born at once at a time of each cycle, repeated `cycles` times `interval` apart; any number of lines. |
| `duration <seconds>` | `1` | Seconds per emission cycle. |
| `looping true \| false` | `true` | Start a new cycle at the end of each. |
| `lifetime <min> [<max>]` | `1 1.5` | Seconds a particle lives. |
| `speed <min> [<max>]` | `1.5 2.5` | Metres per second at birth. |
| `size <min> [<max>]` | `0.08 0.12` | Metres across at birth. |
| `sizeOverLife <t> <value> ...` |  | A curve that scales the size over the life (1 when left out): the width and, without `heightOverLife`, the height. |
| `heightOverLife <t> <value> ...` |  | A curve that scales the height alone over the life (along the sprite's up, or the velocity when stretched; a mesh's y): a tall flame, a flat shockwave. Left out, the height is the width. |
| `colorOverLife { key <t> <r> <g> <b> <a> ... }` |  | Linear colour keys over the life, at most 8; left out, white fading over the last quarter. |
| `randomColorOverLife { key <t> <r> <g> <b> <a> ... }` |  | A second colour over the life: each particle's colour lies between colorOverLife and this at a share of its own (embers from orange to deep red). Left out, every particle takes colorOverLife. |
| `color <r> <g> <b> [<a>]` | `1 1 1 1` | A linear tint over the colour keys; above 1 blooms. |
| `rotation <min> [<max>]` | `0` | The start rotation in degrees. |
| `angularVelocity <min> [<max>]` | `0` | Degrees per second. |
| `gravity <x> <y> <z>` | `0 -2 0` | Metres per second squared. |
| `drag <per second>` | `0` | How fast motion relative to the air dies. |
| `wind <0..1>` | `0` | The share of the air's motion (og.Wind and every og.WindMotor where the system is) that `drag` closes on: smoke and dust drift downwind; 0 slows to rest. |
| `forceFields <0..1>` | `0` | The share of the og.ForceField acceleration where each particle is that it takes (1: as a body does): an updraft lifting smoke, a vortex swirling sparks. World-space particles only. |
| `noise <strength> [<frequency> <scroll>]` | `0 0.5 0.3` | Turbulence: value noise pushing the particles. |
| `texture "<id>"` |  | A texture asset; left out, a soft dot. |
| `sheet <columns> <rows> [<frames> [off \| life <cycles> \| fps <n> \| random] [blend]]` | `1 1` | The texture as a sprite sheet: a frame by age, at a rate, or one at random. `blend` cross-fades each frame into the next as it plays (life and fps, billboards), so 16 frames of smoke roll smoothly instead of stepping; a sheet played once holds its last frame. |
| `blend additive \| alpha \| subtractive \| multiply` |  | Left out: alpha for a lit mesh (a solid chip; added it would glow white), additive for everything else. Subtractive takes the colour away from what is behind (dark smoke), multiply tints what is behind by it (scorch); neither in sprite mode. |
| `space world \| local` | `world` | Particles left behind in the world, or carried with the emitter. |
| `align camera \| velocity [<stretch>] \| upright [<x> <y> <z>] \| facing <x> <y> <z>` | `camera` | Billboards face the camera, stretch along their velocity (stretch 0.05 when left out), stand on an axis of the emitter turning to face the eye (0 1 0 when left out: flames), or lie across one whatever the camera (facing 0 1 0: flat on the ground). |
| `softDistance <metres>` | `0` | Fade where a billboard meets the scene behind it. |
| `collision [<bounce> [<friction>]] [depth \| colliders] [die]` |  | Bounce (0.4) with friction (0.2) off the drawn depth (what the camera sees), or off the scene's og.Collider shapes on or off screen; sprite mode always meets the colliders. With `die` a particle dies where it first hits (rain, sparks going out), a death event for a sub-emitter's splash. |
| `render billboard \| sprite [<layer>] \| mesh box \| sphere \| tetrahedron \| "<model id>"` | `billboard` | 3D billboards, 2D sprites in a sprite layer, or a small mesh per particle that tumbles with its rotation (a model's first mesh). |
| `lit true \| false` | `false` | Billboards and meshes shaded by the sun, the sky and local lights. |
| `roughness <0..1>` | `1` | A lit mesh's surface: 1 matte; smoother reflects over an alpha body (glass shards). |
| `distortion <0..1>` | `0` | Billboards bend the scene behind them by this fraction of the screen at most: heat haze, shock rings. |
| `events birth \| death \| collision ...` |  | Send og.ParticleEvent for these; the emitter's systems then simulate in the tick. |
| `subEmitter birth \| death \| collision "<id>" <count> [<inherit>] \| life "<id>" [<inherit>]` |  | Another emitter's particles born where this one's are born, die or bounce (count 1 to 4096, moving at their start velocity plus `inherit` times the parent's), or each particle running the other's own emission while it lives; up to 4, one level deep. |
| `glow <strength>` | `0` | Sprite mode: strength in the 2D glow buffer (PostEffects.glow). |
| `array "<id>" [<layers>] [off \| life <cycles> \| fps <n> \| random]` |  | Sprite mode: a layer of a texture array (`type texture-array`) per particle instead of `texture`, picked as `sheet` picks a frame from the first `layers`; one draw per emitter. |
| `round true \| false` | `false` | Fade each particle to the soft dot's round edge over its texture or layer. |
| `effect "<id>"` |  | Draw billboards or mesh particles with an effect material (`type effect`, §2). |
| `speedOverLife <t> <value> ...` |  | A curve that scales the velocity over the life (a burst that slows). |
| `spinOverLife <t> <value> ...` |  | A curve that scales the angular velocity over the life. |
| `rateOverTime <t> <value> ...` |  | A curve that scales `rate` over each cycle (t a fraction of `duration`). |
| `trail <seconds> [<width>] [only]` |  | A trail behind each particle: the path it took over the last <seconds>, a strip in its colour narrowing and fading to its tail, <width> a share of the particle's width (1 when left out); `only` draws the trails without the particles (a spark's streak). Billboards and mesh particles, not sprite mode. |
| `inheritVelocity <fraction>` | `0` | The share of the emitter's own velocity, in world space, particles start with. |
| `attractor <x> <y> <z> <strength> [<radius>]` |  | A point in the emitter's space that pulls, or pushes when negative; up to 4. |
| `field curl <strength> [<frequency> <scroll>] \| "<texture-or-field.ogt>" <strength> <size>` |  | Divergence-free curl noise (frequency 0.5, scroll 0.2), or a vector grid baked from an image N² wide and N high (rgb 0..1 as -1..1, y from bottom), or a lossless cubic 3-channel field asset (N 1..16). Explicit size maps one period of the source lattice to emitter-local units; field origin/spacing describe the source sampling domain, not particle placement. Vectors are not normalized. |
| `fieldGrid <cells> <size> <strength> <vectors>` |  | What `og cook` writes in place of a texture field: the baked grid, x fastest. Not written by hand. |

## 2. Effect materials (type effect)

`type effect`, `version 1`: how a billboard, a mesh, a trail or a beam is coloured (`effect "<id>"` on an emitter or a vfx mesh, trail or beam). A value is a number, `random <min> <max>` (fixed per particle, strip or mesh) or `curve <t> <v> ...` (over the life: a particle's age, a strip's length from its start, a mesh's life; at most 8 keys).

| Line | Default | Does |
|---|---|---|
| `main ["<id>"] [rgba \| red \| green \| blue \| alpha] [{ uv }]` |  | The main texture and the channel read; left out, the surface's own (an emitter's texture). |
| `ramp { key <t> <r> <g> <b> <a> ... }` |  | A gradient map: the texture's value picks the linear colour; at most 8 keys. |
| `mask ["<id>"] [<channel>] [{ uv }]` |  | A texture (red by default) multiplied into the alpha. |
| `noise ["<id>"] [<channel>] [{ uv }]` |  | The erosion's noise (red by default); left out, value noise. Needs an `erosion` line. |
| `erosion <value>` |  | The cut: where the noise is below it the surface is gone (a dissolve over the life). |
| `erosionSoftness <width>` | `0.05` | How soft the cut's edge is. |
| `erosionEdge <width> <r> <g> <b> [<a>]` |  | A glowing border of this width and linear colour along the cut. |
| `flowMap ["<id>"] [<channel>] [{ uv }]` |  | Red and green push the main uv (0.5 still). |
| `flow <strength> [<cycles per second>]` | `0.05 0.5` | How far and how fast the flow map pushes. |
| `fresnel <power> [<strength>] [invert]` |  | A rim brighter at grazing angles (strength 1), or with `invert` soft silhouettes. |
| `softDepth <metres>` | `0` | Fade against the scene behind (particles, meshes). |
| `color <r> <g> <b> [<a>]` | `1 1 1 1` | A linear tint; above 1 glows. |
| `intensity <value>` | `1` | Multiplies the colour. |
| `alpha <value>` | `1` | Multiplies the alpha. |
| `offsetU <value>` | `0` | Added to the main u. |
| `offsetV <value>` | `0` | Added to the main v. |
| `custom <slot 1 to 8> <value>` |  | A number a material shader reads: slots 1 to 4 are effect_custom1().x to .w, 5 to 8 effect_custom2(). |
| `blend additive \| alpha \| subtractive \| multiply` |  | Over the surface's own blend. Subtractive takes its colour away from what is behind (dark smoke, a void), multiply tints what is behind by it (scorch, shadow); a trail, ribbon or beam blends either as alpha. |
| `doubleSided true \| false` | `false` | Meshes: draw both faces. |
| `shader "<id>"` |  | A material shader (`type shader`) whose surface_color hook runs last; it reads effect_life(), effect_random(), effect_custom1() and effect_custom2(). |
| `param <name> <1 to 4 numbers>` |  | One of the shader's parameters by name. |

A texture line (`main`, `mask`, `noise`, `flowMap`) takes a block of how it is read:

| Line | Default | Does |
|---|---|---|
| `tile <u> [<v>]` | `1 1` | Repeats across the surface. |
| `offset <u> [<v>]` | `0 0` | Added to the uv. |
| `scroll <u> [<v>]` | `0 0` | Uv per second. |
| `rotate <degrees per second>` | `0` | Turns about the centre. |
| `polar true \| false` | `false` | u the angle about the centre, v the radius: rings and portals. |

## 3. Composed effects (type vfx)

`type vfx`, `version 2` (version 1 still reads, its `flash` as `glow`): one effect as a timeline of elements, spawned with og.vfx.spawn. Times are seconds from the effect's start; the tick they land on is the start plus the time times the tick rate, rounded. A value is as an effect material's, over the element's life. The header lines mean the same in a `type vfx2d` pixel effect.

| Line | Default | Does |
|---|---|---|
| `duration <seconds>` |  | How long the effect lasts; its elements end within it. Required. |
| `loop true \| false` | `false` | Start over at the end. |
| `impact <seconds>` |  | The same as `event <seconds> "impact"`; at most one. |
| `event <seconds> "<name>"` |  | An og.VfxEvent with this name at this time. |
| `param <name> <default>` |  | A number the game sets per instance (og.Vfx.params, in the order declared; at most 4): elements read it with `when` and `amount`. |
| `emitter "<id>" { ... }` |  | An emitter asset (§1) playing its own cycle; `until` stops its emission and its particles live out. |
| `mesh sphere \| quad \| ring \| box \| cylinder \| "<model id>" { ... }` |  | A mesh drawn with an effect material (§2); a ring builds its own shape at load. |
| `trail { ... }` |  | A trail behind the element's point. |
| `beam { ... }` |  | A beam from the element's point to `to`. |
| `light { ... }` |  | A point light. |
| `decal { ... }` |  | A decal projected down the element's -y, from og.RenderSettings.decalAtlas. |
| `glow { ... }` |  | A soft bright glow facing the eye (version 1 named it `flash`). |
| `shake { ... }` |  | og.camera.shake, at once. |
| `sound "<id>" { ... }` |  | A sound asset, played at once by a child entity with og.AudioSource, cut when the effect despawns. |
| `flipbook "<id>" { ... }` |  | A camera-facing og.Sprite with og.Flipbook playing a sheet's first clip once (a `type vfx2d` pixel effect or any og.load.flipbook sheet); its og.FlipbookEvent events, the impact among them, reach the game as this effect's og.VfxEvent. |

An element's block takes these lines; In names the elements that take one (every: all of them; ring: a `mesh ring`):

| Line | In | Default | Does |
|---|---|---|---|
| `at <seconds>` | every | `0` | When it starts, from the effect's start. |
| `life <seconds>` | every |  | How long it lasts (an element that lasts takes `life` or `until`; an emitter runs its cycle and a flipbook its clip without either). |
| `until <seconds>` | every |  | When it ends, instead of `life`. |
| `offset <x> <y> <z>` | every | `0 0 0` | Where it starts, in the effect's space. |
| `rotate <x> <y> <z>` | every | `0 0 0` | Degrees, in the effect's space. |
| `move <x> <y> <z>` | every | `0 0 0` | Metres per second it moves from its offset: an emitter spawns along the path, a mesh, trail or light goes with it. |
| `accelerate <x> <y> <z>` | every | `0 0 0` | Metres per second squared. |
| `when <param>` | every |  | Plays only in an instance whose value of the param is above 0 (a critical hit's extra burst, a decal only on the ground). |
| `amount <param>` | every |  | Multiplies how much of it there is by the param (at least 0): an emitter's share of particles born (at most 1), a mesh's, glow's, decal's or flipbook's size, a trail's or beam's width, a light's intensity, a shake's trauma, a sound's volume. |
| `stretch <param>` | mesh, glow |  | Multiplies its length alone by the param (at least 0), along its own y after `rotate`: a jet plume that lengthens with thrust without growing wider. Applied with `amount`, which scales it whole. |
| `seed <u32>` | emitter | `0` | Added to the effect's seed. |
| `effect "<id>"` | mesh, trail, beam |  | Its effect material (§2); a mesh needs one. |
| `scale <value>` | mesh | `1` | Uniform scale over its life. |
| `spin <x> <y> <z>` | mesh | `0 0 0` | Degrees per second. |
| `face camera` | mesh |  | Turned to face the eye as a billboard is (a glow ring seen from any side): its spin and scale kept, the effect's and its own rotation left out. Without it a mesh lies as it is placed. |
| `blend additive \| alpha \| subtractive \| multiply` | mesh, flipbook |  | A mesh adds by default, a flipbook blends alpha; a flipbook takes additive or alpha only. |
| `color <r> <g> <b> [<a>]` | mesh, trail, beam, decal, glow, flipbook, light | `1 1 1 1` | A linear colour; above 1 blooms (a light's alpha is ignored). |
| `width <head> [<tail>]` | trail | `0.2 0` | Metres across at its head and its tail. |
| `width <metres>` | beam | `0.2` | Metres across. |
| `lifetime <seconds>` | trail | `0.4` | How long a point stays in the trail. |
| `orbit <radius> <turns per second>` | trail | `0 0` | Circles the element's y. |
| `rise <metres per second>` | trail | `0` | Climbs the element's y. |
| `to <x> <y> <z>` | beam | `0 1 0` | Its far end, from the element's point. |
| `segments <n>` | beam | `1` | 1 to 256: more let `jitter` bend it. |
| `jitter <metres>` | beam | `0` | Sideways jitter, new every tick (lightning). |
| `intensity <value>` | light | `1` | Candela over its life. |
| `distance <metres>` | light | `5` | Where it reaches zero. |
| `uv <u0> <v0> <u1> <v1>` | decal | `0 0 1 1` | Its region of og.RenderSettings.decalAtlas. |
| `size <s> \| <x> <y> <z>` | decal | `1 1 1` | x, the depth it reaches down its -y, and z. |
| `fade <seconds>` | decal | `0` | Fades out over its last this many seconds. |
| `size <value>` | glow | `1` | Metres across over its life. |
| `size <metres>` | flipbook | `1` | Its height; the width follows the frame's shape. |
| `trauma <0..1>` | shake | `0.2` | og.camera.shake's trauma. |
| `volume <gain>` | sound | `1` | Its volume. |
| `playbackRate <rate> \| <min> <max>` | sound | `1` | A rate picked from the range by the effect's seed. |
| `inner <radius>` | ring |  | The inner radius (the outer is 1). |
| `band <0..1>` | ring |  | Where its middle row lies between the edges. |
| `height <outer> <inner>` | ring |  | Its edges raised along its axis: a cone, a tube. |
| `arc <start> <length>` | ring |  | Degrees from +x. |
| `segments <n>` | ring |  | 3 to 256 around it. |
| `colors <outer rgba> <band rgba> <inner rgba>` | ring |  | Its vertex colours; u runs along the arc and v from the outer edge (0) to the inner (1). |

## 4. Cook-generated sources

Each of `particles`, `effect` and `vfx` can contain one exclusive `generate` block instead of body declarations. The existing restricted TS executor returns `EffectSource {text}`: only the body, without type/version/header. The target format's ordinary parser validates the result at the file's declared version; imports, input bytes and parameters are tracked as in `generating`. Recursive generation is refused. See `file-types` for the generate block's fields.

`type effect-texture` cooks density/heat volumes or shared procgen noise/Worley/curl into a scattering sheet at its id, a thermal emission sheet at `#emission`, and chronological arrays at `#array` and `#emission#array`. All parameters, units and limits are in `og api show file:effect-texture`. `effects §5` explains sampling and has a checked recipe composing all three generated effect types with a generated volume.
