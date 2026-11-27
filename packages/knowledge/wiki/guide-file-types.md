<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `file-types` guide: File types

Use `og api list --kind file`, `og api show file:terrain` or `og api find erosion --kind file`. Each entry states its author input: migrated domains export a typed file declaration from content/**/*.ts; domains awaiting cutover still use the listed text input. Input paths and field coverage below are per type, not runtime API fields.

<a id="guide-file-types-1"></a>
### `file-types §1` Cook assets

<a id="file-animgraph"></a>
### `file:animgraph`

An authored `animgraph` asset, cooked by `animgraph`. Input: `.ogt` under assets/.

Field coverage: asset bindings and all nested graph declarations share the runtime parser's line tables. Output: a validated graph with tracked model animation dependencies; load with og.load.animgraph. Bind names are resolved at cook; runtime handles are never stored.

| Field | Default | Meaning |
|---|---|---|
| `bind "<name>" "<model asset>" "<clip>"` | `` | Bind a graph clip name to a cooked model's animation clip; repeat for each referenced clip. Asset-root relative paths, no runtime handles. |
| `sprite true \| false` | `false` | Resolve clip names on the flipbook/2D rig at playback, exclusive with bind. Graph structure is still checked at cook. |
| Field | Default | Meaning |
|---|---|---|
| `param <name>` | `` | Declare at most eight unique parameters, in component p0..p7 order. |
| `clip <motion> "<bound clip>" { ... }` | `` | Clip leaf, resolved by bind for skeletal assets or by the 2D target for sprite assets. |
| `blend1d <motion> <parameter> { ... }` | `` | Threshold interpolation; at least one child. |
| `blend2d <motion> <x parameter> <y parameter> { ... }` | `` | Freeform Cartesian interpolation; at least one child. |
| `layer <name> { ... }` | `required` | Layered state machine; at least one state. |

<a id="file-animgraph-clip"></a>
### `file:animgraph.clip`

| Field | Default | Meaning |
|---|---|---|
| `speed <number > 0>` | `1` | Clip playback rate. |
| `loop true \| false` | `true` | Loop playback. |
| `mirror true \| false` | `false` | Mirror the skeletal pose. |

<a id="file-animgraph-blend1d"></a>
### `file:animgraph.blend1d`

| Field | Default | Meaning |
|---|---|---|
| `child <motion> <threshold>` | `required` | Repeat children; thresholds are sorted, clamped at the ends. |

<a id="file-animgraph-blend2d"></a>
### `file:animgraph.blend2d`

| Field | Default | Meaning |
|---|---|---|
| `child <motion> <x> <y>` | `required` | Repeat children at 2D parameter points. |

<a id="file-animgraph-layer"></a>
### `file:animgraph.layer`

| Field | Default | Meaning |
|---|---|---|
| `state <name> <motion> { ... }` | `required` | Declare a unique state, with optional speed and sync. |
| `entry <state>` | `first state` | Initial state. |
| `mode override \| additive` | `override` | Pose combination mode. |
| `weight <number> \| <parameter>` | `1` | Fixed or parameter-driven layer weight, clamped to 0..1. |
| `mask "<node>" [weight]` | `` | Repeat subtree weights 0..1 (omitted weight one), resolved by skeleton node names. |
| `transition <state> \| any <target> { ... }` | `` | Repeat transitions in priority order; needs a when or exit. |

<a id="file-animgraph-layer-state"></a>
### `file:animgraph.layer.state`

| Field | Default | Meaning |
|---|---|---|
| `speed <number >= 0>` | `1` | State playback multiplier. |
| `sync <group>` | `` | Named phase synchronization group. |

<a id="file-animgraph-layer-transition"></a>
### `file:animgraph.layer.transition`

| Field | Default | Meaning |
|---|---|---|
| `when <parameter> gt \| ge \| lt \| le \| eq \| ne <number>` | `` | Repeat conditions; all must hold. |
| `exit <cycles>` | `` | Minimum source phase before the transition may fire. |
| `fade <seconds>` | `0.2` | Cross-fade duration, clamped at zero. |
| `interrupt any \| none \| source \| next` | `any` | Which transitions may interrupt this fade. |

<a id="file-behaviour"></a>
### `file:behaviour`

An authored `behaviour` asset, cooked by `behaviour`. Input: `.ogt` under assets/.


<a id="file-behaviour-chase"></a>
### `file:behaviour.chase`

| Field | Default | Meaning |
|---|---|---|
| `speed <metres/second \| operand>` | `agent` | Fixed speed or a blackboard/reflected field read at runtime. |
| `reach <metres>` | `0.5` | Stop distance. |

<a id="file-behaviour-command"></a>
### `file:behaviour.command`

| Field | Default | Meaning |
|---|---|---|
| `cooldown <seconds>` | `0` | Minimum command interval. |

<a id="file-behaviour-flee"></a>
### `file:behaviour.flee`

| Field | Default | Meaning |
|---|---|---|
| `speed <metres/second \| operand>` | `agent` | Fixed speed or a blackboard/reflected field read at runtime. |
| `distance <metres>` | `6` | Requested separation. |

<a id="file-behaviour-instant"></a>
### `file:behaviour.instant`

| Field | Default | Meaning |
|---|---|---|
| `stop ` | `` | Stop movement. |
| `set <key> <value>` | `` | Set a non-builtin blackboard key. |
| `command <snake_case command> [<args> ...] { cooldown ... }` | `` | Call an explicitly registered game command. |

<a id="file-behaviour-machine"></a>
### `file:behaviour.machine`

| Field | Default | Meaning |
|---|---|---|
| `initial <state>` | `first state` | Initial state name. |
| `state <name> { <nodes>; entry ...; exit ...; to ... }` | `` | Named state, body is an implicit ordered sequence. |

<a id="file-behaviour-move_to"></a>
### `file:behaviour.move_to`

| Field | Default | Meaning |
|---|---|---|
| `speed <metres/second \| operand>` | `agent` | Fixed speed or a blackboard/reflected field read at runtime. |

<a id="file-behaviour-nodes"></a>
### `file:behaviour.nodes`

| Field | Default | Meaning |
|---|---|---|
| `ReactiveFallback [<label>] { <nodes> }` | `` | First child that does not fail, reevaluated. |
| `ReactiveSequence [<label>] { <nodes> }` | `` | First child that does not succeed, reevaluated. |
| `Sequence [<label>] { <nodes> }` | `` | Ordered steps, remembers its running child. |
| `parallel [<label>] { policy all \| one; <nodes> }` | `all` | Nonempty children with explicit completion policy. |
| `utility [<label>] { inertia <number>; option <name> { score ...; <nodes> } }` | `0` | Scores the game's options; inertia adds to the running option. |
| `Inverter { <node> }` | `` | One child, invert success/failure. |
| `ForceSuccess { <node> }` | `` | One child; force successful completion. |
| `repeat [<u32>] { <node> }` | `0` | One child; zero repeats forever. |
| `cooldown <seconds> { <node> }` | `required` | One child with a cooldown. |
| `timeout <seconds> { <node> }` | `required` | One child with a deadline. |
| `check <operand> [is \| not \| below \| above \| atMost \| atLeast <value or operand>]` | `truthy` | Reads blackboard or reflected field. Values are number, bool or xyz. |
| `use "<behaviour asset>"` | `` | Inline a tree; tracked transitive dependencies, cycles fail. |
| `move_to <point key \| x y z> { speed ... }` | `` | Move toward the explicit point. |
| `chase [<target key>] { speed ...; reach ... }` | `target` | Chase the supplied target. |
| `flee [<target key>] { speed ...; distance ... }` | `target` | Move away from the supplied target. |
| `patrol <route> { speed ...; wait ...; loop ... }` | `` | Follow the declared route. |
| `wander { speed ...; radius ...; wait ... }` | `` | Seeded movement within the stated radius. |
| `wait <seconds>` | `required` | Wait for this duration. |
| `stop ` | `` | Stop movement. |
| `set <key> <value>` | `` | Set a non-builtin blackboard key. |
| `command <snake_case command> [<args> ...] { cooldown ... }` | `` | Call an explicitly registered game command. |
| `action <name> [<args> ...]` | `` | Game-owned action name under 64 bytes, arguments at most 255 bytes. |

<a id="file-behaviour-option"></a>
### `file:behaviour.option`

| Field | Default | Meaning |
|---|---|---|
| `score <number> \| <operand> linear \| inverse \| quadratic \| sqrt \| smooth \| step <lo> <hi>` | `required` | At least one score and node per utility option. |

<a id="file-behaviour-patrol"></a>
### `file:behaviour.patrol`

| Field | Default | Meaning |
|---|---|---|
| `speed <metres/second \| operand>` | `agent` | Fixed speed or a blackboard/reflected field read at runtime. |
| `wait <seconds>` | `0` | Wait between movement targets. |
| `loop true \| false` | `true` | Repeat the route. |
| Field | Default | Meaning |
|---|---|---|
| `name "<name>"` | `behaviour` | Asset name. |
| `key <name> <number \| boolean \| x y z>` | `` | Initial blackboard value; builtin keys cannot be redefined. |
| `route <name> { point <x> <y> <z> ... }` | `` | Nonempty patrol route, metres. |
| `tree { <node> ... }` | `` | Exactly one root node; exclusive with machine. |
| `machine { initial <state>; state ... }` | `` | State machine, exclusive with tree. |

<a id="file-behaviour-state"></a>
### `file:behaviour.state`

| Field | Default | Meaning |
|---|---|---|
| `entry { <instant actions> }` | `` | Only set, stop and command. |
| `exit { <instant actions> }` | `` | Only set, stop and command. |
| `to <state> { check ...; after ... }` | `` | Transition to an existing state. |

<a id="file-behaviour-to"></a>
### `file:behaviour.to`

| Field | Default | Meaning |
|---|---|---|
| `check <operand> [is \| not \| below \| above \| atMost \| atLeast <value or operand>]` | `truthy` | Reads blackboard or reflected field. Values are number, bool or xyz. |
| `after <seconds>` | `0` | Minimum time in state. |

<a id="file-behaviour-wander"></a>
### `file:behaviour.wander`

| Field | Default | Meaning |
|---|---|---|
| `speed <metres/second \| operand>` | `agent` | Fixed speed or a blackboard/reflected field read at runtime. |
| `wait <seconds>` | `0` | Wait between movement targets. |
| `radius <metres>` | `4` | Wander radius. |

<a id="file-dialogue"></a>
### `file:dialogue`

An authored `dialogue` asset, cooked by `dialogue`. Input: `.yarn` under assets/.

Source: `.yarn`, not an authored `type dialogue` OGT. This registered kind is the cooked dialogue IR. The cook rejects removed OGT dialogue syntax; write Yarn with explicit #line: ids. See story for Yarn declarations, node headers, commands, expressions and validation.


<a id="file-district"></a>
### `file:district`

An authored `district` asset, cooked by `district`. Input: `.ogt` under assets/.

| Field | Default | Meaning |
|---|---|---|
| `seed <u32>` | `0` | Deterministic layout seed. |
| `region { point <x> <z> ... }` | `` | Polygon in metres; exclusive with along. |
| `along { ... }` | `` | Street centreline and adjacent lots; exclusive with region. |
| `streets { ... }` | `` | Authored polylines and optional grid. |
| `lots { ... }` | `required` | Lot dimensions, metres. |
| `buildings { ... }` | `` | Explicit grammar rules and density, no inferred buildings. |
| `prop "<name>" { ... }` | `` | Repeated placements along streets. |
| `materials "<asset>"` | `required` | Material library. |
| `cells { ... }` | `` | Split output into streamed cells. |
| `hlod { ... }` | `` | Forwarded to scene HLOD processing. |
| `merge true \| false` | `true` | Merge generated static geometry. |
| `terrain "<asset>" { depth <metres> }` | `` | Project onto terrain; nonnegative carve depth. |

<a id="file-district-region"></a>
### `file:district.region`

| Field | Default | Meaning |
|---|---|---|
| `point <x> <z>` | `` | Repeated polygon vertices, metres. |

<a id="file-district-along"></a>
### `file:district.along`

| Field | Default | Meaning |
|---|---|---|
| `point <x> <z>` | `` | Repeated centreline vertices, metres. |
| `width <metres>` | `0` | Street width. |
| `sidewalk <metres>` | `0` | Sidewalk width. |
| `depth <metres>` | `0` | Depth of lots beside the centreline. |
| `name "<name>"` | `` | Street name used by rule filters. |

<a id="file-district-streets"></a>
### `file:district.streets`

| Field | Default | Meaning |
|---|---|---|
| `grid <x> <z>` | `` | Grid spacing in metres. |
| `turn <degrees>` | `0` | Grid rotation. |
| `jitter <metres>` | `0` | Grid jitter. |
| `width <metres>` | `0` | Grid street width. |
| `sidewalk <metres>` | `0` | Grid sidewalk width. |
| `kerb <0..2 metres>` | `0` | Kerb height. |
| `street ["<name>"] { ... }` | `` | Repeated authored centreline. |

<a id="file-district-streets-street"></a>
### `file:district.streets.street`

| Field | Default | Meaning |
|---|---|---|
| `point <x> <z>` | `` | Repeated centreline vertices, metres. |
| `width <metres>` | `0` | Street width. |
| `sidewalk <metres>` | `0` | Sidewalk width. |

<a id="file-district-lots"></a>
### `file:district.lots`

| Field | Default | Meaning |
|---|---|---|
| `width <least> <greatest>` | `required` | Lot width range in metres, least first. |
| `depth <least> <greatest>` | `required` | Lot depth range in metres, least first. |

<a id="file-district-buildings"></a>
### `file:district.buildings`

| Field | Default | Meaning |
|---|---|---|
| `density <0..1>` | `0` | Share of lots built on. |
| `height <least> <greatest>` | `0 0` | Nonnegative height range in metres. |
| `heightNoise <frequency> <share>` | `0 0` | Nonnegative frequency per metre and share 0..1. |
| `rule "<grammar asset>" { ... }` | `` | Repeated grammar candidates, required for nonzero density. |

<a id="file-district-buildings-rule"></a>
### `file:district.buildings.rule`

| Field | Default | Meaning |
|---|---|---|
| `weight <number > 0>` | `1` | Relative selection weight. |
| `street "<name>"` | `` | Restrict to this street. |

<a id="file-district-prop"></a>
### `file:district.prop`

| Field | Default | Meaning |
|---|---|---|
| `model "<asset>"` | `required` | Model to place. |
| `every <metres>` | `0` | Spacing along the street. |
| `jitter <metres>` | `0` | Placement jitter. |
| `kerb <metres>` | `0` | Offset from kerb. |
| `corners true \| false` | `false` | Place at corners. |
| `street "<name>"` | `` | Restrict to this street. |
| `collider box <w> <h> <d> \| capsule <radius> <height>` | `` | Explicit collision dimensions, metres; positive except capsule height may be zero. |

<a id="file-district-cells"></a>
### `file:district.cells`

| Field | Default | Meaning |
|---|---|---|
| `size <8..4096 metres>` | `0 (must be set)` | Cell side. |
| `load <metres >= 0>` | `0` | Load distance. |

<a id="file-district-terrain"></a>
### `file:district.terrain`

| Field | Default | Meaning |
|---|---|---|
| `depth <metres >= 0>` | `0` | Carve depth. |

<a id="file-district-hlod"></a>
### `file:district.hlod`

| Field | Default | Meaning |
|---|---|---|
| `size <4..4096 metres>` | `64` | Leaf cell side; doubles at each level. |
| `error <metres > 0, <= size / 4>` | `size / 256` | Maximum leaf proxy displacement; zero requests the derived default. |
| `texels <64..4096>` | `1024` | Longest texture side in pixels. |

<a id="file-effect"></a>
### `file:effect`

An authored `effect` asset, cooked by `effect`. Input: `.ogt` under assets/.

Field coverage: parser line tables; see `effect-files` for nested blocks and constraints.

Alternatively, one exclusive generate block returns EffectSource {text}: the body declarations without type/version/header. The ordinary parser and asset links validate the result.


<a id="file-effect-generate"></a>
### `file:effect.generate`

| Field | Default | Meaning |
|---|---|---|
| `script "<file.ts>"` | `required` | Synchronous default export; static relative imports and explicit inputs are tracked. |
| `seed <u32>` | `0` | Explicit seed passed to the generator. |
| `params { ... }` | `{}` | Named scalar, array or nested object parameters. |
| `input "<name>" "<path>"` | `` | Repeated input bytes, relative to the authored asset, available as ctx.inputs[name]. |
| Field | Default | Meaning |
|---|---|---|
| `main ["<id>"] [rgba \| red \| green \| blue \| alpha] [{ uv }]` | `` | The main texture and the channel read; left out, the surface's own (an emitter's texture). |
| `ramp { key <t> <r> <g> <b> <a> ... }` | `` | A gradient map: the texture's value picks the linear colour; at most 8 keys. |
| `mask ["<id>"] [<channel>] [{ uv }]` | `` | A texture (red by default) multiplied into the alpha. |
| `noise ["<id>"] [<channel>] [{ uv }]` | `` | The erosion's noise (red by default); left out, value noise. Needs an `erosion` line. |
| `erosion <value>` | `` | The cut: where the noise is below it the surface is gone (a dissolve over the life). |
| `erosionSoftness <width>` | `0.05` | How soft the cut's edge is. |
| `erosionEdge <width> <r> <g> <b> [<a>]` | `` | A glowing border of this width and linear colour along the cut. |
| `flowMap ["<id>"] [<channel>] [{ uv }]` | `` | Red and green push the main uv (0.5 still). |
| `flow <strength> [<cycles per second>]` | `0.05 0.5` | How far and how fast the flow map pushes. |
| `fresnel <power> [<strength>] [invert]` | `` | A rim brighter at grazing angles (strength 1), or with `invert` soft silhouettes. |
| `softDepth <metres>` | `0` | Fade against the scene behind (particles, meshes). |
| `color <r> <g> <b> [<a>]` | `1 1 1 1` | A linear tint; above 1 glows. |
| `intensity <value>` | `1` | Multiplies the colour. |
| `alpha <value>` | `1` | Multiplies the alpha. |
| `offsetU <value>` | `0` | Added to the main u. |
| `offsetV <value>` | `0` | Added to the main v. |
| `custom <slot 1 to 8> <value>` | `` | A number a material shader reads: slots 1 to 4 are effect_custom1().x to .w, 5 to 8 effect_custom2(). |
| `blend additive \| alpha \| subtractive \| multiply` | `` | Over the surface's own blend. Subtractive takes its colour away from what is behind (dark smoke, a void), multiply tints what is behind by it (scorch, shadow); a trail, ribbon or beam blends either as alpha. |
| `doubleSided true \| false` | `false` | Meshes: draw both faces. |
| `shader "<id>"` | `` | A material shader (`type shader`) whose surface_color hook runs last; it reads effect_life(), effect_random(), effect_custom1() and effect_custom2(). |
| `param <name> <1 to 4 numbers>` | `` | One of the shader's parameters by name. |

<a id="file-effect-texture"></a>
### `file:effect.texture`

| Field | Default | Meaning |
|---|---|---|
| `tile <u> [<v>]` | `1 1` | Repeats across the surface. |
| `offset <u> [<v>]` | `0 0` | Added to the uv. |
| `scroll <u> [<v>]` | `0 0` | Uv per second. |
| `rotate <degrees per second>` | `0` | Turns about the centre. |
| `polar true \| false` | `false` | u the angle about the centre, v the radius: rings and portals. |

<a id="file-effect-texture"></a>
### `file:effect-texture`

Author `content/**/*.ts` with `export default file["effect-texture"]({...})` from `@origingame/engine/author`. Project/module types are generated by `og api gen`; canonical output IDs are `<module>/<content-relative-stem>`. This domain does not accept OGT sources.

```ts
/** Offline texture transport input; all appearance values are explicit.
Atlas dimensions, total work and nonzero light direction are domain checks. */
export type Input = ({
/** Nonnegative Beer-Lambert extinction/density/cube width. */
readonly "absorption": (number);
/** Nonnegative isotropic scattering light; does not change alpha or emission. */
readonly "ambient": (number);
/** Signed Y acceleration per unit heat. */
readonly "buoyancy": (number);
/** Linear scattering RGB, each 0..1, illuminated by ambient/direct/light. */
readonly "color": (ReadonlyArray<(number)> & { readonly length: 3 });
/** Sheet columns. Domain validation also checks columns <= frames. */
readonly "columns": (number);
/** Nonnegative exponential heat loss/second. */
readonly "cooling": (number);
/** Nonnegative exponential density loss/second. */
readonly "decay": (number);
/** Density/second; explosion uses initial density. Nonnegative. */
readonly "density": (number);
/** Source boundary/density fBm modulation, 0..1; zero is a smooth sphere. */
readonly "detail": (number);
/** Nonnegative directional light with volume self-shadow; RGB clamps to sRGB8. */
readonly "direct": (number);
/** Seconds per substep, >0..0.1; noise time is (frame+1)*steps*dt. */
readonly "dt": (number);
/** Linear emission RGB/unit heat, each 0..1, written separately to #emission. */
readonly "emission": (ReadonlyArray<(number)> & { readonly length: 3 });
/** Nonnegative radial source speed in cube widths/s. */
readonly "expansion": (number);
/** 1..256 chronological frames, in row-major sheet order. */
readonly "frames": (number);
/** Positive cycles per unit cube; three simplex octaves, lacunarity 2, gain 0.5. */
readonly "frequency": (number);
/** 8..96 volume cells per axis of the unit cube; no per-frame rescaling. */
readonly "grid": (number);
/** Heat/second; explosion uses initial heat. Nonnegative. */
readonly "heat": (number);
/** Nonnegative source duration in seconds; explosion injects only initially. */
readonly "inject": (number);
/** Nonzero direction toward the light, normalized at cook. View rays travel +Z. */
readonly "light": (ReadonlyArray<(number)> & { readonly length: 3 });
/** Density/heat transport, or the shared noise/Worley/curl kernels; no preset. */
readonly "mode": ("smoke" | "fire" | "explosion" | "noise" | "worley" | "curl") & (string);
/** 1..128 Jacobi pressure iterations per volume step. */
readonly "pressure": (number);
/** Injection radius in cube widths, >0..0.5. */
readonly "radius": (number);
/** Positive encoding range: signed noise/curl -range..range maps to 0..1;
Worley 0..range maps to 0..1. Values clamp. Unused for volume transport. */
readonly "range": (number);
/** Seed for the same fBm/curl/Worley kernels as og.procgen. */
readonly "seed": (number);
/** Square frame resolution, 8..1024. Sheet extents <=8192; <=16777216 frame pixels. */
readonly "size": (number);
/** Injection centre in the unit cube, Y up. */
readonly "source": (ReadonlyArray<(number)> & { readonly length: 3 });
/** 1..64 fixed simulation substeps between frames; frame zero follows one interval. */
readonly "steps": (number);
/** Nonnegative curl acceleration; on noise textures this is Z travel speed. */
readonly "turbulence": (number);
/** Nonnegative vorticity confinement in 1/s, scaled by cell width; zero disables it. */
readonly "vorticity": (number);
});

```

Outputs: RGBA scattering sheet at the primary ID, thermal emission at #emission, arrays at #array and #emission#array, and their chronological #layerN textures. All parameters are required; sheet size, work budget and nonzero light direction are domain checks. Volume output is straight-alpha sRGB; noise/Worley/curl output is linear data. See effect-files for sampling and attribution.

<a id="file-environment"></a>
### `file:environment`

An authored `environment` asset, cooked by `environment`. Input: `.ogt` under assets/.

Field coverage: complete parser line table. Source bytes remain tracked by the existing importer; generation does not replace authored images or SVG content.

| Field | Default | Meaning |
|---|---|---|
| `source "<image or KTX2 cube>"` | `` | Exclusive with faces; paths resolve beside this file, then under assets/. |
| `faces "<+x>" "<-x>" "<+y>" "<-y>" "<+z>" "<-z>"` | `` | Six square faces, all HDR/EXR or all sRGB images; exclusive with source. |
| `size <power of two, 16..2048>` | `128 or prefiltered cube size` | Radiance face side. A prefiltered cube must keep its own size. |
| `intensity <number >= 0>` | `1` | Multiply input radiance. |
| `rotation <degrees>` | `0` | Turn about +Y, +X toward -Z. |
| `samples <16..4096>` | `256` | GGX samples per filtered texel; forbidden with prefiltered true. |
| `prefiltered true \| false` | `false` | Use an existing GGX KTX2 cube mip chain without filtering again. |
| `sun keep \| extract` | `extract for LDR, keep for HDR` | Extraction removes the sun from the map and records it as a light fact. |
| `hemisphere upper` | `` | Treat the image bottom edge as the horizon and fill the lower half. |
| `ground <linear r> <g> <b>` | `horizon mean` | Each channel 0..1; requires hemisphere upper. |
| `expand <0..0.9>` | `0.25` | Inverse tone expansion, applied to LDR only. |
| `sunBoost <0..100>` | `1` | Multiply the estimated LDR sun. |

<a id="file-field"></a>
### `file:field`

An authored `field` asset, cooked by `field`. Input: `.ogt` under assets/.

Field coverage: complete parser line table. Output: lossless f32 lattice, loaded with og.load.field. Authored data and generate are exclusive. No implicit normalization, resampling or image encoding.

| Field | Default | Meaning |
|---|---|---|
| `dimensions <x> <y> <z>` | `required` | Positive sample counts; x varies fastest, then y, then z. |
| `channels <1..4>` | `required` | Interleaved scalar channels per cell; the author defines their units and meaning. |
| `origin <x> <y> <z>` | `0 0 0` | Position of the first lattice sample in authored coordinate units. |
| `spacing <x> <y> <z>` | `1 1 1` | Positive distance between samples on each axis; no implicit resampling. |
| `values <float> ...` | `required` | Finite f32 samples, dimensions product times channels; no normalization or quantization. |

<a id="file-field-generate"></a>
### `file:field.generate`

Returns FieldSource: dimensions [x,y,z], channels 1..4, values, optional origin [x,y,z] and positive spacing [x,y,z]. Scripts compose noiseGrid, noiseGrad, Worley or curlNoise through the shared procgen implementation.

| Field | Default | Meaning |
|---|---|---|
| `script "<file.ts>"` | `required` | Synchronous default export; static relative imports and explicit inputs are tracked. |
| `seed <u32>` | `0` | Explicit seed passed to the generator. |
| `params { ... }` | `{}` | Named scalar, array or nested object parameters. |
| `input "<name>" "<path>"` | `` | Repeated input bytes, relative to the authored asset, available as ctx.inputs[name]. |

<a id="file-flipbook"></a>
### `file:flipbook`

An authored `flipbook` asset, cooked by `flipbook`. Input: `.ogt` under assets/.

| Field | Default | Meaning |
|---|---|---|
| `texture "<image>"` | `required` | Atlas image; cook resolves its dimensions. |
| `size <width> <height>` | `image dimensions` | Atlas size in pixels, nonnegative integers. |
| `filter nearest \| linear` | `nearest` | Texture sampling. |
| `array true \| false` | `false` | Cook equal-sized frames as texture-array layers. |
| `pivot <x> <y>` | `0.5 0.5` | Default frame pivot as fractions from bottom-left. |
| `margin <pixels>` | `0` | Nonnegative integer grid margin. |
| `spacing <pixels>` | `0` | Nonnegative integer grid spacing. |
| `grid <width> <height> [<columns> <rows>]` | `` | Positive pixel cell size; counts otherwise derived from atlas size. |
| `frame <name> <x> <y> <w> <h> [<pivot x> <pivot y>]` | `` | Repeated frame rectangles in pixels from top-left; optional fractional bottom-left pivot. |
| `clip <name> { ... }` | `` | Named animation with explicit frame sequence. |
| `atlas "<file.json>"` | `` | Cook imports Aseprite or TexturePacker frames, pivots and tags; no rotated frames. |

<a id="file-flipbook-clip"></a>
### `file:flipbook.clip`

| Field | Default | Meaning |
|---|---|---|
| `frames <name or index> ...` | `required` | Nonempty ordered frame sequence. |
| `fps <number > 0>` | `12` | Uniform playback rate when durations is absent. |
| `durations <seconds> ...` | `1 / fps` | Positive duration for every sequence frame. |
| `mode loop \| pingPong \| once` | `loop` | Playback mode. |
| `event <frame index> "<name>"` | `` | Repeated events on sequence frames, not atlas indices. |

<a id="file-grammar"></a>
### `file:grammar`

An authored `grammar` asset, cooked by `grammar`. Input: `.ogt` under assets/.

Numbers may name declared parameters. Dimensions are metres and roof angles degrees. A rule ends in label/insert/drop or dispatch; no building composition is inferred.

| Field | Default | Meaning |
|---|---|---|
| `param <name> <number>` | `` | Repeated named numeric parameters. |
| `start <rule>` | `required` | Starting rule. |
| `rule <name> { ... }` | `` | Named sequence ending in an emitting or dispatching operation. |

<a id="file-grammar-rule"></a>
### `file:grammar.rule`

| Field | Default | Meaning |
|---|---|---|
| `recess <depth> [<jamb rule>]` | `` | Recess the face in metres before its terminal operation. |
| `detail <integer 0..16>` | `0` | Output detail level. |
| `label <label>` | `` | Emit this surface with a material label. |
| `insert "<model asset>"` | `` | Insert a model into this scope. |
| `drop ` | `` | Emit no surface. |
| `extrude <depth> { ... }` | `` | Extrude in metres; map faces to rules. |
| `split x \| y { ... }` | `` | Split across or up the face. |
| `inset <width> <border rule> <inner rule>` | `` | Inset by metres. |
| `roof flat \| gable <degrees> \| hip <degrees> { ... }` | `` | Build a roof from the footprint. |
| `pick { option <weight> <rule> ... }` | `` | Deterministic weighted choice. |
| `case { ... }` | `` | Ordered size/parameter predicates, optional final otherwise. |

<a id="file-grammar-rule-extrude"></a>
### `file:grammar.rule.extrude`

| Field | Default | Meaning |
|---|---|---|
| `top <rule>` | `` | Top face rule. |
| `bottom <rule>` | `` | Bottom face rule. |
| `front <rule>` | `` | Front face rule. |
| `back <rule>` | `` | Back face rule. |
| `side <rule>` | `` | Side face rule. |
| `sides <rule>` | `` | All side faces rule. |

<a id="file-grammar-rule-split"></a>
### `file:grammar.rule.split`

| Field | Default | Meaning |
|---|---|---|
| `part <length> <rule>` | `` | Absolute length, metres. |
| `fill <weight> <rule>` | `` | Weighted share of remaining length. |
| `repeat <length> <rule>` | `` | Repeat at least once, stretching to fill. |

<a id="file-grammar-rule-roof"></a>
### `file:grammar.rule.roof`

| Field | Default | Meaning |
|---|---|---|
| `overhang <metres>` | `0` | Roof overhang. |
| `slope <rule>` | `required` | Rule for slopes. |
| `gable <rule>` | `` | Rule for gables. |
| `eave <rule>` | `` | Rule for eaves. |

<a id="file-grammar-rule-pick"></a>
### `file:grammar.rule.pick`

| Field | Default | Meaning |
|---|---|---|
| `option <weight > 0> <rule>` | `required` | Repeated weighted candidate. |

<a id="file-grammar-rule-case"></a>
### `file:grammar.rule.case`

| Field | Default | Meaning |
|---|---|---|
| `wider <metres> <rule>` | `` | Dispatch when wider than the value. |
| `taller <metres> <rule>` | `` | Dispatch when taller than the value. |
| `param <name> <value> <rule>` | `` | Parameter predicate. |
| `otherwise <rule>` | `` | Fallback, last. |

<a id="file-icons"></a>
### `file:icons`

An authored `icons` asset, cooked by `icons`. Input: `.ogt` under assets/.

Field coverage: complete parser line table. Source bytes remain tracked by the existing importer; generation does not replace authored images or SVG content.

| Field | Default | Meaning |
|---|---|---|
| `source "<directory>"` | `.` | SVG directory, relative to this file. |
| `size <integer, 1..512>` | `48` | Distance-field em size in pixels. |
| `range <integer, 1..512>` | `8` | Distance-field range in pixels. |
| `join miter \| round \| bevel` | `miter` | Join used by stroked SVG paths without their own join. |
| `icon "<name>" ["<SVG stem>"]` | `` | Repeat named glyphs in index order; omitted stem uses name. Names must be unique. |
| `sticker <outline> <dx> <dy>` | `0 0 0` | Outline 0..4 and offsets -4..4, in 24ths of the em. |

<a id="file-lightmap"></a>
### `file:lightmap`

An authored `lightmap` asset, cooked by `lightmap`. Input: `.ogt` under assets/.

Field coverage: derived bake record root and nested fields. Normally written by og bake lightmaps, not hand-authored; stale scene/geometry bindings require a new bake. Numeric arrays may span repeated lines. Unknown fields are refused.

| Field | Default | Meaning |
|---|---|---|
| `scene "<asset>"` | `required` | Scene asset id whose static lighting was baked. |
| `scene-hash "<hex u64>"` | `required` | Scene source hash when baked. |
| `pages "<file.ktx2>"` | `required` | RGBA16F array pages beside this record, consumed as a lightmap, not an independent texture. |
| `samples <count>` | `0` | Recorded bake sample count. |
| `seconds <seconds>` | `0` | Recorded elapsed bake time. |
| `model "<entity>" { ... }` | `` | Repeat a static model's baked geometry binding. |
| `volume "<entity>" { ... }` | `` | Repeat baked irradiance probes. |

<a id="file-lightmap-model"></a>
### `file:lightmap.model`

| Field | Default | Meaning |
|---|---|---|
| `asset "<model asset>"` | `required` | Model asset id. |
| `hash "<hex u64>"` | `required` | Geometry hash, not cooked blob hash. |
| `part { ... }` | `` | Parts in the model's order. |

<a id="file-lightmap-model-part"></a>
### `file:lightmap.model.part`

| Field | Default | Meaning |
|---|---|---|
| `vertices <count>` | `0` | Original vertex count, checked for stale bakes. |
| `xref <index> ...` | `` | Original vertex per lightmap vertex; repeated lines concatenate. |
| `indices <index> ...` | `` | Triangle indices; repeated lines concatenate. |
| `uv <u> <v> <page> ...` | `` | One UV/page triple per xref; v down. Repeated lines concatenate. |

<a id="file-lightmap-volume"></a>
### `file:lightmap.volume`

| Field | Default | Meaning |
|---|---|---|
| `min <x> <y> <z>` | `required` | Volume minimum in scene metres. |
| `max <x> <y> <z>` | `required` | Volume maximum in scene metres. |
| `fade <metres>` | `0` | Light fade distance outside its box. |
| `counts <x> <y> <z>` | `required` | Probe counts along each axis. |
| `probes <number> ...` | `` | Twelve coefficients per probe, x fastest: RGB each as c1.x c1.y c1.z c0. Repeated lines concatenate. |

<a id="file-look"></a>
### `file:look`

An authored `look` asset, cooked by `look`. Input: `.ogt` under assets/.

Field coverage: parser-owned root fields, UI seed names, and writable reflected resource fields. No runtime-only animated fields. Resource references use asset ids, enum fields accept variant words, colours accept linear numbers or quoted sRGB hex.

| Field | Default | Meaning |
|---|---|---|
| `name "<name>"` | `` | Display name. |
| `extends "<look>" ...` | `` | Base looks, nearest first; at most eight inheritance levels. Asset id or a look slug. |
| `mood "<mood>"` | `` | Mood from MOODS; applied as the existing preset's explicit layer. |
| `variant "<variant>"` | `` | Existing preset variant slug: lowercase alphanumeric words separated by hyphens. |
| `ui "<style>"` | `` | UI style slug or asset id. |
| `antialias <mode>` | `` | Antialias mode from ANTIALIAS. |

Mood values: bright, dark, warm, cold, soft, high-contrast. Antialias values: tier, off, msaa4, taa.


<a id="file-look-uiSeeds"></a>
### `file:look.uiSeeds`

| Field | Default | Meaning |
|---|---|---|
| `primary "#rrggbb"` | `` | UI seed colour, sRGB. |
| `accent "#rrggbb"` | `` | UI seed colour, sRGB. |
| `neutral "#rrggbb"` | `` | UI seed colour, sRGB. |
| `danger "#rrggbb"` | `` | UI seed colour, sRGB. |
| `warn "#rrggbb"` | `` | UI seed colour, sRGB. |
| `success "#rrggbb"` | `` | UI seed colour, sRGB. |
| `info "#rrggbb"` | `` | UI seed colour, sRGB. |
| `team "#rrggbb"` | `` | UI seed colour, sRGB. |

<a id="file-look-RenderSettings"></a>
### `file:look.RenderSettings`

| Field | Default | Meaning |
|---|---|---|
| `environmentIntensity f32` | `0.55` | The strength of the image-based ambient light from the sky on surfaces; og.Environment.intensity takes its place while an environment map is set. The sky and its clouds are drawn as they are whatever it is. |
| `sky bool` | `false` | Draw the procedural sky behind the scene instead of Camera.background (with an og.Environment map set, the map). |
| `bloom bool` | `false` | Light above the threshold bleeds into its surroundings. |
| `bloomThreshold f32` | `1` | Brightness after exposure where bloom starts; 1 is display white. |
| `bloomKnee f32` | `0.5` | Softens the threshold over threshold plus or minus knee; 0 is a hard cut. |
| `bloomIntensity f32` | `0.6` | How much of the blurred light is added back. |
| `bloomRadius f32` | `-1` | Below 0 (the default) keeps Origin's bloom. 0 to 1 is three.js UnrealBloomPass at this radius: a half-resolution high pass from bloomThreshold to bloomThreshold + bloomKnee, five Gaussian mips mixed by mix(f, 1.2 - f, radius), added at bloomIntensity as EffectComposer's additive copy. See `render §5`. |
| `bloomContrast f32` | `1` | With bloomRadius 0 or more: the high pass (and the blur) sees each channel as 0.18 x (colour / 0.18) to this power before exposure, as a contrast grade run ahead of UnrealBloomPass (the way a three.js grade pass before the bloom does) would leave it. 1, the default, leaves the colour as lit. |
| `autoExposure bool` | `false` | Meter every frame to a key that follows its brightness (middle grey for a daylit view), then apply Camera.exposure as compensation. With both adapt speeds 0 the exposure is the frame's own; otherwise it adapts from the exposure metered on the first frame after the first tick, so hosts draw alike. See `render §5`. |
| `autoExposureMin f32` | `-10` | The least exposure auto exposure applies, in stops (EV: the image times 2^EV, Camera.exposure's compensation included): a very bright view is darkened no further. See `render §5`. |
| `autoExposureMax f32` | `10` | The most exposure auto exposure applies, in stops (EV, compensation included): a dark view is brightened no further, so a night stays as dark as its light makes it. Below autoExposureMin it is taken as autoExposureMin. See `render §5`. |
| `adaptSpeedUp f32` | `0` | How fast auto exposure rises when the view gets darker, in EV per second of game time: each drawn frame moves it by at most the speed times the ticks since the last drawn frame, never by the wall clock; 0 is instant. Snapshots and probe sweeps carry the adapted exposure. |
| `adaptSpeedDown f32` | `0` | How fast auto exposure falls when the view gets brighter, in EV per second of game time; 0 is instant. See `render §5`. |
| `toneMapped bool` | `true` | Tone mapping by toneMapping; false clamps, so 2D colours come out exactly as authored at exposure 1. |
| `toneMapping ToneMapping` | `"aces"` | Which curve tone maps the frame while toneMapped is true: aces (the default), agx, neutral or none. |
| `exposure f32` | `1` | A multiplier on every camera's Camera.exposure, as three.js renderer.toneMappingExposure is one for the whole frame: a look's brightness without touching the cameras; 1 leaves them as they are. |
| `lodBias f32` | `1` | A multiplier on every model's projected level-of-detail error (og.Model.lodBias multiplies it again): a level is drawn while its error, measured by the cook against the full mesh, covers under a pixel. 1 is the engine's choice, 2 keeps full meshes twice as far; 0 draws every model in full. See `render §10`. |
| `bakedLighting bool` | `true` | Draw the baked lighting of the scenes that have it (og bake lightmaps): static models with their lightmaps, everything else inside a LightVolume with its probes. False draws every scene as if unbaked, its ambient light from the sky or the environment map. See `render §4`. |
| `staticLightReach f32` | `0` | How far from itself a stationary or static light (LightMobility) has its diffuse light shaded per pixel; past it, that light comes from a grid of probes round the camera and such a rect light skips rough surfaces. Glossy surfaces keep the highlights. 0 shades every light whole. See `render §3`. |
| `quality QualityPreset` | `"none"` | The graphics preset og.render.applyQuality last wrote into the quality fields below (rectShadows to upscaler), or none. A record for the settings screen and the probe: the renderer reads the fields, never this, so a field the game sets after the preset is drawn as set. |
| `rectShadows bool` | `true` | Rect lights whose castShadow is on cast their soft shadows; false draws every rect light unshadowed, each keeping its castShadow for when this is on again. The low preset turns it off. See `render §3`. |
| `shadowedLights u32` | `8` | At most this many point, spot and rect lights cast shadows in a frame, the brightest at the camera kept (RenderCapabilities.maxShadowedLights caps it); 0 casts none. The high preset keeps 16, medium 8, low 2. The directional light's cascades are not counted. |
| `shadowCascades u32` | `3` | The directional light's shadow cascades, 1 to 4: more cover the same shadowDistance with finer shadows near the camera, at one more shadow pass and one more map each. The high preset draws 4, medium 3, low 2. Changing it makes the cascade maps again at the next frame. |
| `shadowMapScale f32` | `1` | The shadow maps at this fraction of their full side: the cascades (2048 texels by default) and local lights' layers (512), each within 64 texels and the device's limit (4096 for cascades). Changing it remakes the maps next frame. |
| `volumetricFog bool` | `true` | A volumetric og.Fog (Fog.volumetric) is drawn lit, with its light shafts, and so is the sun's light under an og.Water (its shafts, where it has caustics); false draws the fog as plain distance and height fog and the water's light even. |
| `skyCaptureSliced bool` | `false` | Spread live sky capture and filtering over 13 rendered frames instead of completing each frame. Low sets true; medium, high and the default keep false so terrain ambient and reflections follow the current sky. Slicing lowers peak GPU cost but retains older lighting during fast sky changes. |
| `softParticles bool` | `true` | Particles with a softDistance fade against the scene behind them; false draws them hard-edged and asks no depth prepass for them. |
| `particleEmission f32` | `1` | Every particle system's emission times this (0 to 1); an emitter left with fewer than 8 particles alive at once keeps more, so a flash or a ring stays whole. The low preset draws 0.5. Drawing only: particle events and collisions are the full emitter's. |
| `maxVfx u32` | `64` | At most this many composed effects (og.Vfx) drawn at once: past it the oldest one-shot effects are left out, then the oldest looping ones, and the probe's dump_render counts them (vfx.dropped). 64; the low preset 32, high 128. Drawing only: their events, shakes and sounds still come. |
| `maxVfxParticles u32` | `65536` | At most this many particle slots (the emitters' maxParticles, summed) among the og.Vfx drawn: past it the oldest one-shot effects are left out, then the oldest looping ones, as with maxVfx. 65536; the low preset 32768, high 131072. |
| `screenSpaceReflections bool` | `true` | PostEffects.ssr draws; false leaves smooth surfaces the reflection probes' and the environment's reflection. PostEffects keeps its settings either way. |
| `ambientOcclusion bool` | `true` | PostEffects.ao darkens creases; false draws none. PostEffects keeps its settings either way. |
| `aoResolution AoResolutionSetting` | `"postEffects"` | The resolution ambient occlusion is searched at: postEffects (the default) draws PostEffects.aoResolution; full or half is drawn in its place, and PostEffects keeps its own for when this is postEffects again. See `render §5`. |
| `ssrStepScale f32` | `1` | PostEffects.ssrSteps times this (within the 4 to 128 steps the pass takes): 2 is 128 steps for the default 64, finer contact with thin and distant reflected surfaces at a higher GPU time. PostEffects keeps its steps. |
| `volumetricStepScale f32` | `1` | Fog.volumetric's samples along each view ray times this: the froxel grid's 64 slices deep (8 to 256), or the raymarch's 24 steps per pixel (4 to 96) where the renderer draws one (RenderCapabilities.volumetricFog). Changing it makes the grid again at the next frame. See `render §7`. |
| `msaaSamples u32` | `4` | Samples per pixel where the antialiasing multisamples (msaa4): 4, or 1, which draws FXAA instead. RenderCapabilities.maxMsaaSamples caps it: 1 where the device resolved no MSAA, which draws one sample whatever this says. Changing it remakes the multisampled targets and pipelines next frame. See `render §2`. |
| `upscaler Upscaler` | `"auto"` | Under TAA, the temporal resolve from the render scale (DisplaySettings.renderScale) to the output: auto (the default and medium: fsr3 below scale 1, taau at it), taau (low) or fsr3 (high). A device lacking fsr3 (RenderCapabilities.upscalers) draws taau. Switching restarts the history. See `render §1`. |
| `interpolate bool` | `true` | Draw what moves between the last two ticks: a frame between ticks blends entities, og.Camera and skinned joints from the tick before the last to the last, showing the world up to one tick late. Frames on a tick boundary, screenshots and captures draw the last tick. Presentation only. See `render §9`. |
| `snapDistance f32` | `10` | An entity or og.Camera that moved further than this between the last two ticks is drawn at the last tick, not blended: a teleport or a respawn this far needs no og.render.snap. 0 never snaps by distance. |

`environmentIntensity` range: 0 .. unbounded; unit: unstated.


`bloomThreshold` range: 0 .. unbounded; unit: unstated.


`bloomKnee` range: 0 .. unbounded; unit: unstated.


`bloomIntensity` range: 0 .. unbounded; unit: unstated.


`bloomRadius` range: -1 .. 1; unit: unstated.


`bloomContrast` range: 0.5 .. 2; unit: unstated.


`autoExposureMin` range: -20 .. 20; unit: unstated.


`autoExposureMax` range: -20 .. 20; unit: unstated.


`adaptSpeedUp` range: 0 .. 100; unit: evPerSecond.


`adaptSpeedDown` range: 0 .. 100; unit: evPerSecond.


`toneMapping` variants: aces, agx, none, neutral.


`exposure` range: 0 .. 16; unit: unstated.


`lodBias` range: 0 .. 16; unit: unstated.


`staticLightReach` range: 0 .. unbounded; unit: m.


`quality` variants: none, low, medium, high.


`shadowedLights` range: 0 .. 16; unit: unstated.


`shadowCascades` range: 1 .. 4; unit: unstated.


`shadowMapScale` range: 0.125 .. 4; unit: unstated.


`particleEmission` range: 0 .. 1; unit: unstated.


`aoResolution` variants: postEffects, full, half.


`ssrStepScale` range: 0 .. 32; unit: unstated.


`volumetricStepScale` range: 0 .. 4; unit: unstated.


`msaaSamples` range: 1 .. 4; unit: unstated.


`upscaler` variants: taau, fsr3, auto.


`snapDistance` range: 0 .. unbounded; unit: m.


<a id="file-look-PostEffects"></a>
### `file:look.PostEffects`

| Field | Default | Meaning |
|---|---|---|
| `ao bool` | `false` | Screen-space ambient occlusion: darkens the ambient light in creases and contact areas. |
| `aoRadius f32` | `0.6` | The world-space radius searched for occluders. |
| `aoIntensity f32` | `1` | 0 none, 1 full; above 1 exaggerates. |
| `aoMethod AoMethod` | `"auto"` | The algorithm. |
| `aoResolution AoResolution` | `"half"` | Half (the default) searches and blurs at half resolution with a normal- and depth-aware upsample; full searches every pixel. QualityPreset.high sets RenderSettings.aoResolution to full. |
| `aoDirect f32` | `0` | How much the occlusion also darkens direct and local light (0 only the ambient light, 1 a fully occluded pixel keeps 35 % of it). Above 0 the AO sees and darkens water surfaces too; at 0 water is outside it. |
| `aoRadiusScale f32` | `0` | Above 0 the radius grows with the distance, min(aoRadius, aoRadiusScale × distance), a fixed share of the screen up close as three.js SSAO passes size theirs; 0 keeps aoRadius everywhere. See `render §5`. |
| `outline bool` | `false` | Outlines every opaque surface from depth and normal edges, creases between the faces of one object included (materials with outlineWidth get their own hull outline instead). |
| `outlineColor Color` | `{"r":0.02,"g":0.02,"b":0.03}` | The line colour, linear. |
| `outlineWidth f32` | `2` | Line width in pixels, drawn inside the silhouette. |
| `outlineDepthThreshold f32` | `0.04` | The relative depth step that counts as a silhouette (0.04 is 4 %). |
| `outlineNormalThreshold f32` | `0.4` | The normal change that counts as a crease (1 - cos of the angle; 0.4 is about 53 degrees). |
| `depthOfField bool` | `false` | Bokeh depth of field: pixels away from the focus distance blur into discs. |
| `dofFocusDistance f32` | `10` | The distance from the camera that is sharp. |
| `dofFocusRange f32` | `2` | The half-width of the sharp band around it. |
| `dofMaxBlur f32` | `0.012` | The largest blur radius, as a fraction of the image height (0.012 is about 9 px at 720p). |
| `dofFalloff f32` | `4` | How quickly blur grows outside the band: full blur focusRange times (1 + falloff) away. |
| `dofTiltShift f32` | `0` | Tilt-shift: above 0 the sharp band is a strip across the image's middle, this fraction of its height either side of the centre line, at any camera distance; blur grows above it as far and below it as near, full at dofTiltShift times (1 + dofFalloff) past it. 0 focuses by depth. See `looks §19`. |
| `dofQuality DofQuality` | `"low"` | Taps per pixel. |
| `motionBlur bool` | `false` | Per-object and camera motion blur along the screen-space velocity. |
| `motionBlurShutter f32` | `0.5` | The shutter as a fraction of the frame (0.5 is a 180 degree shutter). |
| `motionBlurSamples u32` | `12` | Taps along the velocity. |
| `motionBlurMaxPixels f32` | `40` | The longest blur in pixels. |
| `screenBlur ScreenBlur` | `"none"` | A zoom, spin or directional blur of the whole frame (RedGPU's zoom, radial and directional blurs), weighted to the middle of its reach: one pass after motion blur and before bloom and the tone curve, so highlights streak and glow; the UI is not blurred. Off at none or at screenBlurStrength 0. |
| `screenBlurStrength f32` | `0` | How far it reaches each way: for zoom the fraction of the way to the centre, for spin radians, for directional pixels at 720 lines; 0 is off. |
| `screenBlurCenter Vector2` | `{"x":0.5,"y":0.5}` | Zoom's and spin's centre, in fractions of the frame from the top left. |
| `screenBlurAngle f32` | `0` | Directional's direction, counter-clockwise from the right (90 is vertical). |
| `screenBlurSamples u32` | `16` | Taps per pixel for spin and directional. Zoom exceeds this quality floor with three eight-tap stages (512 effective positions), or direct adaptive sampling when strength >= 1 crosses the centre. Both use stable per-pixel jitter. |
| `godRays bool` | `false` | Screen-space light shafts from the DirectionalLight (the sun must be in front of the camera). The sun casts them with its own light, as a source of a fixed angular size whatever size its disc is drawn, dimmed by the clouds over it; the bright sky around it adds its own. |
| `godRaysIntensity f32` | `0.5` | How bright the shafts are. |
| `godRaysDecay f32` | `0.965` | How fast the rays fade along their length. |
| `godRaysDensity f32` | `1` | How far towards the sun each ray samples (a fraction of the distance). |
| `godRaysTint Color` | `{"r":1,"g":1,"b":1}` | A linear tint on the light the rays carry. |
| `godRaysSamples u32` | `48` | Taps per ray. |
| `godRaysThreshold f32` | `2` | Brightness (linear, before exposure) below which the sky and the sun carry no rays: only what is brighter casts shafts. |
| `glow bool` | `false` | Order-independent 2D glow: sprites with SpriteDesc.glow (and 2D particles with a glow) are blurred and added back, with or without bloom. |
| `glowIntensity f32` | `1` | How much of the blurred glow is added back. |
| `lut Lut` | `0` | Colour grading by a lookup table from og.load.lut; 0 is none (and so is a LUT whose streaming group is not resident). |
| `lutIntensity f32` | `1` | 0 leaves the image as tone mapped, 1 applies the table fully. |
| `temperature f32` | `0` | White balance: towards warm orange (positive) or cool blue (negative), Unity's Temperature divided by 100 (a von Kries scaling in LMS from a white point moved along the daylight locus); 0 is off. See `render §5`. |
| `tint f32` | `0` | White balance: towards magenta (positive) or green (negative), as Lightroom's and Unity's Tint divided by 100; 0 is off. |
| `brightness f32` | `0` | Added to every channel of the display colour; 0 is off. |
| `contrast f32` | `0` | Contrast around mid grey: positive spreads the tones by 1 / (1 - contrast), negative pulls them in by 1 + contrast (-1 is flat grey); 0 is off. |
| `balanceShadows Vector3` | `{"x":0,"y":0,"z":0}` | The colour balance of the shadows (luminance below 0.5, fading out towards it): x shifts from cyan to red, y from magenta to green, z from yellow to blue, each -1 to 1 added to its channel; zero is off. |
| `balanceMidtones Vector3` | `{"x":0,"y":0,"z":0}` | The colour balance of the mid-tones (what the shadows and highlights leave), as balanceShadows. |
| `balanceHighlights Vector3` | `{"x":0,"y":0,"z":0}` | The colour balance of the highlights (luminance above 0.5, fading in from it), as balanceShadows. |
| `preserveLuminosity bool` | `true` | The colour balance keeps each pixel's luminance, so it moves only the hue (Photoshop's Preserve Luminosity). |
| `hue f32` | `0` | Turns every hue about the grey axis (red towards green at 120); 0 is off. |
| `saturation f32` | `0` | -1 grey, 0 unchanged, 1 twice the colour (the chroma around the luminance scaled by 1 + saturation). |
| `vibrance f32` | `0` | Saturation that spares colours already saturated and skin tones: positive livens dull colours, negative mutes them; 0 is off. |
| `grayscale f32` | `0` | How far each colour goes towards its luminance (grey); 0 is off. |
| `invert f32` | `0` | How far each colour goes towards its complement; 0 is off. |
| `threshold f32` | `0` | Above 0, each pixel white where its luminance reaches this level and black below it (the table's 32 steps soften the edge over about 1/31 of the range); 0 is off. |
| `vignette bool` | `false` | Darkened (or tinted) edges of the frame. |
| `vignetteIntensity f32` | `0.35` | 0 none, 1 black corners. |
| `vignetteRadius f32` | `0.55` | Where darkening starts, as a fraction of the half-diagonal. |
| `vignetteSmoothness f32` | `0.45` | The width of the fade. |
| `vignetteColor Color` | `{"r":0,"g":0,"b":0}` | The linear colour the edges move towards. |
| `chromaticAberration f32` | `0` | Red and blue split towards the edges, in pixels at the corners at 720p; 0 is off. |
| `lensDistortion f32` | `0` | Barrel (positive: lines bow out) or pincushion (negative: in) distortion; 0 is off. Radius in pixels at any aspect; a barrel keeps its farthest corner on the image. After the depth-reading Stylise passes, before palette quantisation; the UI is not bent, pointer raycasts see the undistorted frame. See `render §5`. |
| `lensDistortionCenter Vector2` | `{"x":0.5,"y":0.5}` | The lens's centre, in fractions of the frame from the top left. |
| `filmGrain f32` | `0` | Luminance-weighted noise amplitude in display units, a new pattern every tick (0.04 is subtle); 0 is off. |
| `filmGrainSize f32` | `1.5` | The grain size in pixels. |
| `sharpen f32` | `-1` | Contrast-adaptive sharpening (CAS) of the final image, as a three.js CAS pass after the output pass: each pixel pushed away from its four neighbours by a weight that falls where the neighbourhood is already near black or white. 0 is the mildest, 1 the strongest; below 0 is off. |
| `flashColor Color` | `{"r":1,"g":1,"b":1}` | The flash's linear display colour. |
| `shakeMaxOffset f32` | `0.05` | The largest offset at trauma 1, as a fraction of the view height. |
| `shakeMaxRoll f32` | `3` | The largest roll at trauma 1. |
| `shakeFrequency f32` | `12` | How fast the shake moves. |
| `shakeSeed u32` | `0` | The shake's noise seed. |
| `ssr bool` | `false` | Stochastic, denoised screen-space reflections (after FidelityFX SSSR and Frostbite): a GGX-lobe ray a pixel up to ssrMaxRoughness through a depth pyramid, reflecting the previous frame's lit image; on a miss, the probes', probe GI's and environment's. Opaque, perspective 3D; turns on the prepass. See `render §5`. |
| `ssrMaxDistance f32` | `100` | How far a reflected ray marches, in world units; the last fifth fades into the probes'. The march's cost is set by ssrSteps, not by the distance. |
| `ssrThickness f32` | `0.3` | How far behind the depth buffer a surface is taken to extend: a ray passing behind a surface by less than this hits it. |
| `ssrSteps u32` | `64` | The hierarchical march's iterations along each ray, each crossing a cell of the min-max depth pyramid's current level; a ray out of steps keeps the probes' reflection. Only the rays that need the steps pay for them. See `render §5`. |
| `ssrFade f32` | `0.15` | The fraction of the screen's edge over which a reflection fades into the probes' (the last fifth of ssrMaxDistance fades too). |
| `ssrIntensity f32` | `1` | How much of the reflection replaces the probes' and the environment's where it resolved; 1 is physical (weighted by the surface's Fresnel and lobe as the probes' are). |
| `ssrMaxRoughness f32` | `0.6` | The roughest surface traced: rougher ones keep the probes' reflection, and the last quarter below it fades into them. See `render §5`. |
| `ssrResolution AoResolution` | `"half"` | Half (the default): one ray per 2x2 block, a different pixel of the block each frame, denoised on the blocks and upsampled to the pixels by their depth, normal and roughness. Full: one ray per pixel, denoised at full resolution. See `render §5`. |
| `custom Post` | `[]` | Custom post passes in order (og.load.post), at most RenderCapabilities.maxCustomPosts (8), each over the one before, after depth of field and motion blur, before exposure and bloom, reading depth and normals (the prepass). A 0 is skipped, as is one failing to compile (og.render.postErrors). See `render §5`. |

`aoRadius` range: 0 .. unbounded; unit: m.


`aoIntensity` range: 0 .. unbounded; unit: unstated.


`aoMethod` variants: auto, gtao, ssao, screen.


`aoResolution` variants: full, half.


`aoDirect` range: 0 .. 1; unit: unstated.


`aoRadiusScale` range: 0 .. unbounded; unit: unstated.


`outlineWidth` range: 0 .. 31; unit: unstated.


`outlineDepthThreshold` range: 0 .. unbounded; unit: unstated.


`outlineNormalThreshold` range: 0 .. unbounded; unit: unstated.


`dofFocusDistance` range: 0 .. unbounded; unit: m.


`dofFocusRange` range: 0 .. unbounded; unit: m.


`dofMaxBlur` range: 0 .. 0.1; unit: unstated.


`dofFalloff` range: 0 .. unbounded; unit: unstated.


`dofTiltShift` range: 0 .. 0.5; unit: unstated.


`dofQuality` variants: low, high.


`motionBlurShutter` range: 0 .. 2; unit: unstated.


`motionBlurSamples` range: 4 .. 32; unit: unstated.


`motionBlurMaxPixels` range: 1 .. unbounded; unit: unstated.


`screenBlur` variants: none, zoom, spin, directional.


`screenBlurStrength` range: 0 .. unbounded; unit: unstated.


`screenBlurAngle` range: unbounded .. unbounded; unit: deg.


`screenBlurSamples` range: 4 .. 64; unit: unstated.


`godRaysIntensity` range: 0 .. unbounded; unit: unstated.


`godRaysDecay` range: 0.5 .. 1; unit: unstated.


`godRaysDensity` range: 0 .. unbounded; unit: unstated.


`godRaysSamples` range: 8 .. 128; unit: unstated.


`godRaysThreshold` range: 0 .. unbounded; unit: unstated.


`glowIntensity` range: 0 .. unbounded; unit: unstated.


`lutIntensity` range: 0 .. 1; unit: unstated.


`temperature` range: -1 .. 1; unit: unstated.


`tint` range: -1 .. 1; unit: unstated.


`brightness` range: -1 .. 1; unit: unstated.


`contrast` range: -1 .. 1; unit: unstated.


`hue` range: -180 .. 180; unit: deg.


`saturation` range: -1 .. 2; unit: unstated.


`vibrance` range: -1 .. 1; unit: unstated.


`grayscale` range: 0 .. 1; unit: unstated.


`invert` range: 0 .. 1; unit: unstated.


`threshold` range: 0 .. 1; unit: unstated.


`vignetteIntensity` range: 0 .. 1; unit: unstated.


`vignetteRadius` range: 0 .. 1; unit: unstated.


`vignetteSmoothness` range: 0 .. 1; unit: unstated.


`chromaticAberration` range: 0 .. 20; unit: unstated.


`lensDistortion` range: -1 .. 1; unit: unstated.


`filmGrain` range: 0 .. 1; unit: unstated.


`filmGrainSize` range: 0.5 .. unbounded; unit: unstated.


`sharpen` range: -1 .. 1; unit: unstated.


`shakeMaxOffset` range: 0 .. unbounded; unit: unstated.


`shakeMaxRoll` range: 0 .. unbounded; unit: deg.


`shakeFrequency` range: 0 .. unbounded; unit: hz.


`ssrMaxDistance` range: 0 .. unbounded; unit: m.


`ssrThickness` range: 0 .. unbounded; unit: m.


`ssrSteps` range: 4 .. 128; unit: unstated.


`ssrFade` range: 0 .. 1; unit: ratio.


`ssrIntensity` range: 0 .. 1; unit: unstated.


`ssrMaxRoughness` range: 0 .. 1; unit: ratio.


`ssrResolution` variants: full, half.


<a id="file-look-Stylise"></a>
### `file:look.Stylise`

| Field | Default | Meaning |
|---|---|---|
| `pixelHeight u32` | `0` | The logical resolution: the scene is rendered this many pixels tall and scaled up to the output by a whole number (pixelUpscale) with nearest filtering; 0 renders at the output's resolution. The width follows the output's aspect, at most the output's width over the scale. See `looks §7`. |
| `pixelUpscale u32` | `0` | The whole-number scale from the logical to the output resolution, a hard request: honoured whenever pixelHeight times it fits the output's height, else the largest that fits; 0 takes the largest that fits. A frame that does not fill the output is centred with Camera.background round it. See `looks §7`. |
| `pixelSnap bool` | `true` | Snap the camera and world sprites to the logical pixel grid, so a moving orthographic camera scrolls by whole pixels and sprites in a 3D world keep their edges (pixel sprites in 3D sets). |
| `posterize u32` | `0` | Tones per colour channel after tone mapping (2 to 64): smooth gradients become flat steps; 0 is off. |
| `posterizeMode PosterizeMode` | `"channel"` | How posterize steps a colour: each channel apart (channel), or its value and its tint apart so light never changes its hue (tone). |
| `palette Palette` | `0` | Quantise every pixel to the nearest colour of this palette (og.load.palette), measured in a perceptual space after posterize; 0 is none. |
| `dither DitherPattern` | `"none"` | How the steps of posterize and palette are mixed: none rounds every pixel, an ordered or blue-noise pattern alternates the two nearest steps in proportion. |
| `ditherStrength f32` | `1` | The dither's amplitude in steps: 1 spreads exactly one step, less keeps flatter bands. |
| `ditherScale u32` | `1` | The size of one dither cell in logical pixels. |
| `halftone f32` | `0` | Halftone print: tones become dots on a rotated grid, larger where the image is darker; 0 is off, 1 dots only. |
| `halftoneSize f32` | `6` | The dot grid's pitch in logical pixels. |
| `halftoneAngle f32` | `45` | The dot grid's angle. |
| `halftoneColor Color` | `{"r":0.02,"g":0.02,"b":0.03}` | The ink of the dots (linear). |
| `halftoneSpace StyliseSpace` | `"screen"` | Where the dots lie: screen, a grid over the frame; surface, grids on the surfaces that stay on them as the camera moves and wrap round forms (halftoneSize is then about the pitch in pixels where they lie); object, the same grids in each instance's own space, so a moving object carries its dots. |
| `hatching f32` | `0` | Pencil or ink cross-hatching from a tonal art map: darker tones add strokes in more directions over a paper-light ground, each set thinning and fading in from clean paper at its tone; 0 is off, 1 strokes only. |
| `hatchingScale f32` | `6` | The spacing of hatch lines in logical pixels. |
| `hatchingLevels u32` | `4` | How many tones the hatching distinguishes, each adding one stroke direction or density (1 to 6). |
| `hatchingAngle f32` | `45` | The direction of the first stroke set. |
| `hatchingColor Color` | `{"r":0.04,"g":0.035,"b":0.03}` | The stroke colour (linear): graphite, sepia or ink. |
| `hatchingPaper Color` | `{"r":1,"g":1,"b":1}` | The paper the strokes lie on (linear): one-ink hatching lifts the ground's light tones towards it, and under hatchingKeepColor the wash lies on the paper's colour at the wash's own lightness (a cream sketchbook, a grey card, a blue print); white keeps the frame's own. |
| `hatchingSpace StyliseSpace` | `"screen"` | Where the strokes lie: screen, stripes over the frame that follow the tone alone; surface, strokes on the surfaces that follow the tone and the form and stay on them as the camera moves (hatchingScale is then about the spacing in pixels where they lie); object, the same in each instance's own space. |
| `hatchingDepth f32` | `0` | Hatching from depth as well as luminance: the depth-normal prepass's silhouettes and creases (inkDepthThreshold's and inkNormalThreshold's edges) darken the tone the strokes follow at this strength, so strokes gather along contours and folds. 0 is the tone alone; above 0 the frame runs the prepass. See `looks §9`. |
| `hatchingDistance f32` | `0` | Aerial perspective: view depth in metres at which stroke width and ink strength halve, thinning exponentially into bare paper. 0 is off. Above 0 runs the prepass in every pattern space and leaves the sky bare. The wash and contour weighting stay unchanged. See `looks §9`. |
| `paintRadius f32` | `0` | Painterly smoothing (an anisotropic Kuwahara filter): flat regions and incoherent texture become strokes of one colour this many logical pixels across; edges and thin lines keep their shape; 0 is off. Under a 2D project's art contract the cook paints each texture and the frame draws neither pass. See `looks §10`. |
| `paintAnisotropy f32` | `0` | How far the strokes follow the image's structure: 0 round brush cells, 0.5 the filter's classic ellipses, 1 thin strokes along every coherent contour, which keep a thin line's shape. See `looks §10`. |
| `paintEdgeDarkening f32` | `0` | Pigment pooling at the edges of colour regions, as wet watercolour dries darker at its rim; 0 is none. |
| `paintBleed f32` | `0` | Colour bleeding across edges into its neighbours, wet in wet; 0 is none. |
| `paintStrokes f32` | `0` | Brush strokes in the paint, sized by paintRadius: marks along each stroke (the image's structure, or a slowly turning lay over flat regions), their own light and dark, and ragged edges where strokes meet. 0 is none; 1 bold, visible at a glance. See `looks §10`. |
| `paintCanvas Texture` | `0` | A tileable paper or canvas texture (og.load.texture) laid over the frame by paintCanvasStrength; 0 is none. |
| `paintCanvasStrength f32` | `0` | How much the canvas texture modulates the frame: 0 none, 1 its full grain. |
| `paintCanvasScale f32` | `1` | Canvas texels per logical pixel, with paintCanvasSpace screen (and on the sky). |
| `paintCanvasSpace StyliseSpace` | `"screen"` | Where the canvas lies: screen, over the frame (the scene slides under the paper as the camera moves); surface, on the surfaces, so the grain sticks to the scene, foreshortened with the floor and walls; object, in each instance's own space, so a moving object carries its grain. |
| `paintCanvasWorldScale f32` | `128` | Canvas texels per metre with paintCanvasSpace surface: 128 repeats a 256-texel canvas every two metres. |
| `ink bool` | `false` | Ink lines where the tone changes sharply (luminance edges), over every surface, sprite and texture alike: comic and storybook contours without PostEffects.outline's depth prepass. inkDepth adds the prepass's silhouettes and creases. |
| `inkThreshold f32` | `0.1` | The luminance step that counts as an edge (0 to 1 in display units): lower draws more lines. |
| `inkWidth f32` | `1` | The line width in logical pixels. |
| `inkColor Color` | `{"r":0.02,"g":0.02,"b":0.03}` | The line colour (linear). |
| `inkDepth f32` | `0` | With ink: lines also where the depth-normal prepass breaks (silhouettes against what is behind, creases between faces), at this strength: comic contours round flat colours that luminance alone misses. 0 is luminance only; above 0 the frame runs the prepass PostEffects.outline uses. |
| `inkDepthThreshold f32` | `0.04` | With inkDepth: the relative depth step to a farther neighbour that counts as a silhouette (0.04 is 4 %), as PostEffects.outlineDepthThreshold: lower inks smaller steps. |
| `inkNormalThreshold f32` | `0.4` | With inkDepth: the normal change between neighbours of about the same depth that counts as a crease (1 - cos of the angle; 0.4 is about 53 degrees), as PostEffects.outlineNormalThreshold: lower inks gentler folds. |
| `depthBands u32` | `0` | Colour bands by depth (2 to 8), interpolating depthBandsNearColor to depthBandsFarColor in lightness, chroma and hue. By default the layers divide depthBandsNear..depthBandsFar evenly; depthBandsFit instead fits logarithmic layers to visible geometry. 0 is off. See `looks §11`. |
| `depthBandsFit bool` | `false` | Fit logarithmic bands to the full visible non-sky depth range, measured each drawn frame. Nearer layers span less distance than farther ones. Ignores depthBandsNear and depthBandsFar; false keeps their absolute linear layers. The farthest layer still merges geometry within it. |
| `depthBandsNear f32` | `5` | Where the first band starts, unless depthBandsFit is true. |
| `depthBandsFar f32` | `60` | Where the last band ends; the far colour starts one band's depth before it and continues beyond it. Ignored when depthBandsFit is true. |
| `depthBandsNearColor Color` | `{"r":0.02,"g":0.02,"b":0.03}` | The nearest band's colour (linear, before exposure). |
| `depthBandsFarColor Color` | `{"r":0.55,"g":0.6,"b":0.7}` | The farthest band's colour (linear, before exposure). |
| `depthBandsFlatten f32` | `1` | How far a band replaces the surfaces' own shading: 1 flat silhouettes in the band's colour; 0 a tint over the lit scene, its colours times the band's at unit luminance (a coloured filter: the shading, the value and the surfaces' own hues show through); between, a mix of the two. |
| `vertexSnap f32` | `0` | Snap every vertex of the 3D scene to a screen grid this many logical pixels wide, as fixed-point hardware did: geometry jitters by whole steps as it moves; 0 is off. |
| `affineTexturing f32` | `0` | How far texture coordinates are interpolated without perspective correction: 1 fully affine, so textures warp and swim across large polygons as on early consoles; 0 is off. |
| `depthBits u32` | `0` | Quantise depth to this many bits, for the coarse depth buffers of early 3D (12 to 16 are period-like: distant surfaces fight); 0 keeps full precision. |
| `paintDetail f32` | `1` | How far detail keeps its shape under paintRadius: a pixel's radius shrinks by its detail (its material's StandardMaterialDesc.paintDetail, or paintCharacterDetail on a skinned mesh) times this, and no stroke crosses an object's edge. Above 0 the frame runs the prepass; 0 paints every pixel alike. See `looks §10`. |
| `paintCharacterDetail f32` | `0.8` | The detail of a skinned mesh's pixels (a character's face and hands) where its material sets less: 0 painted as the rest, 1 kept as drawn. |
| `watercolorEdgeDarkening f32` | `0` | The wash's dark rim, drawn after paintRadius's strokes: pigment pooled two to three pixels wide at a wash's outer boundary (a colour step wider than a line, or a silhouette against something farther), the colour there raised to one plus the strength. Seams and creases get none; 0 is none. See `looks §10`. |
| `watercolorGranulation f32` | `0` | Pigment settling into the paper's grain: a density from the paper's relief (paintCanvas, else a noise) thins each colour towards watercolorPaper on the peaks and thickens it in the pits, weighed by the pigment load so the paper and pale washes stay clean; the wash keeps its mean value. 0 is none. See `looks §10`. |
| `watercolorTurbulence f32` | `0` | Pigment drifting as the wash dries: the same density from a slow noise about a hundred canvas texels across, laid where the canvas lies (paintCanvasSpace) and seeded by the object under the pixel so no blotch spans two washes, weighed by the pigment load; 0 is none. See `looks §10`. |
| `watercolorDryBrush f32` | `0` | A dry brush skipping over the paper's tooth: its peaks (the relief past half its spread, in the paper's own clusters) left bare as the paper (watercolorPaper) in the rim at a wash's outer boundary (watercolorEdgeDarkening's) and on the light-catching tones, the brightest washes first; 0 is none. |
| `watercolorWobble f32` | `0` | The painted frame displaced by the paper's relief, up to this many logical pixels, so every edge wavers with the tooth as a wash's does (MNPR's substrate distortion); 0 is none. |
| `watercolorDilution f32` | `0` | The wash thinned with water: the pigment over watercolorPaper is this much less dense, so darks lift towards the paper, hues keep and the paper stays the paper. About 0.3 to 0.45 turns a lit scene's heavy darks into pale washes; 0 leaves the frame's tones as lit. See `looks §10`. |
| `watercolorPaper Color` | `{"r":1,"g":1,"b":1}` | The paper's colour (linear): what the wash thins towards and the dry brush leaves bare. |
| `ditherSpace StyliseSpace` | `"screen"` | Where the dither's cells lie: screen, a grid over the frame (it crawls as the camera moves); surface, cells on the surfaces at least ditherScale pixels on the screen each way, the sky's on a cube round the camera; object, in each mesh instance's own space. Off screen the frame runs the prepass. See `looks §8`. |
| `halftoneMode HalftoneMode` | `"ink"` | How the halftone inks: one ink (halftoneColor) or four-colour process print (cmyk). |
| `halftoneMisregistration f32` | `0` | With halftoneMode cmyk: how far the cyan, magenta and yellow plates sit off the black one, in logical pixels, as a cheap print's colour slips past its line art; 0 is in register. |
| `halftoneRegion HalftoneRegion` | `"all"` | Which tones the halftone prints on: all, the shade only or the highlights only (the darkest or lightest third of the tones). Under a region the dots are Ben-Day dots, a tint over the region's own colour at most 0.4 of a cell wide. They fade to flat ink on surfaces turned away from the camera. See `looks §9`. |
| `hatchingKeepColor bool` | `false` | Colour hatching: the ground is a light wash of its own colour and each stroke is that colour darkened towards hatchingColor, so hue and chroma stay under the strokes; false lays strokes of hatchingColor over a paper-light ground. See `looks §9`. |

`pixelHeight` range: 0 .. 2160; unit: unstated.


`pixelUpscale` range: 0 .. 16; unit: unstated.


`posterize` range: 0 .. 64; unit: unstated.


`posterizeMode` variants: channel, tone.


`dither` variants: none, bayer2, bayer4, bayer8, blueNoise, interleavedGradient.


`ditherStrength` range: 0 .. 2; unit: unstated.


`ditherScale` range: 1 .. 8; unit: unstated.


`halftone` range: 0 .. 1; unit: ratio.


`halftoneSize` range: 2 .. 64; unit: unstated.


`halftoneAngle` range: unbounded .. unbounded; unit: deg.


`halftoneSpace` variants: screen, surface, object.


`hatching` range: 0 .. 1; unit: ratio.


`hatchingScale` range: 2 .. 64; unit: unstated.


`hatchingLevels` range: 1 .. 6; unit: unstated.


`hatchingAngle` range: unbounded .. unbounded; unit: deg.


`hatchingSpace` variants: screen, surface, object.


`hatchingDepth` range: 0 .. 1; unit: ratio.


`hatchingDistance` range: 0 .. unbounded; unit: m.


`paintRadius` range: 0 .. 8; unit: unstated.


`paintAnisotropy` range: 0 .. 1; unit: ratio.


`paintEdgeDarkening` range: 0 .. 1; unit: ratio.


`paintBleed` range: 0 .. 1; unit: ratio.


`paintStrokes` range: 0 .. 1; unit: ratio.


`paintCanvasStrength` range: 0 .. 1; unit: ratio.


`paintCanvasScale` range: 0.125 .. 8; unit: unstated.


`paintCanvasSpace` variants: screen, surface, object.


`paintCanvasWorldScale` range: 1 .. 4096; unit: unstated.


`inkThreshold` range: 0 .. 1; unit: unstated.


`inkWidth` range: 0.5 .. 8; unit: unstated.


`inkDepth` range: 0 .. 1; unit: ratio.


`inkDepthThreshold` range: 0 .. unbounded; unit: unstated.


`inkNormalThreshold` range: 0 .. 2; unit: unstated.


`depthBands` range: 0 .. 8; unit: unstated.


`depthBandsNear` range: 0 .. unbounded; unit: m.


`depthBandsFar` range: 0 .. unbounded; unit: m.


`depthBandsFlatten` range: 0 .. 1; unit: ratio.


`vertexSnap` range: 0 .. 16; unit: unstated.


`affineTexturing` range: 0 .. 1; unit: ratio.


`depthBits` range: 0 .. 24; unit: unstated.


`paintDetail` range: 0 .. 1; unit: ratio.


`paintCharacterDetail` range: 0 .. 1; unit: ratio.


`watercolorEdgeDarkening` range: 0 .. 1; unit: ratio.


`watercolorGranulation` range: 0 .. 1; unit: ratio.


`watercolorTurbulence` range: 0 .. 1; unit: ratio.


`watercolorDryBrush` range: 0 .. 1; unit: ratio.


`watercolorWobble` range: 0 .. 4; unit: unstated.


`watercolorDilution` range: 0 .. 0.9; unit: ratio.


`ditherSpace` variants: screen, surface, object.


`halftoneMode` variants: ink, cmyk.


`halftoneMisregistration` range: 0 .. 8; unit: unstated.


`halftoneRegion` variants: all, shade, highlight.


<a id="file-look-MaterialStyle"></a>
### `file:look.MaterialStyle`

| Field | Default | Meaning |
|---|---|---|
| `toon bool` | `false` | Cel shading for every lit material, as StandardMaterialDesc.toon: light quantised by the ramp, hard highlights. A material with a physical lobe (clearcoat, transmission and the rest) keeps its physical shading; set its own toon to make it toon anyway. |
| `gradientMap Texture` | `0` | The toon ramp for materials without their own (StandardMaterialDesc.gradientMap, og.load.texture): its first row maps the light's angle to the light received, read without filtering; 0 is three built-in bands. |
| `shadowTint Color` | `{"r":0.18,"g":0.16,"b":0.32}` | The colour the shadowed side of a toon or lit surface moves towards instead of black (linear): the cool or warm shadow of illustration. |
| `shadowTintAmount f32` | `0` | How far shadowed light moves towards shadowTint; 0 is off. |
| `shadowDarkness f32` | `0` | How far shadowed surfaces fall towards black: where the sun does not reach, a surface loses this share of the sun's ramp and of the ambient light (1: solid black shadows). Point, spot and rect lights and the emissive still show inside it; shadowTint's light is added after it. 0 is off. See `looks §12`. |
| `rimColor Color` | `{"r":1,"g":1,"b":1}` | Rim (fresnel) light for materials without their own (linear). |
| `rimIntensity f32` | `0` | The rim light's strength; 0 is none. |
| `rimPower f32` | `3` | How tight the rim is (higher: thinner). |
| `outlineColor Color` | `{"r":0.02,"g":0.02,"b":0.03}` | The outline colour for materials without their own (linear). |
| `outlineWidth f32` | `0` | The outline width in pixels at outlineDistance for materials without their own; 0 is none. A hull outline (StandardMaterialDesc.outlineWidth): silhouettes and creases against other objects, never the faces of one. A surface seen at a grazing angle is lined only where it ends. See `looks §12`. |
| `outlineDistance f32` | `0` | Distance in metres at which outlines have their base width. Nearer they grow up to outlineNearCap times that width (default 4); farther they thin proportionally. 0 keeps a constant width. Foliage never grows beyond its base width and fades below a pixel. |
| `outlineNearCap f32` | `4` | Near-camera width limit as a multiple of the base width, with outlineDistance > 0. Default 4 preserves existing growth; 1 caps near lines at base width while retaining far falloff. Also applies to material and ArtLine outlines. Foliage and orthographic cameras keep their existing rules. |
| `flatShading bool` | `false` | Shade every lit material with face normals, as StandardMaterialDesc.flatShading: visible facets for low-poly, paper-craft and voxel looks. |
| `surface MaterialSurface` | `"none"` | A made surface on every lit material (clay, paper, plastic): its normal detail, colour grain and light response, in each mesh's own space; none is no change. Unlit materials, materials with StandardMaterialDesc.styled false and those with a MeshPhysicalMaterial lobe keep their own shading. |
| `surfaceStrength f32` | `1` | How strongly the surface shows: its bumps, grain and wrapped light; 0 is none. |
| `surfaceScale f32` | `0` | The size of the surface's pattern in metres of the mesh's own space (a clay lump, the paper's fibre); 0 takes the surface's own (clay 0.12, paper 0.15, plastic 0.3). |
| `toonAmbient f32` | `0` | How smooth a toon material's ambient light (the Environment and its hemisphere light) and reflection stay: 0 one flat colour, the light from above and below averaged (MToon's equalised ambient), so the bands sit on flat colour; 1 the smooth PBR ambient and reflections. See `looks §12`. |
| `toonSoftness f32` | `1` | The toon bands' and highlight's edges, anti-aliased over this many pixels (lilToon's screen-space smoothing); 0 hard steps that only TAA softens. |
| `outlinePlace OutlinePlace` | `"inside"` | Where outlines lie against an object's silhouette: inside (the zero value, its own edge pixels), outside (round it, at any width, anti-aliased) or both. Creases between objects stay inside. |
| `wireframe bool` | `false` | Draw every lit material's feature edges over its faces (StandardMaterialDesc.wireframe's; this style's angle and colour where it sets none), in the colour the surface shows along each edge or wireframeColor; the faces hide the edges behind them. A material's own wireframe draws its edges alone. See `looks §12`. |
| `wireframeAngle f32` | `30` | The crease angle wireframe draws past (StandardMaterialDesc.wireframeAngle); 0 every triangle edge. |
| `wireframeColor Color` | `{"r":0,"g":0,"b":0}` | The look's line colour for wireframe (StandardMaterialDesc.wireframeColor), linear, above 1 blooms; black (the zero value) draws each edge in the colour its surface shows there. |

`shadowTintAmount` range: 0 .. 1; unit: ratio.


`shadowDarkness` range: 0 .. 1; unit: ratio.


`rimIntensity` range: 0 .. unbounded; unit: unstated.


`rimPower` range: 0.1 .. unbounded; unit: unstated.


`outlineWidth` range: 0 .. 31; unit: unstated.


`outlineDistance` range: 0 .. unbounded; unit: m.


`outlineNearCap` range: 1 .. 4; unit: ratio.


`surface` variants: none, clay, paper, plastic.


`surfaceStrength` range: 0 .. 1; unit: ratio.


`surfaceScale` range: 0 .. unbounded; unit: m.


`toonAmbient` range: 0 .. 1; unit: ratio.


`toonSoftness` range: 0 .. 4; unit: unstated.


`outlinePlace` variants: inside, outside, both.


`wireframeAngle` range: 0 .. 180; unit: deg.


<a id="file-look-Fog"></a>
### `file:look.Fog`

| Field | Default | Meaning |
|---|---|---|
| `enabled bool` | `false` | Draw the fog. |
| `color Color` | `{"r":0.62,"g":0.68,"b":0.76}` | The colour the fog tends to (linear, before exposure). |
| `density f32` | `0.02` | Exponential distance fog per metre past start, with or without end's ramp (0: none). |
| `start f32` | `0` | The distance from the camera where distance fog starts, also with volumetric on. Height fog and FogVolume are independent of start. |
| `end f32` | `0` | The distance from the camera where fog is full: with end > start, a linear ramp from start to end applied together with density's exponential fog (0: no ramp). |
| `heightDensity f32` | `0` | Height fog at and below height (0: none). |
| `height f32` | `0` | The world height below which height fog is densest. |
| `falloff f32` | `0.5` | How fast height fog thins above height, per metre. |
| `sunColor Color` | `{"r":1,"g":0.86,"b":0.62}` | Plain fog's colour looking into the sun. Ignored by volumetric fog, which scatters the actual lights' colours; anisotropy controls its angular spread. |
| `sunPower f32` | `8` | How tight plain fog's sun-colour glow is (higher: smaller). Ignored by volumetric fog: use anisotropy for its Henyey-Greenstein angular spread. |
| `maxOpacity f32` | `1` | The most the global distance and height fog together hide, also with volumetric on. Does not limit independent FogVolume media. |
| `sky f32` | `1` | How much fog covers the sky and clouds (0: clear). Below a cumulus base, or without cumulus, the lower sky stands for distant land and takes land fog. Above a cumulus base it is cloud sea and keeps this share. |
| `volumetric bool` | `false` | Light the fog as a medium: unlit, it tends to color as the plain fog does; over that it (density, thickened below height) scatters the sun through its cascades and the point, spot and rect lights (up to RenderCapabilities.volumetricLights a sample). Blended surfaces and sprites take the plain fog. See `render §7`. |
| `scattering f32` | `0.8` | How much of the sun's and the lamps' light the fog scatters (its albedo, tinted by color): 0 leaves the plain fog's haze without shafts or glow, 1 sends all of that light on, above 1 brightens the shafts for effect. |
| `anisotropy f32` | `0.6` | Which way the fog scatters (Henyey–Greenstein g): 0 evenly, towards 1 forward (bright looking into the light: shafts against the sun), below 0 back towards the light. |
| `volumetricDistance f32` | `48` | How far from the camera the fog shows shafts and the local lights; past it the fog goes on in its colour, with the sun's light in it unshadowed. |

`density` range: 0 .. 2; unit: unstated.


`start` range: 0 .. unbounded; unit: m.


`end` range: 0 .. unbounded; unit: m.


`heightDensity` range: 0 .. 10; unit: unstated.


`height` range: unbounded .. unbounded; unit: m.


`falloff` range: 0.0001 .. 20; unit: unstated.


`sunPower` range: 0 .. 256; unit: unstated.


`maxOpacity` range: 0 .. 1; unit: ratio.


`sky` range: 0 .. 1; unit: ratio.


`scattering` range: 0 .. 4; unit: unstated.


`anisotropy` range: -0.95 .. 0.95; unit: unstated.


`volumetricDistance` range: 4 .. 400; unit: m.


<a id="file-look-Sky"></a>
### `file:look.Sky`

| Field | Default | Meaning |
|---|---|---|
| `enabled bool` | `false` | Draw and light by this sky. |
| `zenith Color` | `{"r":0.18,"g":0.36,"b":0.78}` | Overhead. |
| `horizon Color` | `{"r":0.78,"g":0.84,"b":0.92}` | At the horizon. |
| `ground Color` | `{"r":0.16,"g":0.14,"b":0.12}` | Below the horizon. |
| `sunDirection Vector3` | `{"x":0,"y":0,"z":0}` | Towards the sun: the opposite of the way its light shines. Zero (the default) follows the frame's light: towards the DirectionalLight drawn (its entity's +Z; it shines down -Z; the brightest of several), or the built-in sun's (0.45, 0.8, 0.35) with none. A non-zero direction is used as given. |
| `sunColor Color` | `{"r":1,"g":0.95,"b":0.88}` | The sun's colour. |
| `sunSize f32` | `1.8` | The sun disc's angular radius (0: no disc). |
| `sunIntensity f32` | `40` | The disc's brightness. |
| `glow f32` | `0.6` | The glow around the sun. |
| `glowPower f32` | `24` | How tight the glow is. |
| `stars f32` | `0` | Stars where the sky is dark: 0 none, 1 a full night sky. |
| `brightness f32` | `1.1` | A multiplier on the whole sky. |
| `moonSize f32` | `0` | The moon disc's angular radius (0: no moon). It sets below the horizon and hides the stars behind it. |
| `moonPhase f32` | `0.5` | How much of the moon is lit: 0 new, 0.25 the first quarter (its right half), 0.5 full, 0.75 the last quarter (its left half). |
| `moonIntensity f32` | `1.4` | The lit moon's brightness. |
| `moonColor Color` | `{"r":0.9,"g":0.92,"b":1}` | The moon's colour. |
| `moonDirection Vector3` | `{"x":0,"y":0,"z":0}` | Towards the moon; zero puts it opposite sunDirection. |

`sunSize` range: 0 .. 20; unit: deg.


`sunIntensity` range: 0 .. unbounded; unit: unstated.


`glow` range: 0 .. unbounded; unit: unstated.


`glowPower` range: 1 .. unbounded; unit: unstated.


`stars` range: 0 .. 1; unit: ratio.


`brightness` range: 0 .. unbounded; unit: unstated.


`moonSize` range: 0 .. 20; unit: deg.


`moonPhase` range: 0 .. 1; unit: ratio.


`moonIntensity` range: 0 .. unbounded; unit: unstated.


<a id="file-look-Environment"></a>
### `file:look.Environment`

| Field | Default | Meaning |
|---|---|---|
| `map EnvironmentMap` | `0` | The map from og.load.environment; 0 lights by the sky. |
| `intensity f32` | `1` | The strength of the map's light on every surface, as three.js scene.environmentIntensity; while a map is set it replaces RenderSettings.environmentIntensity. |
| `rotation f32` | `0` | Turns the map about +Y, as three.js scene.environmentRotation.y: positive turns what was at +X towards -Z. The light and the background turn together. |
| `background bool` | `false` | Draw the map behind the scene, as three.js scene.background = the environment (Camera.background, og.Sky and RenderSettings.sky give way); false leaves the background as they set it. |
| `backgroundIntensity f32` | `1` | Multiplies the map where it is drawn behind the scene, as three.js scene.backgroundIntensity; the light is not changed. |
| `backgroundBlurriness f32` | `0` | Blurs only the background, as three.js scene.backgroundBlurriness: 0 sharp to 1 the coarsest mip. HDR and physical skies use GGX levels; authored LDR skies use their own source-resolution box mips. The light is not changed. |
| `groundHeight f32` | `0` | Draws the sky on a dome with a flat ground under it, as three.js GroundedSkybox: how far above the ground the map was photographed (metres). Objects stand on the map's ground, and the environment's reflections show it too. 0 (the default) draws the sky at infinity. See `render §4`. |
| `groundRadius f32` | `100` | The grounded dome's radius (metres, larger than groundHeight), as GroundedSkybox's radius: the ground looks flat out to about this distance. |
| `groundCenter Vector3` | `{"x":0,"y":0,"z":0}` | The ground point under where the map was photographed: the dome's centre is groundHeight above it. |
| `groundShadows f32` | `1` | How dark the grounded ground (groundHeight above 0) takes the shadows of what stands on it, the sun's (a DirectionalLight with castShadow) and the map's light they hide, so props sit on it: 1 as dark as a real floor, 0 none (the ground as photographed). |
| `blendTo EnvironmentMap` | `0` | A second map from og.load.environment that the light and the background blend towards (indoor to outdoor); 0 is none. Both maps turn by rotation and the background is scaled by backgroundIntensity. |
| `blendWeight f32` | `0` | How far the blend has gone: 0 the map, 1 blendTo, in between a mix of both maps' light (every GGX level and the harmonics) and background. Drive it from the tick (a tween or a system), never from a clock, so a replay blends alike; it is drawn in 128 steps, each regenerating the light once. |
| `hemiSky Color` | `{"r":1,"g":1,"b":1}` | A hemisphere light's sky colour (linear), as three.js HemisphereLight.color: diffuse-only ambient light mix(hemiGround, hemiSky, 0.5 n.y + 0.5) * hemiIntensity, darkened by ambient occlusion like the map's light. Unlike light baked into the map it adds no reflection. |
| `hemiGround Color` | `{"r":0,"g":0,"b":0}` | The hemisphere light's ground colour (linear), as three.js HemisphereLight.groundColor. |
| `hemiIntensity f32` | `0` | The hemisphere light's intensity, as three.js HemisphereLight.intensity; 0 (the default) is off. |

`intensity` range: 0 .. unbounded; unit: unstated.


`rotation` range: unbounded .. unbounded; unit: rad.


`backgroundIntensity` range: 0 .. unbounded; unit: unstated.


`backgroundBlurriness` range: 0 .. 1; unit: unstated.


`groundHeight` range: 0 .. unbounded; unit: m.


`groundRadius` range: 0 .. unbounded; unit: m.


`groundCenter` range: unbounded .. unbounded; unit: m.


`groundShadows` range: 0 .. 1; unit: unstated.


`blendWeight` range: 0 .. 1; unit: unstated.


`hemiIntensity` range: 0 .. unbounded; unit: unstated.


<a id="file-look-ArtPalette"></a>
### `file:look.ArtPalette`

| Field | Default | Meaning |
|---|---|---|
| `colors Palette` | `0` | The palette (og.load.palette of a `.hex` or GIMP `.gpl` file; a look names one under assets/ or a built-in `@palettes/<name>`); 0 is none, any colour. |
| `strict bool` | `false` | Every opaque texel is exactly a colour of colors (indexed colour, as hand-placed pixel art); false lets texels between them (shading, filtering) and holds only the hues to the palette. |
| `maxColors u32` | `0` | The most distinct colours one asset (a sprite sheet, a texture, a model's colours) may use; 0 is no limit. |
| `rampSteps u32` | `0` | The most shades one hue ramps through from its shadow to its highlight (3 or 4 for 8-bit art, 4 to 6 for 16-bit); 0 is no limit. |
| `accents u32` | `[]` | The colours of colors kept for signals (the player, danger, the goal, a pickup), by their index in the palette (0 its first colour): scenery and backgrounds stay off them. A look gives them on one line (`accents 3 7`); empty keeps none. |
| `dither DitherPattern` | `"none"` | How an asset mixes the two nearest colours where a tone falls between them: none in flat bands (the zero value), or an ordered or noise pattern as Stylise.dither. |

`maxColors` range: 0 .. 256; unit: unstated.


`rampSteps` range: 0 .. 32; unit: unstated.


`dither` variants: none, bayer2, bayer4, bayer8, blueNoise, interleavedGradient.


<a id="file-look-ArtPixels"></a>
### `file:look.ArtPixels`

| Field | Default | Meaning |
|---|---|---|
| `unit f32` | `0` | Logical pixels per metre (per world unit in 2D): one pixel of every sprite, tile and backdrop covers 1/unit metres, so no two assets show pixels of different sizes (16 with 16-pixel tiles one metre wide); 0 is no constraint. |
| `logicalHeight u32` | `0` | The rows of the logical resolution the art is drawn for, as Stylise.pixelHeight counts them (180, 224, 240); 0 is no constraint. |
| `spriteSizes u32` | `[]` | The cell sizes, in logical pixels, a sprite, a flipbook frame or a tile may have: each side is one of them (`spriteSizes 16 32` takes 16 x 16, 16 x 32 and 32 x 32). A look gives them on one line; empty is any size. |
| `rotation ArtRotation` | `"free"` | How pixel art turns: free (no constraint), nearest, cleanEdge, or prerotated in rotationSteps. |
| `rotationSteps u32` | `0` | With rotation prerotated: the rotations baked per full turn (8 is one every 45 degrees); 0 is none. |
| `snap bool` | `false` | Sprites and tiles stand on whole logical pixels: their positions are multiples of 1/unit metres, so no pixel of the art lies between two of the grid. |

`unit` range: 0 .. 4096; unit: unstated.


`logicalHeight` range: 0 .. 2160; unit: unstated.


`rotation` variants: free, nearest, cleanEdge, prerotated.


`rotationSteps` range: 0 .. 64; unit: unstated.


<a id="file-look-ArtTexels"></a>
### `file:look.ArtTexels`

| Field | Default | Meaning |
|---|---|---|
| `density f32` | `0` | Texels per metre on the surfaces of 3D models (a 2 m character at 64 texels per metre takes a 128-texel-tall texture); 0 is no constraint. |
| `maxSize u32` | `0` | The longest side of any texture of the style, in texels (256 for low-resolution 3D); 0 is no limit. |
| `filter ArtFilter` | `"any"` | How the style's textures are sampled: any (no constraint), nearest or linear. |
| `mips ArtMips` | `"any"` | Their mip maps: any (no constraint), full or none. |

`density` range: 0 .. 16384; unit: unstated.


`maxSize` range: 0 .. 16384; unit: unstated.


`filter` variants: any, nearest, linear.


`mips` variants: any, full, none.


<a id="file-look-ArtShape"></a>
### `file:look.ArtShape`

| Field | Default | Meaning |
|---|---|---|
| `propFaces u32` | `0` | The most triangles one prop may have (50 to 500 for faceted low poly); 0 is no budget. An effect's mesh (`conform effect` in its import settings, as og vfx import writes) has none. |
| `characterFaces u32` | `0` | The most triangles one character may have; 0 is no budget. |
| `normals ArtNormals` | `"any"` | How models are shaded across their faces: any (no constraint), smooth, flat, or angle (split past normalAngle). |
| `normalAngle f32` | `30` | With normals angle: the angle between two faces past which their edge is shaded hard. |
| `bevel f32` | `0` | Hard edges rounded by a chamfer of this radius (clay, toys, diorama pieces); 0 is none. |
| `facetAngle f32` | `0` | With normals flat: neighbouring faces within this angle of their facet's normal shade as one facet, one normal and one baked colour, so a gently curved ground is a few broad planes, not a pattern of triangles. 0 is every triangle its own facet. |
| `voxel f32` | `0` | Models built of cubes of this edge (voxel worlds); 0 is no voxel grid. |
| `silhouette f32` | `0` | The thinnest feature a model's silhouette keeps at the style's scale: thinner detail does not read and is left out; 0 is no constraint. |

`propFaces` range: 0 .. unbounded; unit: unstated.


`characterFaces` range: 0 .. unbounded; unit: unstated.


`normals` variants: any, smooth, flat, angle.


`normalAngle` range: 0 .. 180; unit: deg.


`bevel` range: 0 .. unbounded; unit: m.


`facetAngle` range: 0 .. 90; unit: deg.


`voxel` range: 0 .. unbounded; unit: m.


`silhouette` range: 0 .. unbounded; unit: m.


<a id="file-look-ArtLine"></a>
### `file:look.ArtLine`

| Field | Default | Meaning |
|---|---|---|
| `width f32` | `0` | The line's width: in logical pixels round a sprite (1 is one dark pixel round the shape), in pixels at distanceFalloff round a mesh (as MaterialStyle.outlineWidth); 0 is no line. |
| `color Color` | `{"r":0.02,"g":0.02,"b":0.03}` | The line's colour (linear; a look may write an sRGB "#rrggbb"). |
| `place ArtLinePlace` | `"outside"` | Where the line lies against the shape: outside (the zero value), inside, or both. |
| `distanceFalloff f32` | `0` | Round a mesh: the distance at which the line is width wide, nearer wider and farther thinner (as MaterialStyle.outlineDistance); 0 keeps one width. |
| `brush ArtBrush` | `"diamond"` | Round a sprite: the brush the cook draws the line into its cells with: diamond (the zero value), circle or square (docs/art.md §2.3). |

`width` range: 0 .. 32; unit: unstated.


`place` variants: outside, inside, both.


`distanceFalloff` range: 0 .. unbounded; unit: m.


`brush` variants: diamond, circle, square.


<a id="file-look-ArtMotion"></a>
### `file:look.ArtMotion`

| Field | Default | Meaning |
|---|---|---|
| `stepFps f32` | `0` | Clips and flipbooks advance this many times a second whatever the tick, holding each pose between (12 is on twos at 24 fps, 8 on threes); 0 is smooth, no constraint. |
| `maxFrames u32` | `0` | The most frames one clip or flipbook may hold; 0 is no limit. |
| `squash f32` | `0` | The default squash and stretch of the style's movement: og.tween.squash's impact where a call leaves it out, 0.5 the preset's own (1.25, 0.8), 1 as rubber hose (1.5, 0.6); 0 is no constraint (the preset's own). |

`stepFps` range: 0 .. 120; unit: hz.


`maxFrames` range: 0 .. unbounded; unit: unstated.


`squash` range: 0 .. 1; unit: ratio.


<a id="file-look-ArtCamera"></a>
### `file:look.ArtCamera`

| Field | Default | Meaning |
|---|---|---|
| `projection ArtProjection` | `"any"` | The projection: any (no constraint), perspective or orthographic. |
| `fixedAngle bool` | `false` | The camera keeps one angle, pitch and yaw: false is no constraint. |
| `pitch f32` | `0` | With fixedAngle: how far below the horizon the camera looks (0 side-on, 90 straight down, 30 the 2:1 pixel isometric, 35.264 true isometric). |
| `yaw f32` | `0` | With fixedAngle: its turn about the vertical axis (45 for isometric). |
| `fovMin f32` | `0` | The narrowest vertical field of view a perspective camera of the style takes; 0 is no limit. |
| `fovMax f32` | `0` | The widest vertical field of view a perspective camera of the style takes; 0 is no limit. |
| `pixelPerfect bool` | `false` | A 2D camera moves and zooms by whole logical pixels only (ArtPixels.unit), as Stylise.pixelSnap does the frame's: pixel art never shimmers as it scrolls. |

`projection` variants: any, perspective, orthographic.


`pitch` range: -90 .. 90; unit: deg.


`yaw` range: -180 .. 180; unit: deg.


`fovMin` range: 0 .. 179; unit: deg.


`fovMax` range: 0 .. 179; unit: deg.


<a id="file-materials"></a>
### `file:materials`

An authored `materials` asset, cooked by `materials`. Input: `.ogt` under assets/.

Field coverage: library parameters and shared model material fields. Repeat material "<name>" blocks, each with at least one label. Images and map destinations are asset-root paths. og materials brief lists authored requests; og materials make processes supplied .source.png images, never calls an image model.


<a id="file-materials-material"></a>
### `file:materials.material`

| Field | Default | Meaning |
|---|---|---|
| `generate { source "<image>" }` | `off` | Prepare the declared map destinations during cook, without writing generated files into assets/. |
| `label <label>` | `required` | Repeat labels this material covers. |
| `weight <number > 0>` | `1` | Explicit selection weight within a label. |
| `tile <width> <height>` | `1 1` | Positive texture coverage in metres. |
| `vary { ... }` | `` | Per-building value and warmth variation. |
| `brief "<text>"` | `` | Author's image request; the engine does not generate an image model's response. |
| `facade true \| false` | `false` | Facade source repeats horizontally; requires brief. |
| `relief <-1..1 metres>` | `0` | Height range for the derived normal map; nonzero needs brief and normalMap. |
| `roughnessRange <black> <white>` | `` | Roughness endpoints 0..1; requires brief and metalnessMap. |
| `occlusion <strength> <radius>` | `` | Strength 0..1, whole radius 1..64 texels; requires brief, relief and aoMap. |
| `color <r> <g> <b>` | `1 1 1` | Linear base colour. |
| `opacity <0..1>` | `1` | Base alpha; below one enables blending unless alphaTest is present. |
| `metalness <0..1>` | `0` | Metallic factor. |
| `roughness <0..1>` | `1` | Roughness factor. |
| `emissive <r> <g> <b>` | `0 0 0` | Linear emitted colour. |
| `emissiveIntensity <number >= 0>` | `1` | Multiply emissive, preserving values above one through the emissive-strength extension. |
| `doubleSided true \| false` | `false` | Draw both faces. |
| `alphaTest <0..1>` | `` | Alpha mask cutoff; takes precedence over opacity blending. |
| `map "<image>"` | `` | Base colour image relative to the model, not an asset handle. |
| `normalMap "<image>"` | `` | Normal image relative to the model. |
| `normalScale <number>` | `1` | Scale normal-map strength. |
| `metalnessMap "<image>"` | `` | Packed roughness (green) and metalness (blue). |
| `aoMap "<image>"` | `` | Occlusion in red. |
| `transmission <0..1>` | `0` | Specular transmission. |
| `ior <1..3>` | `` | Explicit index of refraction. |
| `thickness <metres >= 0>` | `0` | Volume thickness when transmission is nonzero. |
| `clearcoat <0..1>` | `0` | Clearcoat weight. |
| `clearcoatRoughness <0..1>` | `0` | Clearcoat roughness when clearcoat is nonzero. |
| `translucency <0..1>` | `0` | Diffuse transmission; foliage draws both faces. |
| `domeNormals <0..1>` | `0` | Foliage normals' canopy-dome blend. |
| `season <0..1>` | `0` | Evergreen at zero, deciduous at one. |
| `autumnColor <r> <g> <b>` | `0.42 0.12 0.015` | Nonnegative linear seasonal colour. |
| `autumnColor2 <r> <g> <b>` | `0.25 0.025 0.015` | Second seasonal colour. |
| `seasonShift <days>` | `0` | Shift the species' seasonal cycle within LeafSeason.MAX_SHIFT. |
| `grime <0..1>` | `0` | Ground grime weight. |
| `grimeColor <r> <g> <b>` | `0 0 0` | Nonnegative linear grime colour. |
| `grimeHeight <metres >= 0>` | `0` | Ground grime extent. |
| `stains <0..1>` | `0` | Stain variation weight. |
| `stochastic <0..1>` | `0` | Stochastic texture sampling weight. |

<a id="file-materials-material-generate"></a>
### `file:materials.material.generate`

| Field | Default | Meaning |
|---|---|---|
| `source "<image>"` | `required` | Authored input image, relative to assets/. Its bytes and processing parameters form the action key. |

<a id="file-materials-material-vary"></a>
### `file:materials.material.vary`

| Field | Default | Meaning |
|---|---|---|
| `value <0..0.5>` | `0` | Brightness variation either way. |
| `warmth <0..0.5>` | `0` | Warmth variation either way. |

<a id="file-model"></a>
### `file:model`

An authored `model` asset, cooked by `model`. Input: `.ogt` under assets/.

Field coverage: text-model declarations, every geometry, placement, material, plant level and fracture block share parser line tables. Output: a normal model loaded with og.load.gltf. Program-based models instead use program/params and the .shape.json language (see model); program operation schema is separate. Segment counts can be reduced by the project's face budget, never increased. A part contains one geometry; merge parts and CSG a/b are geometries plus placement, not named model parts.

| Field | Default | Meaning |
|---|---|---|
| `material "<name>" { ... }` | `` | Named material declared before its uses. |
| `pivot "<name>" { ... }` | `` | Named TRS node; parent must precede child. |
| `part "<name>" { ... }` | `` | Placed mesh; one geometry block, with optional instanced at placements. |
| `plant "<name>" { ... }` | `` | One generated plant per model. |
| `fracture { ... }` | `` | Prefracture the imported model through the shared cutting kernel. |
| `process { ... }` | `` | Explicit static-mesh repair, simplification, UVs, material transfer and skin binding before prefracture; see model.process. |
| `conform off \| character \| effect` | `project art contract` | Art conformance category. |

<a id="file-model-program"></a>
### `file:model.program`

| Field | Default | Meaning |
|---|---|---|
| `program "<file.shape.json>"` | `required` | Model program beside this model. |
| `params { <name> <finite number> ... }` | `program parameters` | Named numeric parameter overrides. |
| `material "<name>" { ... }` | `program materials` | Material overrides by name. |
| `atlas <integer 16..8192>` | `none` | Repack UVs at this texel grid, gutter 4. |
| `conform off \| character \| effect` | `project look` | Art conformance category. |
| `process { ... }` | `off` | Shared static model preparation. |

<a id="file-model-program-material"></a>
### `file:model.program.material`

| Field | Default | Meaning |
|---|---|---|
| `color <r> <g> <b>; linear` | `program material` | Override the named program material; source image contents are tracked. |
| `metalness <number 0..1>` | `program material` | Override the named program material; source image contents are tracked. |
| `roughness <number 0..1>` | `program material` | Override the named program material; source image contents are tracked. |
| `opacity <number 0..1>` | `program material` | Override the named program material; source image contents are tracked. |
| `emissive <r> <g> <b>; linear` | `program material` | Override the named program material; source image contents are tracked. |
| `emissiveIntensity <number >= 0>` | `program material` | Override the named program material; source image contents are tracked. |
| `map "<image path>"; relative to model` | `program material` | Override the named program material; source image contents are tracked. |
| `normalMap "<image path>"; relative to model` | `program material` | Override the named program material; source image contents are tracked. |
| `normalScale <number >= 0>` | `program material` | Override the named program material; source image contents are tracked. |
| `roughnessMap "<image path>"; relative to model` | `program material` | Override the named program material; source image contents are tracked. |
| `metalnessMap "<image path>"; relative to model` | `program material` | Override the named program material; source image contents are tracked. |
| `emissiveMap "<image path>"; relative to model` | `program material` | Override the named program material; source image contents are tracked. |
| `aoMap "<image path>"; relative to model` | `program material` | Override the named program material; source image contents are tracked. |
| `aoMapIntensity <number 0..1>` | `program material` | Override the named program material; source image contents are tracked. |
| `clearcoat <number 0..1>` | `program material` | Override the named program material; source image contents are tracked. |
| `clearcoatRoughness <number 0..1>` | `program material` | Override the named program material; source image contents are tracked. |
| `sheen <r> <g> <b>; linear` | `program material` | Override the named program material; source image contents are tracked. |
| `sheenRoughness <number 0..1>` | `program material` | Override the named program material; source image contents are tracked. |
| `transmission <number 0..1>` | `program material` | Override the named program material; source image contents are tracked. |
| `iridescence <number 0..1>` | `program material` | Override the named program material; source image contents are tracked. |
| `alphaTest <number 0..1>` | `program material` | Override the named program material; source image contents are tracked. |
| `doubleSided true \| false` | `program material` | Override the named program material; source image contents are tracked. |

<a id="file-model-part-boxGeometry"></a>
### `file:model.part.boxGeometry`

| Field | Default | Meaning |
|---|---|---|
| `width <metres >= 0>` | `1` | X extent. |
| `height <metres >= 0>` | `1` | Y extent. |
| `depth <metres >= 0>` | `1` | Z extent. |
| `physics true \| false` | `true` | Accepted runtime descriptor field; model import settings own cooked collision. |
| `dynamic true \| false` | `false` | Accepted runtime descriptor field; a cooked geometry is static source data. |
| `uvRepeat <u> <v>` | `1 1` | Multiply source UVs. |

<a id="file-model-part-planeGeometry"></a>
### `file:model.part.planeGeometry`

| Field | Default | Meaning |
|---|---|---|
| `width <metres >= 0>` | `1` | X extent. |
| `height <metres >= 0>` | `1` | Y extent; plane faces +Z. |
| `physics true \| false` | `true` | Accepted runtime descriptor field; model import settings own cooked collision. |
| `dynamic true \| false` | `false` | Accepted runtime descriptor field; a cooked geometry is static source data. |
| `uvRepeat <u> <v>` | `1 1` | Multiply source UVs. |

<a id="file-model-part-sphereGeometry"></a>
### `file:model.part.sphereGeometry`

| Field | Default | Meaning |
|---|---|---|
| `radius <metres >= 0>` | `1` | Sphere radius. |
| `widthSegments <3..512>` | `32` | Longitudinal segments. |
| `heightSegments <2..512>` | `16` | Latitudinal segments. |
| `physics true \| false` | `true` | Accepted runtime descriptor field; model import settings own cooked collision. |
| `dynamic true \| false` | `false` | Accepted runtime descriptor field; a cooked geometry is static source data. |
| `uvRepeat <u> <v>` | `1 1` | Multiply source UVs. |

<a id="file-model-part-cylinderGeometry"></a>
### `file:model.part.cylinderGeometry`

| Field | Default | Meaning |
|---|---|---|
| `radiusTop <metres >= 0>` | `1` | Top radius. |
| `radiusBottom <metres >= 0>` | `1` | Bottom radius. |
| `height <metres >= 0>` | `1` | Y extent. |
| `radialSegments <3..65536>` | `32` | Segments around the axis. |
| `heightSegments <1..65536>` | `1` | Segments along the axis. |
| `openEnded true \| false` | `false` | Omit end caps. |
| `thetaStart <radians>` | `0` | Starting angle. |
| `thetaLength <radians>` | `6.283185307179586` | Arc length. |
| `physics true \| false` | `true` | Accepted runtime descriptor field; model import settings own cooked collision. |
| `dynamic true \| false` | `false` | Accepted runtime descriptor field; a cooked geometry is static source data. |
| `uvRepeat <u> <v>` | `1 1` | Multiply source UVs. |

<a id="file-model-part-coneGeometry"></a>
### `file:model.part.coneGeometry`

| Field | Default | Meaning |
|---|---|---|
| `radius <metres >= 0>` | `1` | Base radius. |
| `height <metres >= 0>` | `1` | Y extent. |
| `radialSegments <3..65536>` | `32` | Segments around the axis. |
| `heightSegments <1..65536>` | `1` | Segments along the axis. |
| `openEnded true \| false` | `false` | Omit base cap. |
| `physics true \| false` | `true` | Accepted runtime descriptor field; model import settings own cooked collision. |
| `dynamic true \| false` | `false` | Accepted runtime descriptor field; a cooked geometry is static source data. |
| `uvRepeat <u> <v>` | `1 1` | Multiply source UVs. |

<a id="file-model-part-capsuleGeometry"></a>
### `file:model.part.capsuleGeometry`

| Field | Default | Meaning |
|---|---|---|
| `radius <metres >= 0>` | `1` | Cap radius. |
| `height <metres >= 0>` | `1` | Distance between cap centres. |
| `capSegments <1..65536>` | `4` | Segments of each cap. |
| `radialSegments <3..65536>` | `8` | Segments around the axis. |
| `heightSegments <1..65536>` | `1` | Segments along the cylinder. |
| `physics true \| false` | `true` | Accepted runtime descriptor field; model import settings own cooked collision. |
| `dynamic true \| false` | `false` | Accepted runtime descriptor field; a cooked geometry is static source data. |
| `uvRepeat <u> <v>` | `1 1` | Multiply source UVs. |

<a id="file-model-part-torusGeometry"></a>
### `file:model.part.torusGeometry`

| Field | Default | Meaning |
|---|---|---|
| `radius <metres >= 0>` | `1` | Major radius. |
| `tube <metres >= 0>` | `0.4` | Tube radius. |
| `radialSegments <2..65536>` | `12` | Segments around the tube. |
| `tubularSegments <3..65536>` | `48` | Segments around the ring. |
| `arc <radians>` | `6.283185307179586` | Ring angle. |
| `physics true \| false` | `true` | Accepted runtime descriptor field; model import settings own cooked collision. |
| `dynamic true \| false` | `false` | Accepted runtime descriptor field; a cooked geometry is static source data. |
| `uvRepeat <u> <v>` | `1 1` | Multiply source UVs. |

<a id="file-model-part-ringGeometry"></a>
### `file:model.part.ringGeometry`

| Field | Default | Meaning |
|---|---|---|
| `innerRadius <metres >= 0>` | `0.5` | Inner radius. |
| `outerRadius <metres >= 0>` | `1` | Outer radius. |
| `thetaSegments <3..65536>` | `32` | Angular segments. |
| `phiSegments <1..65536>` | `1` | Radial segments. |
| `thetaStart <radians>` | `0` | Starting angle. |
| `thetaLength <radians>` | `6.283185307179586` | Arc length. |
| `physics true \| false` | `true` | Accepted runtime descriptor field; model import settings own cooked collision. |
| `dynamic true \| false` | `false` | Accepted runtime descriptor field; a cooked geometry is static source data. |
| `uvRepeat <u> <v>` | `1 1` | Multiply source UVs. |

<a id="file-model-part-circleGeometry"></a>
### `file:model.part.circleGeometry`

| Field | Default | Meaning |
|---|---|---|
| `radius <metres >= 0>` | `1` | Disc radius. |
| `segments <3..65536>` | `32` | Angular segments. |
| `thetaStart <radians>` | `0` | Starting angle. |
| `thetaLength <radians>` | `6.283185307179586` | Arc length. |
| `physics true \| false` | `true` | Accepted runtime descriptor field; model import settings own cooked collision. |
| `dynamic true \| false` | `false` | Accepted runtime descriptor field; a cooked geometry is static source data. |
| `uvRepeat <u> <v>` | `1 1` | Multiply source UVs. |

<a id="file-model-part-tubeGeometry"></a>
### `file:model.part.tubeGeometry`

| Field | Default | Meaning |
|---|---|---|
| `points <x y z> ...` | `` | At least two polyline points; exclusive with path. |
| `path { ... }` | `` | 3D path block, exclusive with points and closed. |
| `tubularSegments <1..65536>` | `64` | Longitudinal segments. |
| `radius <metres >= 0>` | `1` | Tube radius. |
| `radialSegments <3..65536>` | `8` | Cross-section segments. |
| `closed true \| false` | `false` | Close the polyline; a path owns its own closed flag. |
| `physics true \| false` | `true` | Accepted runtime descriptor field; model import settings own cooked collision. |
| `dynamic true \| false` | `false` | Accepted runtime descriptor field; a cooked geometry is static source data. |
| `uvRepeat <u> <v>` | `1 1` | Multiply source UVs. |

<a id="file-model-part-shapeGeometry"></a>
### `file:model.part.shapeGeometry`

| Field | Default | Meaning |
|---|---|---|
| `shape { ... }` | `triangle` | Repeat Shape path blocks, each with optional holes. |
| `curveSegments <1..4096>` | `12` | Curve subdivisions. |
| `physics true \| false` | `true` | Accepted runtime descriptor field; model import settings own cooked collision. |
| `dynamic true \| false` | `false` | Accepted runtime descriptor field; a cooked geometry is static source data. |
| `uvRepeat <u> <v>` | `1 1` | Multiply source UVs. |

<a id="file-model-part-extrudeGeometry"></a>
### `file:model.part.extrudeGeometry`

| Field | Default | Meaning |
|---|---|---|
| `shape { ... }` | `unit square` | Repeat Shape path blocks, each with optional holes. |
| `depth <metres >= 0>` | `1` | Straight extrusion depth when extrudePath is absent. |
| `steps <1..65536>` | `1` | Steps along the extrusion. |
| `bevelEnabled true \| false` | `true` | Bevel the contour. |
| `bevelThickness <metres >= 0>` | `0.2` | Bevel depth. |
| `bevelSize <metres>` | `bevelThickness - 0.1` | Bevel contour inset. |
| `bevelOffset <metres>` | `0` | Bevel contour offset. |
| `bevelSegments <1..4096>` | `3` | Bevel subdivisions. |
| `curveSegments <1..4096>` | `12` | Shape curve subdivisions. |
| `extrudePath { ... }` | `` | 3D path block for curved extrusion. |
| `physics true \| false` | `true` | Accepted runtime descriptor field; model import settings own cooked collision. |
| `dynamic true \| false` | `false` | Accepted runtime descriptor field; a cooked geometry is static source data. |
| `uvRepeat <u> <v>` | `1 1` | Multiply source UVs. |

<a id="file-model-part-latheGeometry"></a>
### `file:model.part.latheGeometry`

| Field | Default | Meaning |
|---|---|---|
| `points <x y> ...` | `generator profile` | At least two radial profile points. |
| `segments <1..65536>` | `12` | Angular segments. |
| `phiStart <radians>` | `0` | Starting angle. |
| `phiLength <radians>` | `6.283185307179586` | Arc length. |
| `physics true \| false` | `true` | Accepted runtime descriptor field; model import settings own cooked collision. |
| `dynamic true \| false` | `false` | Accepted runtime descriptor field; a cooked geometry is static source data. |
| `uvRepeat <u> <v>` | `1 1` | Multiply source UVs. |

<a id="file-model-part-polyhedronGeometry"></a>
### `file:model.part.polyhedronGeometry`

| Field | Default | Meaning |
|---|---|---|
| `vertices <x y z> ...` | `required` | Input polyhedron vertices. |
| `indices <u32> ...` | `required` | Triangle indices; at least one face. |
| `radius <metres >= 0>` | `1` | Projected radius. |
| `detail <0..64>` | `0` | Subdivision detail. |
| `physics true \| false` | `true` | Accepted runtime descriptor field; model import settings own cooked collision. |
| `dynamic true \| false` | `false` | Accepted runtime descriptor field; a cooked geometry is static source data. |
| `uvRepeat <u> <v>` | `1 1` | Multiply source UVs. |

<a id="file-model-part-icosahedronGeometry"></a>
### `file:model.part.icosahedronGeometry`

| Field | Default | Meaning |
|---|---|---|
| `radius <metres >= 0>` | `1` | Projected radius. |
| `detail <0..64>` | `0` | Subdivision detail. |
| `physics true \| false` | `true` | Accepted runtime descriptor field; model import settings own cooked collision. |
| `dynamic true \| false` | `false` | Accepted runtime descriptor field; a cooked geometry is static source data. |
| `uvRepeat <u> <v>` | `1 1` | Multiply source UVs. |

<a id="file-model-part-dodecahedronGeometry"></a>
### `file:model.part.dodecahedronGeometry`

| Field | Default | Meaning |
|---|---|---|
| `radius <metres >= 0>` | `1` | Projected radius. |
| `detail <0..64>` | `0` | Subdivision detail. |
| `physics true \| false` | `true` | Accepted runtime descriptor field; model import settings own cooked collision. |
| `dynamic true \| false` | `false` | Accepted runtime descriptor field; a cooked geometry is static source data. |
| `uvRepeat <u> <v>` | `1 1` | Multiply source UVs. |

<a id="file-model-part-octahedronGeometry"></a>
### `file:model.part.octahedronGeometry`

| Field | Default | Meaning |
|---|---|---|
| `radius <metres >= 0>` | `1` | Projected radius. |
| `detail <0..64>` | `0` | Subdivision detail. |
| `physics true \| false` | `true` | Accepted runtime descriptor field; model import settings own cooked collision. |
| `dynamic true \| false` | `false` | Accepted runtime descriptor field; a cooked geometry is static source data. |
| `uvRepeat <u> <v>` | `1 1` | Multiply source UVs. |

<a id="file-model-part-tetrahedronGeometry"></a>
### `file:model.part.tetrahedronGeometry`

| Field | Default | Meaning |
|---|---|---|
| `radius <metres >= 0>` | `1` | Projected radius. |
| `detail <0..64>` | `0` | Subdivision detail. |
| `physics true \| false` | `true` | Accepted runtime descriptor field; model import settings own cooked collision. |
| `dynamic true \| false` | `false` | Accepted runtime descriptor field; a cooked geometry is static source data. |
| `uvRepeat <u> <v>` | `1 1` | Multiply source UVs. |

<a id="file-model-part-sweepGeometry"></a>
### `file:model.part.sweepGeometry`

| Field | Default | Meaning |
|---|---|---|
| `section { ... }` | `required` | Path2 cross-section without holes. |
| `path { ... }` | `required` | 3D path block. |
| `steps <1..65536>` | `64` | Path subdivisions. |
| `scale <t> <value>` | `` | Repeat interpolation keys along the path. |
| `twist <t> <radians>` | `` | Repeat twist keys along the path. |
| `caps true \| false` | `true` | Close the ends. |
| `curveSegments <1..4096>` | `12` | Section curve subdivisions. |
| `physics true \| false` | `true` | Accepted runtime descriptor field; model import settings own cooked collision. |
| `dynamic true \| false` | `false` | Accepted runtime descriptor field; a cooked geometry is static source data. |
| `uvRepeat <u> <v>` | `1 1` | Multiply source UVs. |

<a id="file-model-part-csgGeometry"></a>
### `file:model.part.csgGeometry`

| Field | Default | Meaning |
|---|---|---|
| `op union \| subtract \| intersect` | `union` | Boolean operation. |
| `a { ... }` | `required` | Placed geometry operand. |
| `b { ... }` | `required` | Placed geometry operand. |
| `physics true \| false` | `true` | Accepted runtime descriptor field; model import settings own cooked collision. |
| `dynamic true \| false` | `false` | Accepted runtime descriptor field; a cooked geometry is static source data. |
| `uvRepeat <u> <v>` | `1 1` | Multiply source UVs. |

<a id="file-model-part-mergeGeometries"></a>
### `file:model.part.mergeGeometries`

| Field | Default | Meaning |
|---|---|---|
| `part { ... }` | `required` | Repeat placed geometry blocks; retain attributes and correct reflected winding. |
| `physics true \| false` | `true` | Accepted runtime descriptor field; model import settings own cooked collision. |
| `dynamic true \| false` | `false` | Accepted runtime descriptor field; a cooked geometry is static source data. |
| `uvRepeat <u> <v>` | `1 1` | Multiply source UVs. |

<a id="file-model-part"></a>
### `file:model.part`

| Field | Default | Meaning |
|---|---|---|
| `material "<name>"` | `glTF default` | Previously declared material. |
| `pivot "<name>"` | `model root` | Previously declared parent pivot. |
| `at { ... }` | `` | Repeat placement blocks, exclusive with the part's own placement. |
| `position <x> <y> <z>` | `0 0 0` | Translation in metres. |
| `quaternion <x> <y> <z> <w>` | `0 0 0 1` | Nonzero quaternion, normalized on import. |
| `scale <x> <y> <z>` | `1 1 1` | Nonzero scale; negative components bake reflected triangles. |
| `mirror none \| x \| y \| z` | `none` | Reflect geometry, correcting winding and tangent handedness. |
| `modelGeometry { ... }` | `` | Exactly one geometry block per part. |
| `bufferGeometry { ... }` | `` | Exactly one geometry block per part. |
| `boxGeometry { ... }` | `` | Exactly one geometry block per part. |
| `planeGeometry { ... }` | `` | Exactly one geometry block per part. |
| `sphereGeometry { ... }` | `` | Exactly one geometry block per part. |
| `cylinderGeometry { ... }` | `` | Exactly one geometry block per part. |
| `coneGeometry { ... }` | `` | Exactly one geometry block per part. |
| `capsuleGeometry { ... }` | `` | Exactly one geometry block per part. |
| `torusGeometry { ... }` | `` | Exactly one geometry block per part. |
| `ringGeometry { ... }` | `` | Exactly one geometry block per part. |
| `circleGeometry { ... }` | `` | Exactly one geometry block per part. |
| `tubeGeometry { ... }` | `` | Exactly one geometry block per part. |
| `shapeGeometry { ... }` | `` | Exactly one geometry block per part. |
| `extrudeGeometry { ... }` | `` | Exactly one geometry block per part. |
| `latheGeometry { ... }` | `` | Exactly one geometry block per part. |
| `polyhedronGeometry { ... }` | `` | Exactly one geometry block per part. |
| `icosahedronGeometry { ... }` | `` | Exactly one geometry block per part. |
| `dodecahedronGeometry { ... }` | `` | Exactly one geometry block per part. |
| `octahedronGeometry { ... }` | `` | Exactly one geometry block per part. |
| `tetrahedronGeometry { ... }` | `` | Exactly one geometry block per part. |
| `sweepGeometry { ... }` | `` | Exactly one geometry block per part. |
| `csgGeometry { ... }` | `` | Exactly one geometry block per part. |
| `mergeGeometries { ... }` | `` | Exactly one geometry block per part. |

<a id="file-model-pivot"></a>
### `file:model.pivot`

| Field | Default | Meaning |
|---|---|---|
| `parent "<name>"` | `model root` | Previously declared pivot. Scale must be positive; mirror is not accepted. |
| `position <x> <y> <z>` | `0 0 0` | Translation in metres. |
| `quaternion <x> <y> <z> <w>` | `0 0 0 1` | Nonzero quaternion, normalized on import. |
| `scale <x> <y> <z>` | `1 1 1` | Nonzero scale; negative components bake reflected triangles. |

<a id="file-model-mergeGeometries"></a>
### `file:model.mergeGeometries`

| Field | Default | Meaning |
|---|---|---|
| `part { ... }` | `required` | Repeat placed geometry blocks; retain attributes and correct reflected winding. |
| `physics true \| false` | `true` | Accepted runtime descriptor field; model import settings own cooked collision. |
| `dynamic true \| false` | `false` | Accepted runtime descriptor field; a cooked geometry is static source data. |
| `uvRepeat <u> <v>` | `1 1` | Multiply source UVs. |

<a id="file-model-path"></a>
### `file:model.path`

| Field | Default | Meaning |
|---|---|---|
| `kind catmullRom \| bezier \| linear` | `catmullRom` | 3D curve interpolation. |
| `closed true \| false` | `false` | Close the path. |
| `tension <number>` | `0.5` | Catmull-Rom tension. |
| `point <x> <y> <z>` | `required` | Repeat control points in metres. |

<a id="file-model-shape"></a>
### `file:model.shape`

| Field | Default | Meaning |
|---|---|---|
| `moveTo <x> <y>` | `` | Move the contour cursor. |
| `lineTo <x> <y>` | `` | Append a line. |
| `quadraticCurveTo <cx> <cy> <x> <y>` | `` | Append a quadratic curve. |
| `bezierCurveTo <c1x> <c1y> <c2x> <c2y> <x> <y>` | `` | Append a cubic curve. |
| `splineThru <x y> ...` | `` | Append a spline through point pairs. |
| `points <x y> ...` | `` | Set the contour from point pairs. |
| `arc <x> <y> <radius> <start> <end> [clockwise]` | `` | Relative-centre circular arc; angles in radians, clockwise defaults false. |
| `absarc <x> <y> <radius> <start> <end> [clockwise]` | `` | Absolute-centre circular arc. |
| `ellipse <x> <y> <rx> <ry> <start> <end> [clockwise] [rotation]` | `` | Relative-centre ellipse; radians, false and zero for optional values. |
| `absellipse <x> <y> <rx> <ry> <start> <end> [clockwise] [rotation]` | `` | Absolute-centre ellipse. |
| `closePath ` | `` | Close the contour. |
| `hole { ... }` | `` | Hole contour; only within a Shape, never nested or in a sweep section. |

<a id="file-model-material"></a>
### `file:model.material`

| Field | Default | Meaning |
|---|---|---|
| `color <r> <g> <b>` | `1 1 1` | Linear base colour. |
| `opacity <0..1>` | `1` | Base alpha; below one enables blending unless alphaTest is present. |
| `metalness <0..1>` | `0` | Metallic factor. |
| `roughness <0..1>` | `1` | Roughness factor. |
| `emissive <r> <g> <b>` | `0 0 0` | Linear emitted colour. |
| `emissiveIntensity <number >= 0>` | `1` | Multiply emissive, preserving values above one through the emissive-strength extension. |
| `doubleSided true \| false` | `false` | Draw both faces. |
| `alphaTest <0..1>` | `` | Alpha mask cutoff; takes precedence over opacity blending. |
| `map "<image>"` | `` | Base colour image relative to the model, not an asset handle. |
| `normalMap "<image>"` | `` | Normal image relative to the model. |
| `normalScale <number>` | `1` | Scale normal-map strength. |
| `metalnessMap "<image>"` | `` | Packed roughness (green) and metalness (blue). |
| `aoMap "<image>"` | `` | Occlusion in red. |
| `transmission <0..1>` | `0` | Specular transmission. |
| `ior <1..3>` | `` | Explicit index of refraction. |
| `thickness <metres >= 0>` | `0` | Volume thickness when transmission is nonzero. |
| `clearcoat <0..1>` | `0` | Clearcoat weight. |
| `clearcoatRoughness <0..1>` | `0` | Clearcoat roughness when clearcoat is nonzero. |
| `translucency <0..1>` | `0` | Diffuse transmission; foliage draws both faces. |
| `domeNormals <0..1>` | `0` | Foliage normals' canopy-dome blend. |
| `season <0..1>` | `0` | Evergreen at zero, deciduous at one. |
| `autumnColor <r> <g> <b>` | `0.42 0.12 0.015` | Nonnegative linear seasonal colour. |
| `autumnColor2 <r> <g> <b>` | `0.25 0.025 0.015` | Second seasonal colour. |
| `seasonShift <days>` | `0` | Shift the species' seasonal cycle within LeafSeason.MAX_SHIFT. |
| `grime <0..1>` | `0` | Ground grime weight. |
| `grimeColor <r> <g> <b>` | `0 0 0` | Nonnegative linear grime colour. |
| `grimeHeight <metres >= 0>` | `0` | Ground grime extent. |
| `stains <0..1>` | `0` | Stain variation weight. |
| `stochastic <0..1>` | `0` | Stochastic texture sampling weight. |

<a id="file-model-placement"></a>
### `file:model.placement`

| Field | Default | Meaning |
|---|---|---|
| `position <x> <y> <z>` | `0 0 0` | Translation in metres. |
| `quaternion <x> <y> <z> <w>` | `0 0 0 1` | Nonzero quaternion, normalized on import. |
| `scale <x> <y> <z>` | `1 1 1` | Nonzero scale; negative components bake reflected triangles. |
| `mirror none \| x \| y \| z` | `none` | Reflect geometry, correcting winding and tangent handedness. |

<a id="file-model-plant"></a>
### `file:model.plant`

| Field | Default | Meaning |
|---|---|---|
| `kind <kind>` | `` | Optional existing plant defaults; explicit numbers override them. No implicit species selection. |
| `seed <u32>` | `Plant default or kind` | Deterministic generator seed. |
| `growth deciduous \| evergreen` | `Plant default or kind` | Branch growth mechanism. |
| `levels <0..3>` | `Plant default or kind` | Branch levels after the trunk. |
| `path <x y z> ...` | `` | 2..1025 authored trunk ring centres in metres; replaces trunk wandering and sections. Children grow from this curve. |
| `branch { ... }` | `` | Repeat parent-first complete skeleton limbs, exclusive with path. No procedural branches are added; terminal limbs receive foliage. |
| `bounds <min x y z> <max x y z>` | `` | Centreline box in metres, min < max on every axis. Procedural limbs stop before their first outside ring; explicit skeletons must fit. Does not clip wood radii or foliage. |
| `leafGeometry { <geometry> { ... } }` | `` | Unit custom leaf, x across, y along, z facing; transformed by leaf size and width with generated wind. Uses the normal geometry and model operand parser. |
| `flowerGeometry { <geometry> { ... } }` | `` | Unit custom whole flower replacing procedural petals and centre; transformed by flower size and width. |
| `force <x> <y> <z> <strength>` | `Plant default or kind` | Tropism direction and strength. |
| `trunk { ... }` | `Plant default or kind` | Trunk level parameters. |
| `branch1 { ... }` | `Plant default or kind` | First branch level. |
| `branch2 { ... }` | `Plant default or kind` | Second branch level. |
| `branch3 { ... }` | `Plant default or kind` | Third branch level. |
| `leaves { ... }` | `none without kind` | Enable leaves or override kind leaves. |
| `flowers { ... }` | `none without kind` | Enable flowers or override kind flowers. |
| `material "<name>"` | `generated bark material` | Previously declared wood material. |
| `barkColor <r> <g> <b>` | `0.13 0.085 0.05 without kind` | Nonnegative linear bark colour. |

<a id="file-model-plant-branch"></a>
### `file:model.plant.branch`

| Field | Default | Meaning |
|---|---|---|
| `parent <-1 or earlier index>` | `-1` | First limb is sole root (-1); every other limb names an earlier limb. |
| `path <x y z> ...` | `required` | 2..1025 finite ring centres; child starts on its parent's centreline within 0.000001 m. |
| `radii <metres> ...` | `required` | One radius >= 0.0001 per ring; replaces taper. |
| `segments <3..32>` | `6` | Ring sides. |

<a id="file-model-plant-leaves"></a>
### `file:model.plant.leaves`

| Field | Default | Meaning |
|---|---|---|
| `shape blade \| card \| cross` | `blade` | Leaf mesh construction. |
| `count <0..2000>` | `inherited` | Leaves per last-level branch. |
| `start <0..1>` | `inherited` | First leaf position. |
| `end <0..1>` | `inherited` | Last leaf position, at least start. |
| `angle <degrees>` | `inherited` | Leaf angle. |
| `facing <0..1>` | `inherited` | Sky-facing blend. |
| `rows <0..8>` | `inherited` | Leaf rows. |
| `size <metres >= 0>` | `inherited` | Leaf length. |
| `width <number >= 0>` | `inherited` | Width relative to length. |
| `sizeVariance <0..1>` | `inherited` | Length variation. |
| `color <r> <g> <b>` | `inherited` | Nonnegative linear colour. |
| `color2 <r> <g> <b>` | `color when specified, else inherited` | Second linear colour. |
| `colorVariation <0..1>` | `inherited` | Per-leaf variation. |
| `material "<name>"` | `generated leaf material` | Previously declared material; owns translucency, domeNormals and season fields when named. |
| `translucency <0..1>` | `0.5 without kind` | Diffuse transmission, exclusive with a named material. |
| `domeNormals <0..1>` | `0.6 without kind` | Canopy normal blend, exclusive with a named material. |
| `season <0..1>` | `0 without kind` | Seasonal blend, exclusive with a named material. |
| `autumnColor <r> <g> <b>` | `inherited` | Seasonal colour, exclusive with a named material. |
| `autumnColor2 <r> <g> <b>` | `inherited` | Second seasonal colour, exclusive with a named material. |
| `seasonShift <days>` | `0 without kind` | Season shift, exclusive with a named material. |

<a id="file-model-plant-flowers"></a>
### `file:model.plant.flowers`

| Field | Default | Meaning |
|---|---|---|
| `share <0..1>` | `inherited` | Share of last-level ends with flowers. |
| `count <0..200>` | `inherited` | Flowers per branch. |
| `petals <1..32>` | `inherited` | Petal count. |
| `size <metres >= 0>` | `inherited` | Petal length. |
| `width <number >= 0>` | `inherited` | Relative petal width. |
| `cup <degrees>` | `inherited` | Petal cup angle. |
| `color <r> <g> <b>` | `inherited` | Nonnegative linear petal colour. |
| `color2 <r> <g> <b>` | `color when specified, else inherited` | Second petal colour. |
| `centerColor <r> <g> <b>` | `inherited` | Nonnegative linear flower centre colour. |

<a id="file-model-plant-trunk"></a>
### `file:model.plant.trunk`

| Field | Default | Meaning |
|---|---|---|
| `length <metres >= 0>` | `inherited` | Level length. |
| `radius <number >= 0>` | `inherited` | Trunk metres, branch share of parent radius. |
| `sections <1..64>` | `inherited` | Longitudinal rings. |
| `segments <3..32>` | `inherited` | Ring sides. |
| `taper <0..1>` | `inherited` | Share of radius lost by the end. |
| `gnarliness <radians>` | `inherited` | Section wander. |
| `twist <radians>` | `inherited` | Turn around the axis per section. |
| `children <0..400>` | `inherited` | Child branches. |

<a id="file-model-plant-branch1"></a>
### `file:model.plant.branch1`

| Field | Default | Meaning |
|---|---|---|
| `length <metres >= 0>` | `inherited` | Level length. |
| `radius <number >= 0>` | `inherited` | Trunk metres, branch share of parent radius. |
| `sections <1..64>` | `inherited` | Longitudinal rings. |
| `segments <3..32>` | `inherited` | Ring sides. |
| `taper <0..1>` | `inherited` | Share of radius lost by the end. |
| `gnarliness <radians>` | `inherited` | Section wander. |
| `twist <radians>` | `inherited` | Turn around the axis per section. |
| `children <0..400>` | `inherited` | Child branches. |
| `angle <degrees>` | `inherited` | Branch angle from parent; not a trunk field. |
| `start <0..1>` | `inherited` | First child position; not a trunk field. |

<a id="file-model-plant-branch2"></a>
### `file:model.plant.branch2`

| Field | Default | Meaning |
|---|---|---|
| `length <metres >= 0>` | `inherited` | Level length. |
| `radius <number >= 0>` | `inherited` | Trunk metres, branch share of parent radius. |
| `sections <1..64>` | `inherited` | Longitudinal rings. |
| `segments <3..32>` | `inherited` | Ring sides. |
| `taper <0..1>` | `inherited` | Share of radius lost by the end. |
| `gnarliness <radians>` | `inherited` | Section wander. |
| `twist <radians>` | `inherited` | Turn around the axis per section. |
| `children <0..400>` | `inherited` | Child branches. |
| `angle <degrees>` | `inherited` | Branch angle from parent; not a trunk field. |
| `start <0..1>` | `inherited` | First child position; not a trunk field. |

<a id="file-model-plant-branch3"></a>
### `file:model.plant.branch3`

| Field | Default | Meaning |
|---|---|---|
| `length <metres >= 0>` | `inherited` | Level length. |
| `radius <number >= 0>` | `inherited` | Trunk metres, branch share of parent radius. |
| `sections <1..64>` | `inherited` | Longitudinal rings. |
| `segments <3..32>` | `inherited` | Ring sides. |
| `taper <0..1>` | `inherited` | Share of radius lost by the end. |
| `gnarliness <radians>` | `inherited` | Section wander. |
| `twist <radians>` | `inherited` | Turn around the axis per section. |
| `children <0..400>` | `inherited` | Child branches. |
| `angle <degrees>` | `inherited` | Branch angle from parent; not a trunk field. |
| `start <0..1>` | `inherited` | First child position; not a trunk field. |

<a id="file-model-fracture"></a>
### `file:model.fracture`

| Field | Default | Meaning |
|---|---|---|
| `style voronoi \| concrete \| brick \| wood \| glass` | `voronoi` | Fracture mechanism, not a gameplay rule. |
| `chunks <2..256>` | `16` | Support chunk count. |
| `seed <u32>` | `1` | Deterministic cut pattern. |
| `site <x> <y> <z>` | `` | Repeat 2..128 explicit Voronoi centres in model metres; requires style voronoi and pattern volume, replaces automatic support-cell sampling. Site order fixes cell ids. Children retain their declared generator. |
| `children <0..32>` | `0` | Child pieces per support chunk. |
| `pattern auto \| volume \| slab` | `auto` | 3D cells or shards through the thinnest axis; auto detects thin input. |
| `brick <length> <height>` | `0.25 0.1` | Brick dimensions, each at least 0.01 metres. |
| `grain x \| y \| z` | `longest axis` | Wood grain axis. |
| `rings <1..8>` | `2` | Glass concentric cracks. |
| `impact <x> <y> <z> <radius>` | `` | Gather the pattern around an authored model-space impact; radius positive. |
| `anchor bottom [ends] \| ends \| none \| plane <nx> <ny> <nz> <distance>` | `bottom` | Support surfaces; plane normal must be nonzero. |
| `interiorScale <number > 0>` | `1` | Cut-face UV repeats per metre. |
| `interior { ... }` | `darkened main material` | Cut-face material using model.material fields. |
| `mortar { ... }` | `` | Debris material using model.material fields. |

<a id="file-model-process"></a>
### `file:model.process`

| Field | Default | Meaning |
|---|---|---|
| `repair { ... }` | `off` | Run the existing mesh repair kernel before simplification. |
| `decimate { ... }` | `off` | Explicit triangle/error budget, independently per material primitive. |
| `unwrap { ... }` | `off` | Replace UVs. Existing maps require bake so their appearance is not silently lost. |
| `bake { ... }` | `off` | Transfer the source material and normals to a fresh low-mesh atlas. No image generation. |
| `project { ... }` | `off` | Project registered authored views into prepared UVs through og-bake, preserving other material channels. See model.process.project. |
| `transfer { ... }` | `off` | Carry an imported base model's painted maps to named parts in their own unscaled metric frames, through og-bake. Runs after project. |
| `rig { ... }` | `off` | Bind explicit joints, or explicitly request the existing humanoid finder, after static processing. |

<a id="file-model-process-repair"></a>
### `file:model.process.repair`

| Field | Default | Meaning |
|---|---|---|
| `weld <metres >= 0>` | `0` | Weld distance; zero welds exact duplicates. |
| `weldAll true \| false` | `false` | Also weld interior vertices. |
| `floaters <metres >= 0>` | `0` | Drop disconnected pieces below this diagonal. |
| `floaterFraction <0..1>` | `0` | Drop pieces below this share of the largest diagonal. |
| `nonManifold true \| false` | `false` | Split non-manifold sheets. |
| `holeEdges <integer >= 0>` | `0` | Largest hole rim to close. |
| `holesThrough true \| false` | `false` | Permit a crossing fill of larger holes. |
| `thicken <metres >= 0>` | `0` | Thickness given to open sheets. |
| `fuse true \| false` | `false` | Self-union closed interpenetrating shells. |
| `minEdge <metres >= 0>` | `0` | Collapse edges below this length where manifoldness survives. |

<a id="file-model-process-decimate"></a>
### `file:model.process.decimate`

| Field | Default | Meaning |
|---|---|---|
| `target <integer >= 0>` | `0` | Maximum triangles per material primitive; zero is unset. |
| `ratio <0..1>` | `0` | Triangle share; zero is unset. With target the smaller count wins. |
| `maxError <metres >= 0>` | `0` | Quadric RMS displacement limit; zero is unlimited. |
| `lockBorder true \| false` | `false` | Keep open-border vertices fixed. |
| `mergeCharts <UV distance >= 0>` | `0` | Join UV charts agreeing up to a rigid move. |
| `crossColorBorders true \| false` | `false` | Permit collapse across hard vertex-colour borders. |

<a id="file-model-process-unwrap"></a>
### `file:model.process.unwrap`

| Field | Default | Meaning |
|---|---|---|
| `angle <degrees, 0..90)` | `0 (kernel 66)` | Chart projection angle. |
| `resolution <integer, 0..8192>` | `0 (kernel 1024)` | Grid used to measure padding. |
| `padding <texels >= 0>` | `0 (kernel 2)` | Chart separation. |
| `shared true \| false` | `false` | Share chart projection directions. |
| `conformal true \| false` | `false` | Use angle-preserving flattening where valid. |
| `seams true \| false` | `false` | Try seam-cut charts before flattening. |

<a id="file-model-process-bake"></a>
### `file:model.process.bake`

| Field | Default | Meaning |
|---|---|---|
| `size <integer, 1..8192>` | `1024` | Atlas and material-map side in texels. |
| `distance <metres >= 0>` | `0` | Source search distance; zero uses the kernel's 2% of diagonal. |
| `padding <integer, 0..8192>` | `4` | Atlas gutters. |
| `dilate <integer >= 0>` | `0` | Dilation layers; zero fills all uncovered texels. |
| `normals true \| false` | `true` | Transfer source shading normals as a tangent-space normal map. |

<a id="file-model-process-project"></a>
### `file:model.process.project`

| Field | Default | Meaning |
|---|---|---|
| `size <1..8192>` | `1024` | Output map side in texels per primitive. |
| `padding <0..8192>` | `4` | Grow UV chart gutters after filling unseen texels. |
| `view "<image>" { ... }` | `required` | Repeat square RGBA images registered to explicit cameras. Paths are model-relative; alpha zero leaves a pixel unpainted. No implicit silhouette fitting. |

<a id="file-model-process-project-view"></a>
### `file:model.process.project.view`

| Field | Default | Meaning |
|---|---|---|
| `position <x> <y> <z>` | `required` | Camera position in model-local metres, Y up. |
| `target <x> <y> <z>` | `required` | Camera target, distinct from position. Vertical views use the kernel's fixed X right axis. |
| `fov <degrees, 0..179>` | `0` | Positive perspective vertical field of view; exclusive with ortho. |
| `ortho <metres > 0>` | `0` | Orthographic half-height and half-width; exclusive with positive fov. One projection size is required. |

<a id="file-model-process-transfer"></a>
### `file:model.process.transfer`

| Field | Default | Meaning |
|---|---|---|
| `source "<model asset>"` | `required` | Asset-root-relative base model, normally imported and processed before this model. Source contents, external files and parameters enter the cache key; cycles fail. |
| `size <integer, 1..8192>` | `1024` | Destination map side. Source painted maps must be square and share one size; they are not resampled. |
| `padding <integer, 0..8192>` | `4` | Destination chart gutters after nearest-painted-surface hole filling. |

<a id="file-model-process-rig"></a>
### `file:model.process.rig`

| Field | Default | Meaning |
|---|---|---|
| `auto true \| false` | `false` | Explicitly run the existing humanoid joint finder; exclusive with joint lines. |
| `joint "<name>" "<parent or empty>" <x> <y> <z>` | `` | Repeat parent-first joints in model-local metres, Y up. Each processed primitive is bound to the same skeleton. |

Attribute arrays are validated by the same bufferGeometry implementation as runtime. May also be nested in mergeGeometries or CSG operands; topology-changing CSG generates new tangent frames.

<a id="file-model-part-bufferGeometry"></a>
### `file:model.part.bufferGeometry`

| Field | Default | Meaning |
|---|---|---|
| `positions <x y z> ...` | `required` | Finite vertex positions in metres; at least one triangle. |
| `normals <x y z> ...` | `computed` | One normal per vertex, validated and normalised by bufferGeometry. |
| `uvs <u v> ...` | `0 0` | One UV per vertex, v down the image. |
| `colors <r g b> ... \| <r g b a> ...` | `1 1 1 1` | Linear vertex colours, one RGB or RGBA value per vertex. |
| `tangents <x y z w> ...` | `generated when needed` | One glTF tangent frame per vertex; w is its handedness. |
| `indices <u32> ...` | `consecutive vertices` | Counter-clockwise triangle index triples within the vertex count. |
| `generate { ... }` | `` | Deterministic script returns MeshSource attributes, exclusive with authored attribute arrays. |
| `uvRepeat <u> <v>` | `1 1` | Scale UVs after reading the source. |
| `physics true \| false` | `true` | Runtime descriptor field; model collision is owned by model import settings. |
| `dynamic true \| false` | `false` | Runtime descriptor field; a cooked model is static source data. |

<a id="file-model-part-modelGeometry"></a>
### `file:model.part.modelGeometry`

An imported model's rest geometry as an operand of mergeGeometries or CSG, or as a part by itself. The source uses its ordinary importer and generation path before this model.

| Field | Default | Meaning |
|---|---|---|
| `source "<model asset>"` | `required` | Asset-root relative model, imported before this model. Cycles are refused. |
| `node "<node name>"` | `all visible` | Select a mesh node by name, otherwise merge the visible default scene at rest. Node placements, UVs, colours and tangent frames are retained. The surrounding part owns the material; skins and animation are not copied into a static geometry operand. |

<a id="file-model-part-bufferGeometry-generate"></a>
### `file:model.part.bufferGeometry.generate`

Returns MeshSource: positions, and optional normals/uvs/colors/tangents/indices. Materials, pivots and placements remain authored in the model.

| Field | Default | Meaning |
|---|---|---|
| `script "<file.ts>"` | `required` | Synchronous default export; static relative imports and explicit inputs are tracked. |
| `seed <u32>` | `0` | Explicit seed passed to the generator. |
| `params { ... }` | `{}` | Named scalar, array or nested object parameters. |
| `input "<name>" "<path>"` | `` | Repeated input bytes, relative to the authored asset, available as ctx.inputs[name]. |

<a id="file-motion-action"></a>
### `file:motion-action`

An authored `motion-action` asset, cooked by `motion-action`. Input: `.ogt` under assets/.

Field coverage: complete shared action parser line table. Defines an existing authored clip's hold interval/contact, not offline neural motion inference.

| Field | Default | Meaning |
|---|---|---|
| `source "<model asset>"` | `required` | Source animation model, checked by cook. |
| `clip "<clip name>"` | `required` | Authored clip name, not a runtime handle. |
| `rig "<rig asset>"` | `detected` | Optional humanoid map, checked by cook. |
| `holdStart <seconds >= 0>` | `0` | Beginning of the authored hold interval. |
| `holdEnd <seconds > holdStart>` | `required` | End of the authored hold interval. |
| `hand left \| right` | `` | Optional hand contact. |
| `handTarget <x> <y> <z>` | `0 0 0` | Finite target in the head's local frame. |

<a id="file-navmesh"></a>
### `file:navmesh`

An authored `navmesh` asset, cooked by `navmesh`. Input: `.ogt` under assets/.

Field coverage: parser-owned root and nested field tables. Bake validates the voxel/agent parameters. Paths are asset-root relative. No agent traversal rule is inferred.

| Field | Default | Meaning |
|---|---|---|
| `scene "<scene asset>"` | `` | Static scene geometry; asset-root relative path. |
| `agent [<name>] { ... }` | `unnamed default profile` | Repeat agent profiles; names contain letters, digits, underscore or hyphen. The unnamed profile, otherwise first, is the default. |
| `cellSize <metres>` | `0.2` | Horizontal voxel size. |
| `cellHeight <metres>` | `0.1` | Vertical voxel size. |
| `tileSize <number>` | `32` | Tile side in cells; clamped to 8..256 and truncated to an integer. |
| `maxError <number>` | `1.3` | Contour simplification error. |
| `box { ... }` | `` | Repeat additional static boxes. |
| `model "<model asset>" { ... }` | `` | Repeat additional placed static models; paths are asset-root relative. |
| `terrain "<terrain asset>" { ... }` | `` | Repeat terrain surfaces; their source and height data are cook dependencies. |
| `bounds { ... }` | `unbounded` | Optional bake region. |
| `area { ... }` | `` | Repeat area shapes; later shapes override earlier shapes in overlaps. Radius selects cylinder, otherwise points select prism, otherwise box. |
| `link { ... }` | `` | Repeat explicitly authored off-mesh links. |

<a id="file-navmesh-agent"></a>
### `file:navmesh.agent`

| Field | Default | Meaning |
|---|---|---|
| `radius <metres>` | `0.4` | Agent radius. |
| `height <metres>` | `1.8` | Agent height. |
| `climb <metres>` | `0.4` | Maximum step height. |
| `slope <degrees>` | `45` | Maximum walkable slope. |

<a id="file-navmesh-box"></a>
### `file:navmesh.box`

| Field | Default | Meaning |
|---|---|---|
| `center <x> <y> <z>` | `0 0 0` | Box centre in metres. |
| `size <x> <y> <z>` | `1 1 1` | Full box dimensions in metres. |
| `quaternion <x> <y> <z> <w>` | `0 0 0 1` | Box rotation; normalized on read. |

<a id="file-navmesh-model"></a>
### `file:navmesh.model`

| Field | Default | Meaning |
|---|---|---|
| `position <x> <y> <z>` | `0 0 0` | Model translation in metres. |
| `scale <x> <y> <z>` | `1 1 1` | Model scale. |
| `quaternion <x> <y> <z> <w>` | `0 0 0 1` | Model rotation; normalized on read. |

<a id="file-navmesh-terrain"></a>
### `file:navmesh.terrain`

| Field | Default | Meaning |
|---|---|---|
| `position <x> <y> <z>` | `0 0 0` | Terrain translation in metres. |
| `step <number>` | `1` | Sampling stride; clamped to at least one and truncated to an integer. |

<a id="file-navmesh-bounds"></a>
### `file:navmesh.bounds`

| Field | Default | Meaning |
|---|---|---|
| `min <x> <y> <z>` | `-10 -10 -10` | Bake region minimum in metres. |
| `max <x> <y> <z>` | `10 10 10` | Bake region maximum in metres. |

<a id="file-navmesh-area"></a>
### `file:navmesh.area`

| Field | Default | Meaning |
|---|---|---|
| `id <integer, 1..63>` | `required` | Area id; zero is the ground. |
| `center <x> <y> <z>` | `0 0 0` | Box centre or cylinder base in metres. |
| `size <x> <y> <z>` | `1 1 1` | Full box dimensions. |
| `yaw <radians>` | `0` | Box rotation about +Y. |
| `radius <metres>` | `` | Presence selects a cylinder; negative values clamp to zero. |
| `height <metres>` | `2` | Cylinder height; negative values clamp to zero. |
| `point <x> <y> <z>` | `` | Repeat at least three points for a convex prism; only x and z are used. |
| `bottom <metres>` | `-1` | Prism lower Y. |
| `top <metres>` | `2` | Prism upper Y. |

<a id="file-navmesh-link"></a>
### `file:navmesh.link`

| Field | Default | Meaning |
|---|---|---|
| `id <integer >= 1>` | `required` | Unique link id. |
| `start <x> <y> <z>` | `required` | Start in metres. |
| `end <x> <y> <z>` | `required` | End in metres. |
| `kind jump \| ladder \| door \| teleport` | `jump` | Traversal kind chosen by the author. |
| `bidirectional true \| false` | `true` | Allow reverse traversal. |
| `cost <number>` | `0` | Traversal cost; negative values clamp to zero. |
| `duration <seconds>` | `0` | Traversal duration; negative values clamp to zero. |
| `open true \| false` | `true` | Initial availability. |
| `traverse engine \| game` | `engine` | Who moves the agent while traversing. |

<a id="file-particles"></a>
### `file:particles`

An authored `particles` asset, cooked by `emitter`. Input: `.ogt` under assets/.

Field coverage: parser line tables; see `effect-files` for nested blocks and constraints.

Alternatively, one exclusive generate block returns EffectSource {text}: the body declarations without type/version/header. The ordinary parser and asset links validate the result.


<a id="file-particles-generate"></a>
### `file:particles.generate`

| Field | Default | Meaning |
|---|---|---|
| `script "<file.ts>"` | `required` | Synchronous default export; static relative imports and explicit inputs are tracked. |
| `seed <u32>` | `0` | Explicit seed passed to the generator. |
| `params { ... }` | `{}` | Named scalar, array or nested object parameters. |
| `input "<name>" "<path>"` | `` | Repeated input bytes, relative to the authored asset, available as ctx.inputs[name]. |
| Field | Default | Meaning |
|---|---|---|
| `maxParticles <n>` | `256` | The ring of particles, 1 to 1048576: the newest overwrite the oldest. |
| `shape point \| sphere <radius> [surface] \| cone <angle°> <radius> \| box <x> <y> <z> \| spline <x> <y> <z> ... [closed] \| mesh box \| sphere \| tetrahedron \| "<model id>"` | `cone 15 0.05` | Where particles are born and which way they start: a box by its half extents, a spline through 2 to 8 Catmull-Rom points, emitted evenly along it and along its tangent, a mesh's surface (a model's first mesh, in the emitter's space), area weighted and along its normals (a statue crumbling to embers, a sword's glow). |
| `spread <degrees>` | `0` | Random deviation from the shape's direction; 180 is every direction. |
| `rate <per second>` | `40` | Particles born per second. |
| `burst <time> <count> [<cycles> <interval>]` | `` | Particles born at once at a time of each cycle, repeated `cycles` times `interval` apart; any number of lines. |
| `duration <seconds>` | `1` | Seconds per emission cycle. |
| `looping true \| false` | `true` | Start a new cycle at the end of each. |
| `lifetime <min> [<max>]` | `1 1.5` | Seconds a particle lives. |
| `speed <min> [<max>]` | `1.5 2.5` | Metres per second at birth. |
| `size <min> [<max>]` | `0.08 0.12` | Metres across at birth. |
| `sizeOverLife <t> <value> ...` | `` | A curve that scales the size over the life (1 when left out): the width and, without `heightOverLife`, the height. |
| `heightOverLife <t> <value> ...` | `` | A curve that scales the height alone over the life (along the sprite's up, or the velocity when stretched; a mesh's y): a tall flame, a flat shockwave. Left out, the height is the width. |
| `colorOverLife { key <t> <r> <g> <b> <a> ... }` | `` | Linear colour keys over the life, at most 8; left out, white fading over the last quarter. |
| `randomColorOverLife { key <t> <r> <g> <b> <a> ... }` | `` | A second colour over the life: each particle's colour lies between colorOverLife and this at a share of its own (embers from orange to deep red). Left out, every particle takes colorOverLife. |
| `color <r> <g> <b> [<a>]` | `1 1 1 1` | A linear tint over the colour keys; above 1 blooms. |
| `rotation <min> [<max>]` | `0` | The start rotation in degrees. |
| `angularVelocity <min> [<max>]` | `0` | Degrees per second. |
| `gravity <x> <y> <z>` | `0 -2 0` | Metres per second squared. |
| `drag <per second>` | `0` | How fast motion relative to the air dies. |
| `wind <0..1>` | `0` | The share of the air's motion (og.Wind and every og.WindMotor where the system is) that `drag` closes on: smoke and dust drift downwind; 0 slows to rest. |
| `forceFields <0..1>` | `0` | The share of the og.ForceField acceleration where each particle is that it takes (1: as a body does): an updraft lifting smoke, a vortex swirling sparks. World-space particles only. |
| `noise <strength> [<frequency> <scroll>]` | `0 0.5 0.3` | Turbulence: value noise pushing the particles. |
| `texture "<id>"` | `` | A texture asset; left out, a soft dot. |
| `sheet <columns> <rows> [<frames> [off \| life <cycles> \| fps <n> \| random] [blend]]` | `1 1` | The texture as a sprite sheet: a frame by age, at a rate, or one at random. `blend` cross-fades each frame into the next as it plays (life and fps, billboards), so 16 frames of smoke roll smoothly instead of stepping; a sheet played once holds its last frame. |
| `blend additive \| alpha \| subtractive \| multiply` | `` | Left out: alpha for a lit mesh (a solid chip; added it would glow white), additive for everything else. Subtractive takes the colour away from what is behind (dark smoke), multiply tints what is behind by it (scorch); neither in sprite mode. |
| `space world \| local` | `world` | Particles left behind in the world, or carried with the emitter. |
| `align camera \| velocity [<stretch>] \| upright [<x> <y> <z>] \| facing <x> <y> <z>` | `camera` | Billboards face the camera, stretch along their velocity (stretch 0.05 when left out), stand on an axis of the emitter turning to face the eye (0 1 0 when left out: flames), or lie across one whatever the camera (facing 0 1 0: flat on the ground). |
| `softDistance <metres>` | `0` | Fade where a billboard meets the scene behind it. |
| `collision [<bounce> [<friction>]] [depth \| colliders] [die]` | `` | Bounce (0.4) with friction (0.2) off the drawn depth (what the camera sees), or off the scene's og.Collider shapes on or off screen; sprite mode always meets the colliders. With `die` a particle dies where it first hits (rain, sparks going out), a death event for a sub-emitter's splash. |
| `render billboard \| sprite [<layer>] \| mesh box \| sphere \| tetrahedron \| "<model id>"` | `billboard` | 3D billboards, 2D sprites in a sprite layer, or a small mesh per particle that tumbles with its rotation (a model's first mesh). |
| `lit true \| false` | `false` | Billboards and meshes shaded by the sun, the sky and local lights. |
| `roughness <0..1>` | `1` | A lit mesh's surface: 1 matte; smoother reflects over an alpha body (glass shards). |
| `distortion <0..1>` | `0` | Billboards bend the scene behind them by this fraction of the screen at most: heat haze, shock rings. |
| `events birth \| death \| collision ...` | `` | Send og.ParticleEvent for these; the emitter's systems then simulate in the tick. |
| `subEmitter birth \| death \| collision "<id>" <count> [<inherit>] \| life "<id>" [<inherit>]` | `` | Another emitter's particles born where this one's are born, die or bounce (count 1 to 4096, moving at their start velocity plus `inherit` times the parent's), or each particle running the other's own emission while it lives; up to 4, one level deep. |
| `glow <strength>` | `0` | Sprite mode: strength in the 2D glow buffer (PostEffects.glow). |
| `array "<id>" [<layers>] [off \| life <cycles> \| fps <n> \| random]` | `` | Sprite mode: a layer of a texture array (`type texture-array`) per particle instead of `texture`, picked as `sheet` picks a frame from the first `layers`; one draw per emitter. |
| `round true \| false` | `false` | Fade each particle to the soft dot's round edge over its texture or layer. |
| `effect "<id>"` | `` | Draw billboards or mesh particles with an effect material (`type effect`, §2). |
| `speedOverLife <t> <value> ...` | `` | A curve that scales the velocity over the life (a burst that slows). |
| `spinOverLife <t> <value> ...` | `` | A curve that scales the angular velocity over the life. |
| `rateOverTime <t> <value> ...` | `` | A curve that scales `rate` over each cycle (t a fraction of `duration`). |
| `trail <seconds> [<width>] [only]` | `` | A trail behind each particle: the path it took over the last <seconds>, a strip in its colour narrowing and fading to its tail, <width> a share of the particle's width (1 when left out); `only` draws the trails without the particles (a spark's streak). Billboards and mesh particles, not sprite mode. |
| `inheritVelocity <fraction>` | `0` | The share of the emitter's own velocity, in world space, particles start with. |
| `attractor <x> <y> <z> <strength> [<radius>]` | `` | A point in the emitter's space that pulls, or pushes when negative; up to 4. |
| `field curl <strength> [<frequency> <scroll>] \| "<texture-or-field.ogt>" <strength> <size>` | `` | Divergence-free curl noise (frequency 0.5, scroll 0.2), or a vector grid baked from an image N² wide and N high (rgb 0..1 as -1..1, y from bottom), or a lossless cubic 3-channel field asset (N 1..16). Explicit size maps one period of the source lattice to emitter-local units; field origin/spacing describe the source sampling domain, not particle placement. Vectors are not normalized. |
| `fieldGrid <cells> <size> <strength> <vectors>` | `` | What `og cook` writes in place of a texture field: the baked grid, x fastest. Not written by hand. |

<a id="file-post"></a>
### `file:post`

An authored `post` asset, cooked by `post`. Input: `.ogt` under assets/.

Field coverage: complete shared shader-family parser table, filtered by document type. Cook validates the same WGSL contract on every supported backend. See render or ui-materials for WGSL entry structures and callable functions.

| Field | Default | Meaning |
|---|---|---|
| `source "<WGSL file>"` | `` | One source file; exclusive with code. Cook tracks its bytes and inlines it. |
| `code "<WGSL text>"` | `` | Inline WGSL; exclusive with source. One input is required. |
| `param <name> <float \| vec2 \| vec3 \| vec4 \| color> [<defaults> ...]` | `zero components` | Repeat shader parameters; defaults are absent or exactly the type's finite component count. Color is linear RGB. Backend checks validate names, count and WGSL contract. |

<a id="file-prefab"></a>
### `file:prefab`

An authored `prefab` asset, cooked by `prefab`. Input: `.ogt` under assets/.

| Field | Default | Meaning |
|---|---|---|
| `base "<prefab asset>"` | `` | One base before the root entity; remaining fields override it. |
| `entity "<name>" { ... }` | `required` | Exactly one root; nested entity grammar is shared with scenes. |

<a id="file-prefab-entity"></a>
### `file:prefab.entity`

| Field | Default | Meaning |
|---|---|---|
| `entity "<name>" { ... }` | `` | Repeated children in this entity's space. |
| `model "<asset>"` | `` | Instantiate a model; its geometry/material handles are resolved at load. |
| `scene "<asset>"` | `` | Place one sub-scene; dependency cycles fail. |
| `static true \| false` | `false` | Mark this entity's model for baked lighting and HLOD. |
| `prefab "<asset>"` | `` | Expand a prefab; remaining entries are explicit overrides. |
| `override "<node path>" { ... }` | `` | Instance-only component overrides on a descendant. |
| `remove "<node path>" [<component>]` | `` | Instance-only removal of a subtree or component. |

Component blocks and their fields come from reflection, as for scenes. Query the record by name with og api show.

<a id="file-quest"></a>
### `file:quest`

An authored `quest` asset, cooked by `quest`. Input: `.ogt` under assets/.

| Field | Default | Meaning |
|---|---|---|
| `id <id>` | `required` | Unique quest id. |
| `title "<i18n key>"` | `required` | Quest title. |
| `summary "<i18n key>"` | `` | Summary key. |
| `category <word>` | `main` | Game-owned category, no engine semantics. |
| `giver "<i18n key>"` | `` | Quest giver key. |
| `region "<i18n key>"` | `` | Quest region key. |
| `global <name> <value>` | `` | Repeated story variables, checked by the shared expression parser. |
| `reward "<i18n key>" [<amount>] { ... }` | `` | Declared reward metadata; amount defaults to one. Game applies rewards. |
| `stage <id> { ... }` | `required` | Repeated stages in authored order. |

<a id="file-quest-reward"></a>
### `file:quest.reward`

| Field | Default | Meaning |
|---|---|---|
| `image "<asset>"` | `` | Reward image. |
| `rarity <word>` | `` | Game-owned rarity label. |

<a id="file-quest-stage"></a>
### `file:quest.stage`

| Field | Default | Meaning |
|---|---|---|
| `summary "<i18n key>"` | `` | Summary key. |
| `next <stage id>` | `next stage` | Explicit next stage; references are validated. |
| `fail "<expression>"` | `` | Fail when this condition holds. |
| `objective <id> "<i18n key>" { ... }` | `` | Repeated objectives; ids unique per stage. |

<a id="file-quest-objective"></a>
### `file:quest.objective`

| Field | Default | Meaning |
|---|---|---|
| `arg <name> "<expression>"` | `` | Repeated localized message arguments. |
| `target <integer, 1..1000000>` | `1` | Required progress count. |
| `when "<expression>"` | `` | Complete when condition holds. |
| `on <event>` | `` | Progress on this event. |
| `where "<expression>"` | `` | Filter the declared event. |
| `optional ` | `false` | Not required for stage completion. |
| `hidden ` | `false` | Hide until first progress. |
| `marker <x> [<y> <z>]` | `` | World point in metres; omitted y/z are zero. |

<a id="file-rig"></a>
### `file:rig`

An authored `rig` asset, cooked by `rig`. Input: `.ogt` under assets/.

Field coverage: parser-owned rig fields and every standard humanoid bone name accepted by the same reader. This maps a skeleton; it does not generate one.

| Field | Default | Meaning |
|---|---|---|
| `scheme <naming scheme>` | `auto` | Node naming scheme. Explicit bone mappings override it. |
| `footHeight <left> [<right>]` | `rest foot heights` | Ankle heights above ground, -10..10 metres; one value applies to both feet. |
| `hips "<node name>" \| none` | `scheme mapping` | Explicit node override, or no such bone; each standard bone occurs once. |
| `spine "<node name>" \| none` | `scheme mapping` | Explicit node override, or no such bone; each standard bone occurs once. |
| `chest "<node name>" \| none` | `scheme mapping` | Explicit node override, or no such bone; each standard bone occurs once. |
| `upperChest "<node name>" \| none` | `scheme mapping` | Explicit node override, or no such bone; each standard bone occurs once. |
| `neck "<node name>" \| none` | `scheme mapping` | Explicit node override, or no such bone; each standard bone occurs once. |
| `head "<node name>" \| none` | `scheme mapping` | Explicit node override, or no such bone; each standard bone occurs once. |
| `leftShoulder "<node name>" \| none` | `scheme mapping` | Explicit node override, or no such bone; each standard bone occurs once. |
| `leftUpperArm "<node name>" \| none` | `scheme mapping` | Explicit node override, or no such bone; each standard bone occurs once. |
| `leftLowerArm "<node name>" \| none` | `scheme mapping` | Explicit node override, or no such bone; each standard bone occurs once. |
| `leftHand "<node name>" \| none` | `scheme mapping` | Explicit node override, or no such bone; each standard bone occurs once. |
| `rightShoulder "<node name>" \| none` | `scheme mapping` | Explicit node override, or no such bone; each standard bone occurs once. |
| `rightUpperArm "<node name>" \| none` | `scheme mapping` | Explicit node override, or no such bone; each standard bone occurs once. |
| `rightLowerArm "<node name>" \| none` | `scheme mapping` | Explicit node override, or no such bone; each standard bone occurs once. |
| `rightHand "<node name>" \| none` | `scheme mapping` | Explicit node override, or no such bone; each standard bone occurs once. |
| `leftUpperLeg "<node name>" \| none` | `scheme mapping` | Explicit node override, or no such bone; each standard bone occurs once. |
| `leftLowerLeg "<node name>" \| none` | `scheme mapping` | Explicit node override, or no such bone; each standard bone occurs once. |
| `leftFoot "<node name>" \| none` | `scheme mapping` | Explicit node override, or no such bone; each standard bone occurs once. |
| `leftToes "<node name>" \| none` | `scheme mapping` | Explicit node override, or no such bone; each standard bone occurs once. |
| `rightUpperLeg "<node name>" \| none` | `scheme mapping` | Explicit node override, or no such bone; each standard bone occurs once. |
| `rightLowerLeg "<node name>" \| none` | `scheme mapping` | Explicit node override, or no such bone; each standard bone occurs once. |
| `rightFoot "<node name>" \| none` | `scheme mapping` | Explicit node override, or no such bone; each standard bone occurs once. |
| `rightToes "<node name>" \| none` | `scheme mapping` | Explicit node override, or no such bone; each standard bone occurs once. |
| `leftThumbMetacarpal "<node name>" \| none` | `scheme mapping` | Explicit node override, or no such bone; each standard bone occurs once. |
| `leftThumbProximal "<node name>" \| none` | `scheme mapping` | Explicit node override, or no such bone; each standard bone occurs once. |
| `leftThumbDistal "<node name>" \| none` | `scheme mapping` | Explicit node override, or no such bone; each standard bone occurs once. |
| `leftIndexProximal "<node name>" \| none` | `scheme mapping` | Explicit node override, or no such bone; each standard bone occurs once. |
| `leftIndexIntermediate "<node name>" \| none` | `scheme mapping` | Explicit node override, or no such bone; each standard bone occurs once. |
| `leftIndexDistal "<node name>" \| none` | `scheme mapping` | Explicit node override, or no such bone; each standard bone occurs once. |
| `leftMiddleProximal "<node name>" \| none` | `scheme mapping` | Explicit node override, or no such bone; each standard bone occurs once. |
| `leftMiddleIntermediate "<node name>" \| none` | `scheme mapping` | Explicit node override, or no such bone; each standard bone occurs once. |
| `leftMiddleDistal "<node name>" \| none` | `scheme mapping` | Explicit node override, or no such bone; each standard bone occurs once. |
| `leftRingProximal "<node name>" \| none` | `scheme mapping` | Explicit node override, or no such bone; each standard bone occurs once. |
| `leftRingIntermediate "<node name>" \| none` | `scheme mapping` | Explicit node override, or no such bone; each standard bone occurs once. |
| `leftRingDistal "<node name>" \| none` | `scheme mapping` | Explicit node override, or no such bone; each standard bone occurs once. |
| `leftLittleProximal "<node name>" \| none` | `scheme mapping` | Explicit node override, or no such bone; each standard bone occurs once. |
| `leftLittleIntermediate "<node name>" \| none` | `scheme mapping` | Explicit node override, or no such bone; each standard bone occurs once. |
| `leftLittleDistal "<node name>" \| none` | `scheme mapping` | Explicit node override, or no such bone; each standard bone occurs once. |
| `rightThumbMetacarpal "<node name>" \| none` | `scheme mapping` | Explicit node override, or no such bone; each standard bone occurs once. |
| `rightThumbProximal "<node name>" \| none` | `scheme mapping` | Explicit node override, or no such bone; each standard bone occurs once. |
| `rightThumbDistal "<node name>" \| none` | `scheme mapping` | Explicit node override, or no such bone; each standard bone occurs once. |
| `rightIndexProximal "<node name>" \| none` | `scheme mapping` | Explicit node override, or no such bone; each standard bone occurs once. |
| `rightIndexIntermediate "<node name>" \| none` | `scheme mapping` | Explicit node override, or no such bone; each standard bone occurs once. |
| `rightIndexDistal "<node name>" \| none` | `scheme mapping` | Explicit node override, or no such bone; each standard bone occurs once. |
| `rightMiddleProximal "<node name>" \| none` | `scheme mapping` | Explicit node override, or no such bone; each standard bone occurs once. |
| `rightMiddleIntermediate "<node name>" \| none` | `scheme mapping` | Explicit node override, or no such bone; each standard bone occurs once. |
| `rightMiddleDistal "<node name>" \| none` | `scheme mapping` | Explicit node override, or no such bone; each standard bone occurs once. |
| `rightRingProximal "<node name>" \| none` | `scheme mapping` | Explicit node override, or no such bone; each standard bone occurs once. |
| `rightRingIntermediate "<node name>" \| none` | `scheme mapping` | Explicit node override, or no such bone; each standard bone occurs once. |
| `rightRingDistal "<node name>" \| none` | `scheme mapping` | Explicit node override, or no such bone; each standard bone occurs once. |
| `rightLittleProximal "<node name>" \| none` | `scheme mapping` | Explicit node override, or no such bone; each standard bone occurs once. |
| `rightLittleIntermediate "<node name>" \| none` | `scheme mapping` | Explicit node override, or no such bone; each standard bone occurs once. |
| `rightLittleDistal "<node name>" \| none` | `scheme mapping` | Explicit node override, or no such bone; each standard bone occurs once. |

Accepted schemes: `auto`, `mixamo`, `vrm`, `unity`, `rigify`, `ue`, `blender`, `soma`.


<a id="file-rig2d"></a>
### `file:rig2d`

An authored `rig2d` asset, cooked by `rig2d`. Input: `.ogt` under assets/.

Field coverage: shared runtime grammar plus cook-only layered-art input. Coordinates are pixels, +y up, angles counterclockwise degrees. Key easing is linear by default, step, or curve x1 y1 x2 y2 with optional further anchors/control points. Named attachment/constraint/bone references are validated; armatures share the same body grammar.

| Field | Default | Meaning |
|---|---|---|
| `texture "<image>"` | `required` | Shared atlas image. |
| `size <width> <height>` | `from image` | Atlas pixels. |
| `filter nearest \| linear` | `linear` | Atlas sampling. |
| `unit <pixels > 0>` | `1` | Pixels per world metre. |
| `bone <name> { ... }` | `required` | Parent-first setup bones. |
| `slot <name> { ... }` | `` | Slots in draw order. |
| `region <name> { ... }` | `` | Rectangular attachment. |
| `mesh <name> { ... }` | `` | Weighted mesh attachment. |
| `box <name> { ... }` | `` | Polygon attachment, used for queries. |
| `path <name> { ... }` | `` | Cubic path attachment, three vertices per knot. |
| `child <name> { ... }` | `` | Child armature attachment. |
| `ik <name> { ... }` | `` | IK constraint. |
| `transform <name> { ... }` | `` | Transform constraint. |
| `along <name> { ... }` | `` | Path constraint. |
| `skin <name> { swap <attachment> <attachment or none> ... }` | `` | Named attachment swaps; default is reserved. |
| `armature <name> { ... }` | `` | Child armature using the same bone/slot/attachment/constraint/clip grammar. |
| `clip <name> { ... }` | `` | Authored animation keys and events. |
| `dragonbones "<export.json>"` | `` | Cook-only DragonBones 5.5 input; exclusive with authored bones/attachments/clips. |
| `atlas "<atlas.json>"` | `` | Cook-only imported atlas metadata. |

<a id="file-rig2d-bone"></a>
### `file:rig2d.bone`

| Field | Default | Meaning |
|---|---|---|
| `parent <bone>` | `` | Parent bone declared earlier. |
| `at <x> <y>` | `0 0` | Setup translation, pixels, +y up. |
| `rotation <degrees>` | `0` | Counterclockwise setup rotation. |
| `skew <degrees>` | `0` | Additional rotation of y axis. |
| `scale <x> <y>` | `1 1` | Setup scale. |
| `length <pixels>` | `0` | Bone length. |

<a id="file-rig2d-slot"></a>
### `file:rig2d.slot`

| Field | Default | Meaning |
|---|---|---|
| `bone <bone>` | `required` | Owning bone. |
| `attachment <attachment or none>` | `none` | Setup display. |
| `color <r> <g> <b> <a>` | `1 1 1 1` | Linear colour multiplier. |
| `offset <r> <g> <b> <a>` | `0 0 0 0` | Additive colour offset. |

<a id="file-rig2d-region"></a>
### `file:rig2d.region`

| Field | Default | Meaning |
|---|---|---|
| `slot <slot>` | `required` | Owning slot. |
| `rect <x> <y> <w> <h>` | `required` | Atlas rectangle, top-left pixels. |
| `at <x> <y>` | `0 0` | Attachment translation, pixels. |
| `rotation <degrees>` | `0` | Counterclockwise rotation. |
| `skew <degrees>` | `0` | Y-axis skew. |
| `size <w> <h>` | `rectangle size` | Drawn dimensions, pixels. |

<a id="file-rig2d-mesh"></a>
### `file:rig2d.mesh`

| Field | Default | Meaning |
|---|---|---|
| `slot <slot>` | `required` | Owning slot. |
| `vertex <u> <v> <x> <y> \| <u> <v> <bone> <x> <y> <weight> ...` | `` | Unweighted slot-local pixels or repeated bone-local influences; weights sum to one. |
| `triangles <index> ...` | `` | Nonnegative integer indices, triples. |

<a id="file-rig2d-box"></a>
### `file:rig2d.box`

| Field | Default | Meaning |
|---|---|---|
| `slot <slot>` | `required` | Owning slot. |
| `polygon <x> <y> ...` | `` | Polygon vertices, pixels. |

<a id="file-rig2d-path"></a>
### `file:rig2d.path`

| Field | Default | Meaning |
|---|---|---|
| `slot <slot>` | `required` | Owning slot. |
| `closed true \| false` | `false` | Closed spline. |
| `constant true \| false` | `true` | Measure distance by arc length. |
| `lengths <pixels> ...` | `` | Curve ends, required if constant is false. |
| `vertex <x> <y> \| <bone> <x> <y> <weight> ...` | `` | Handle-in/knot/handle-out, at least two knots; unweighted or bone influences. |

<a id="file-rig2d-child"></a>
### `file:rig2d.child`

| Field | Default | Meaning |
|---|---|---|
| `slot <slot>` | `required` | Owning slot. |
| `armature <name>` | `required` | Child armature in this file. |
| `clip <name>` | `` | Fallback clip when no inherited name matches. |
| `at <x> <y>` | `0 0` | Translation, pixels. |
| `rotation <degrees>` | `0` | Counterclockwise rotation. |
| `skew <degrees>` | `0` | Y-axis skew. |
| `scale <x> <y>` | `1 1` | Scale factors. |

<a id="file-rig2d-ik"></a>
### `file:rig2d.ik`

| Field | Default | Meaning |
|---|---|---|
| `bones <bone> ...` | `required` | Contiguous parent-first chain. |
| `target <bone>` | `required` | Target bone. |
| `bend clockwise \| counterclockwise` | `clockwise` | IK bend side. |
| `weight <number>` | `1` | Constraint weight, clamped 0..1. |

<a id="file-rig2d-transform"></a>
### `file:rig2d.transform`

| Field | Default | Meaning |
|---|---|---|
| `bones <bone> ...` | `required` | Affected bones. |
| `target <bone>` | `required` | Target bone. |
| `rotation <degrees>` | `0` | Rotation offset. |
| `at <x> <y>` | `0 0` | Translation offset, pixels. |
| `scale <x> <y>` | `0 0` | Scale offset. |
| `shear <degrees>` | `0` | Shear offset. |
| `mix <rotation> <translation> <scale> <shear>` | `1 1 1 1` | Weights clamped 0..1. |

<a id="file-rig2d-along"></a>
### `file:rig2d.along`

| Field | Default | Meaning |
|---|---|---|
| `path <attachment>` | `required` | Path attachment. |
| `bones <bone> ...` | `required` | Affected bones. |
| `position <value> fixed \| percent` | `0 percent` | Start position in pixels or path fraction. |
| `spacing <value> length \| fixed \| percent` | `0 length` | Bone spacing. |
| `rotate tangent \| chain \| chainScale <degrees>` | `tangent 0` | Rotation mode and offset. |
| `mix <rotation> <translation>` | `1 1` | Weights clamped 0..1. |

<a id="file-rig2d-skin"></a>
### `file:rig2d.skin`

| Field | Default | Meaning |
|---|---|---|
| `swap <attachment> <attachment or none>` | `` | Repeated swaps between attachments belonging to the same slot. |

<a id="file-rig2d-clip"></a>
### `file:rig2d.clip`

| Field | Default | Meaning |
|---|---|---|
| `duration <seconds>` | `0` | Clip duration; keys must be ordered and within duration. |
| `loop true \| false` | `true` | Loop playback. |
| `event <seconds> "<name>"` | `` | Repeated named markers. |
| `bone <name> { ... }` | `` | Bone timeline. |
| `slot <name> { ... }` | `` | Slot timeline. |
| `deform <attachment> { key ... }` | `` | Mesh/path influence offsets. |
| `order <seconds> [<slot> ...]` | `` | Every slot once in draw order, or empty for setup order. |
| `ik <constraint> { key ... }` | `` | IK weight and bend timeline. |

<a id="file-rig2d-clip-bone"></a>
### `file:rig2d.clip.bone`

| Field | Default | Meaning |
|---|---|---|
| `rotate <seconds> <degrees> [clockwise\|counterclockwise <turns>] [<ease>]` | `` | Relative setup rotation; shortest arc unless explicit nonnegative integer extra turns. |
| `skew <seconds> <degrees> [<ease>]` | `` | Relative setup skew. |
| `translate <seconds> <x> <y> [<ease>]` | `` | Offset from setup, pixels. |
| `scale <seconds> <x> <y> [<ease>]` | `` | Multiplier of setup scale. |

<a id="file-rig2d-clip-slot"></a>
### `file:rig2d.clip.slot`

| Field | Default | Meaning |
|---|---|---|
| `attachment <seconds> <attachment or none>` | `` | Held display key. |
| `color <seconds> <r> <g> <b> <a> [<ease>]` | `` | Colour multiplier. |
| `offset <seconds> <r> <g> <b> <a> [<ease>]` | `` | Additive colour. |

<a id="file-rig2d-clip-deform"></a>
### `file:rig2d.clip.deform`

| Field | Default | Meaning |
|---|---|---|
| `key <seconds> <x> <y> ... [<ease>]` | `` | One bone-local pixel offset pair per influence, in vertex order. |

<a id="file-rig2d-clip-ik"></a>
### `file:rig2d.clip.ik`

| Field | Default | Meaning |
|---|---|---|
| `key <seconds> <weight> clockwise\|counterclockwise [<ease>]` | `` | Eased weight, held bend side. |

<a id="file-rig2d-parts"></a>
### `file:rig2d.parts`

| Field | Default | Meaning |
|---|---|---|
| `parts "<file.psd or folder/>"` | `required` | Layered art input; part files enter the cook stamp. |
| `template <existing rig layout>` | `required` | Existing art-contract skeleton layout, not generated gameplay. |
| `pivot <part> <x> <y>` | `` | Repeated authored joints in canvas pixels. |
| `unit <pixels > 0>` | `from art` | Pixels per world metre. |
| `filter nearest \| linear` | `linear` | Atlas sampling. |

<a id="file-scatter"></a>
### `file:scatter`

An authored `scatter` asset, cooked by `scene`. Input: `.ogt` under assets/.

Field coverage: complete parser line table. Output: a normal scene; load with og.load.scene or reference from a scene. Model placements reuse normal model instancing; prefab placements expand their authored entities. No distribution, density, model or gameplay rule is inferred. Paths are asset-root relative.

| Field | Default | Meaning |
|---|---|---|
| `model "<asset>"` | `` | Model to place at every transform; exclusive with prefab. |
| `prefab "<asset>"` | `` | Prefab to instantiate at every transform; exclusive with model. |
| `point <x> <y> <z>` | `` | Repeat for authored points, exclusive with generate. Positions use scene/terrain-local metres. |
| `generate { ... }` | `` | Returns ScatterSource positions, optional quaternions and scales. May compose shared poissonDisk; no implicit distribution. |
| `project "<terrain.ogt>"` | `` | Add the named terrain surface height to each point's y. Reject off-surface points and holes, not clamp or discard them. |
| `align true \| false` | `false` | Rotate local up to the projected surface normal, before the authored quaternion; needs project. |

<a id="file-scatter-generate"></a>
### `file:scatter.generate`

Returns ScatterSource: flat positions, optional xyzw quaternions and xyz scales with one per point; at most 1,000,000 points. Authored point lines and generation are exclusive. Project adds the authoritative terrain height to y and can align local up before authored rotation. Terrain and placements share local coordinates; off-terrain points and holes fail cook.

| Field | Default | Meaning |
|---|---|---|
| `script "<file.ts>"` | `required` | Synchronous default export; static relative imports and explicit inputs are tracked. |
| `seed <u32>` | `0` | Explicit seed passed to the generator. |
| `params { ... }` | `{}` | Named scalar, array or nested object parameters. |
| `input "<name>" "<path>"` | `` | Repeated input bytes, relative to the authored asset, available as ctx.inputs[name]. |

<a id="file-scene"></a>
### `file:scene`

An authored `scene` asset, cooked by `scene`. Input: `.ogt` under assets/.

Field coverage: parser-owned declarations and reflected component/resource fields. Component blocks take schema field names: numbers/vectors, enum words, booleans or quoted strings; bounded lists may repeat. Query the reflected record with og api show. Game-owned records use the game's schema, never a gameplay template. Output: a normal scene, loaded with og.load.scene.

| Field | Default | Meaning |
|---|---|---|
| `name "<name>"` | `` | Scene name. |
| `entity "<name>" { ... }` | `` | Repeated entity trees, parents before children. |
| `resource <record> { ... }` | `` | One block per reflected resource; fields use its schema. |
| `stream "<group>" { ... }` | `` | Asset streaming group. |
| `hlod { ... }` | `` | Explicit distant static-model proxy settings. |
| `path "<name>" { ... }` | `` | Named local-space path, shared with timeline paths. |
| `generate { ... }` | `` | Deterministic SceneSource appended before normal scene validation. |

<a id="file-scene-entity"></a>
### `file:scene.entity`

| Field | Default | Meaning |
|---|---|---|
| `entity "<name>" { ... }` | `` | Repeated children in this entity's space. |
| `model "<asset>"` | `` | Instantiate a model; its geometry/material handles are resolved at load. |
| `scene "<asset>"` | `` | Place one sub-scene; dependency cycles fail. |
| `static true \| false` | `false` | Mark this entity's model for baked lighting and HLOD. |
| `prefab "<asset>"` | `` | Expand a prefab; remaining entries are explicit overrides. |
| `override "<node path>" { ... }` | `` | Instance-only component overrides on a descendant. |
| `remove "<node path>" [<component>]` | `` | Instance-only removal of a subtree or component. |

<a id="file-scene-stream"></a>
### `file:scene.stream`

| Field | Default | Meaning |
|---|---|---|
| `asset "<asset>"` | `` | Repeated asset ids; each asset belongs to at most one group. |
| `distance <x> <y> <z> <radius>` | `manual` | Distance trigger in scene metres; exclusive with volume and cell. |
| `unload <metres>` | `radius * 1.25` | Unload distance for a distance trigger. |
| `volume "<entity>"` | `manual` | Trigger inside the named entity's box. |
| `priority <i32>` | `0` | Higher loads first and unloads last. |
| `cell ` | `` | Sub-scene cell group, at most one, without other trigger. |

<a id="file-scene-hlod"></a>
### `file:scene.hlod`

| Field | Default | Meaning |
|---|---|---|
| `size <4..4096 metres>` | `64` | Leaf cell side; doubles at each level. |
| `error <metres > 0, <= size / 4>` | `size / 256` | Maximum leaf proxy displacement; zero requests the derived default. |
| `texels <64..4096>` | `1024` | Longest texture side in pixels. |

<a id="file-scene-path"></a>
### `file:scene.path`

| Field | Default | Meaning |
|---|---|---|
| `kind catmullRom \| bezier \| linear` | `catmullRom` | Spline kind. |
| `closed true \| false` | `false` | Join the last point to the first. |
| `tension <number>` | `0.5` | Catmull-Rom tangent scale. |
| `point <x> <y> <z>` | `required` | Repeated local points, metres; point count must satisfy the spline kind. |

<a id="file-scene-generate"></a>
### `file:scene.generate`

Returns SceneSource: {entities, resources?}. Each entity has name, optional model/scene/prefab asset paths, static boolean, components and children. Component/resource objects map reflected field names to numbers, booleans, strings or flat arrays. No world, callbacks or runtime handles. Generated entities append after authored entities; a resource cannot be both authored and generated. Normal prefab expansion, schema, reference and sub-scene cycle checks apply.

| Field | Default | Meaning |
|---|---|---|
| `script "<file.ts>"` | `required` | Synchronous default export; static relative imports and explicit inputs are tracked. |
| `seed <u32>` | `0` | Explicit seed passed to the generator. |
| `params { ... }` | `{}` | Named scalar, array or nested object parameters. |
| `input "<name>" "<path>"` | `` | Repeated input bytes, relative to the authored asset, available as ctx.inputs[name]. |

<a id="file-sfx"></a>
### `file:sfx`

Author `content/**/*.ts` with `export default file["sfx"]({...})` from `@origingame/engine/author`. Project/module types are generated by `og api gen`; canonical output IDs are `<module>/<content-relative-stem>`. This domain does not accept OGT sources.

```ts
/** Procedural sound: optional preset plus explicit oscillator/sample layers.
At least one preset/layer and maximum rendered duration are domain checks. */
export type Input = ({
readonly "layers"?: ((ReadonlyArray<(InputDefinitions["SynthLayer"])> | (number extends ((InputDefinitions["SynthLayer"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never)) | (null));
readonly "length"?: ((number) | (null));
readonly "pitch"?: ((number) | (null));
readonly "preset"?: (((InputDefinitions["SfxPreset"])) | ((null)));
readonly "seed"?: ((number) | (null));
readonly "volume"?: ((number) | (null));
});
export interface InputDefinitions {
readonly "Pcm": (number);
readonly "SampleInput": (((({
readonly "kind": ("file") & (string);
readonly "looping"?: ((boolean) | (null));
readonly "reference"?: ((number) | (null));
/** Explicit original audio input, relative to the owning module. */
readonly "source": (string);
})) & { readonly "rate"?: never; readonly "samples"?: never; }) | ((({
readonly "kind": ("pcm") & (string);
readonly "looping"?: ((boolean) | (null));
readonly "rate": (number);
readonly "reference"?: ((number) | (null));
readonly "samples": (ReadonlyArray<(InputDefinitions["Pcm"])> | (number extends ((InputDefinitions["Pcm"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
})) & { readonly "source"?: never; }));
readonly "SfxPreset": ("coin" | "pickup" | "powerup" | "jump" | "land" | "footstep" | "hit" | "hurt" | "explosion" | "laser" | "whoosh" | "door" | "death" | "click" | "select" | "back" | "error" | "levelup") & (string);
/** A mono sample replaces the oscillator; sample layers reject FM. */
readonly "SynthLayer": ({
readonly "arp"?: ((number) | (null));
readonly "arpAt"?: ((number) | (null));
readonly "attack"?: ((number) | (null));
readonly "decay"?: ((number) | (null));
readonly "delay"?: ((number) | (null));
readonly "drive"?: ((number) | (null));
readonly "duty"?: ((number) | (null));
readonly "fmFreq"?: ((number) | (null));
readonly "fmIndex"?: ((number) | (null));
readonly "freq"?: ((number) | (null));
readonly "freqEnd"?: ((number) | (null));
readonly "gain"?: ((number) | (null));
readonly "highpass"?: ((number) | (null));
readonly "lowpass"?: ((number) | (null));
readonly "lpSlide"?: ((number) | (null));
readonly "punch"?: ((number) | (null));
readonly "q"?: ((number) | (null));
readonly "release"?: ((number) | (null));
readonly "sample"?: (((InputDefinitions["SampleInput"])) | ((null)));
readonly "slide"?: ((number) | (null));
readonly "sustain"?: ((number) | (null));
readonly "sustainLevel"?: ((number) | (null));
readonly "vibDepth"?: ((number) | (null));
readonly "vibRate"?: ((number) | (null));
readonly "wave"?: (((InputDefinitions["Wave"])) | ((null)));
});
readonly "Wave": ("sine" | "square" | "saw" | "triangle" | "noise") & (string);
}

```

Output: an ordinary mono 48 kHz sound loaded with og.load.sound. A preset or at least one layer is required; preset layers precede custom layers. Rendering is limited to 30 seconds. A sample is either inline PCM or an explicit canonical path in the owning module's assets/ or sources/; stereo recordings average to mono. Sample bytes participate in cook stamps. FM is oscillator-only. og sfx writes TS declarations or listening WAVs.

<a id="file-shader"></a>
### `file:shader`

An authored `shader` asset, cooked by `shader`. Input: `.ogt` under assets/.

Field coverage: complete shared shader-family parser table, filtered by document type. Cook validates the same WGSL contract on every supported backend. See render or ui-materials for WGSL entry structures and callable functions.

| Field | Default | Meaning |
|---|---|---|
| `source "<WGSL file>"` | `` | One source file; exclusive with code. Cook tracks its bytes and inlines it. |
| `code "<WGSL text>"` | `` | Inline WGSL; exclusive with source. One input is required. |
| `param <name> <float \| vec2 \| vec3 \| vec4 \| color> [<defaults> ...]` | `zero components` | Repeat shader parameters; defaults are absent or exactly the type's finite component count. Color is linear RGB. Backend checks validate names, count and WGSL contract. |
| `mode hooks \| sprite \| full [lit \| unlit]` | `hooks` | Select the material shader contract. Full defaults to lit. Post and UI material types do not accept mode. |

<a id="file-sound"></a>
### `file:sound`

Author `content/**/*.ts` with `export default file["sound"]({...})` from `@origingame/engine/author`. Project/module types are generated by `og api gen`; canonical output IDs are `<module>/<content-relative-stem>`. This domain does not accept OGT sources.

```ts
export type Input = ({
/** One mono channel or interleaved stereo. */
readonly "channels": (number);
/** Sample rate in hertz; no resampling before the codec import. */
readonly "rate": (number);
/** Finite PCM samples, at least one complete interleaved frame. */
readonly "samples": (ReadonlyArray<(number)> | (number extends ((number)) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
});

```

Output: an ordinary sound loaded with og.load.sound. Explicit rate 8000..192000 Hz, channels 1 or 2, and at least one complete interleaved frame of finite samples are required. Float samples share the WAV importer's clamping and quantization, then the normal sound encoder and automatic streaming. Actual #chunkN outputs are reported by the producer. Oscillator synthesis remains the separate sfx domain.

<a id="file-strings"></a>
### `file:strings`

An authored `strings` asset, cooked by `strings`. Input: `.ogt` under assets/.

Field coverage: complete shared string-table parser line table. Messages use the existing ICU subset; locale key/argument checks still run in cook.

| Field | Default | Meaning |
|---|---|---|
| `locale "<language tag>"` | `required` | Locale, canonicalized on read. |
| `fallback "<language tag>"` | `base table` | Fallback locale, canonicalized on read. |
| `base true \| false` | `false` | Marks the reference table for key and argument checks. |
| `direction ltr \| rtl` | `locale direction` | Explicit text direction override. |
| `fonts "<font asset>" ...` | `` | Ordered font fallback chain for this locale. |
| `string "<key>" "<message>"` | `` | Repeat unique keys with ICU-subset messages; see localization for argument and plural syntax. |

<a id="file-table"></a>
### `file:table`

An authored `table` asset, cooked by `table`. Input: `.ogt` under assets/.

Field coverage: shared table parser structure and value forms. Row field names, types and defaults belong to the game's reflected record. The engine supplies no item, wave or gameplay schema.

| Field | Default | Meaning |
|---|---|---|
| `locale "<language tag>"` | `en` | One non-empty tag for text fields' base language. |
| `row "<id>" { ... }` | `` | Repeat unique, non-empty row ids. Each row field occurs once, has values and has no block. |

<a id="file-table-row"></a>
### `file:table.row`

| Field | Default | Meaning |
|---|---|---|
| `<field> <number \| string \| word> ...` | `` | Plain values are checked against the game's record when loaded. |
| `<field> asset "<path>"` | `` | One asset-root path checked by cook. |
| `<field> text "<key>" "<base message>"` | `` | Localized text; a key cannot have conflicting base messages. |
| `<field> row ["<table asset>"] "<id>" ...` | `` | One or more row references; omitted table refers to this table. Cook checks referenced rows. |

<a id="file-terrain"></a>
### `file:terrain`

An authored `terrain` asset, cooked by `terrain`. Input: `.ogt` under assets/.

Field coverage: parser line tables, including every nested block below. Output: tiled terrain, control pages and collision data, loaded with og.terrain.load.

| Field | Default | Meaning |
|---|---|---|
| `heightmap "<png\|exr>"` | `` | Image input, relative to the terrain then assets/. May be followed by generate processing. Exclusive with generate.script. |
| `generate { ... }` | `` | Noise or script input, followed by hydraulic, thermal, terraces, shore, rivers, biome stages in that order. |
| `height <low> <high>` | `0 100 (EXR: 0 1)` | Metres corresponding to image 0 and 1, or noise range. Not allowed with script; authored fields are not normalised or clipped by processing. |
| `size <x> <z>` | `one metre per sample interval` | Positive finite world extents in metres; rows run -z to +z, columns -x to +x. |
| `control "<image>" ...` | `` | RGBA weights, one image per four layers, all equal size. Exclusive with biome rules. |
| `holes "<image>"` | `` | Black or transparent cells have no surface or collision. |
| `tile <integer 16..1024>` | `128` | Quads per streaming tile side. |
| `heightBlend <number >= 0>` | `0` | Layer height-blend amount. |
| `castShadows true \| false` | `true` | Terrain casts shadows. |
| `collision true \| false` | `true` | Build terrain collision. |
| `lod { ... }` | `` | Drawn clipmap spacing and ring. |
| `layer { ... }` | `` | Repeated material layer; at most sixteen. Past four use texture arrays. |
| `layers "<texture-array.ogt>"` | `` | Albedo array asset. |
| `layerNormals "<texture-array.ogt>"` | `` | Normal array asset. |
| `edit { ... }` | `` | Repeated local-space brush. Edits and stamps run in written order after generate and its final shape projection; explicit edits may intentionally change protected samples. |
| `stamp { ... }` | `` | Repeated placed geometry projected through the shared runtime stamp mechanism. Geometry may be procedural, generated buffers or modelGeometry; model dependencies import first. |

<a id="file-terrain-edit"></a>
### `file:terrain.edit`

Terrain `edit` block fields.

| Field | Default | Meaning |
|---|---|---|
| `mode raise \| lower \| flatten \| smooth \| paint \| hole \| fill` | `required` | The runtime TerrainBrush operation. |
| `center <x> <y> <z>` | `required` | Terrain-local metres. Flatten targets y. |
| `radius <metres > 0>` | `required` | Circular brush radius. |
| `strength <number >= 0>` | `0` | Raise/lower distance in metres; other modes clamp strength to 1. |
| `falloff <0..1>` | `0` | Fraction of radius fading smoothly to zero. |
| `layer <index>` | `0` | Existing material layer for paint. |

<a id="file-terrain-lod"></a>
### `file:terrain.lod`

Terrain `lod` block fields.

| Field | Default | Meaning |
|---|---|---|
| `spacing <number >= 0>` | `0` | Finest spacing in metres; zero uses sample spacing. |
| `ring <integer >= 0>` | `0` | Clipmap quads per side; zero selects the renderer's ring. |

<a id="file-terrain-layer"></a>
### `file:terrain.layer`

Terrain `layer` block fields.

| Field | Default | Meaning |
|---|---|---|
| `albedo "<texture>"` | `` | Albedo asset; alpha holds material height. With arrays use albedoLayer. |
| `normal "<texture>"` | `` | Normal asset; with arrays use normalLayer. |
| `albedoLayer <integer >= 0>` | `layer index` | Albedo-array slice. |
| `normalLayer <integer >= 0>` | `layer index` | Normal-array slice. |
| `color <r> <g> <b>` | `1 1 1` | Linear colour multiplier. |
| `roughness <number 0..1>` | `0.9` | PBR roughness. |
| `tile <metres > 0>` | `4` | World metres per texture repeat. |
| `triplanar true \| false` | `false` | Project texture on three axes. |
| `normalStrength <number 0..4>` | `1` | Normal strength. |
| `slope <low> <high>` | `0 90` | Slope band in degrees. |
| `slopeFade <number >= 0>` | `4` | Slope-band fade in degrees. |
| `altitude <low> <high>` | `-1000000 1000000` | Height band in metres above origin. |
| `altitudeFade <number >= 0>` | `4` | Height-band fade in metres. |
| `projectSlope <degrees 0..90>` | `0` | Use the two facing projections above this slope; zero follows triplanar. |
| `antiTiling <number 0..1>` | `0` | Hex anti-tiling amount. |
| `heightContrast <number 0..1>` | `0` | Height-blend contrast. |
| `macroVariation <number 0..1>` | `0` | Coarse brightness and hue variation. |
| `displacement <metres 0..4>` | `0` | Near-camera displacement from albedo alpha. |
| `cellVariation <number 0..1>` | `0` | Per-cell colour variation from normal alpha. |

<a id="file-terrain-generate"></a>
### `file:terrain.generate`

Terrain `generate` block fields.

| Field | Default | Meaning |
|---|---|---|
| `samples <columns 2..4097> <rows 2..4097>` | `257 257` | Noise grid only. An authored input supplies its dimensions; no implicit resampling. |
| `seed <integer 0..4294967295>` | `1` | Noise, erosion and script seed. |
| `noise { ... }` | `` | Noise input parameters; exclusive with authored input. |
| `mask island [<falloff > 0 and <= 1>] \| ridge [<strength 0..1>]` | `none` | Noise shaping only; island defaults to 0.4, ridge to 1. |
| `erosion { ... }` | `` | Hydraulic droplets; absent means no hydraulic processing. |
| `thermal { ... }` | `` | Thermal talus processing; absent means none. |
| `terraces { ... }` | `` | Terraces; authored fields anchor at zero, noise at its low height. |
| `sea <finite metres>` | `` | Sea level for shore flattening and automatic drainage. |
| `shore <metres 0..1000000>` | `2` | Shore width; requires sea if explicitly written. |
| `rivers { ... }` | `` | Automatic drainage and river carving; absent means none. |
| `channel { ... }` | `` | Repeat explicit valley/river centre lines, carved after automatic stages using the shared parabolic section. |
| `preserve "<field.ogt>"` | `` | Single-channel field matching the height grid; finite weights 0..1 blend the final carved result toward the original base. No resampling. One preserves the original bits, zero is free. |
| `preserveBorder <samples 0..4097>` | `0` | Pin this many boundary samples to the original base, overriding preserve weights; a band covering the whole field pins every sample. |
| `moisture { ... }` | `` | Flow/noise mixture read by biome rules. |
| `biome { ... }` | `` | Repeated rule, writes an existing material layer's control channel. |
| `script "<file.ts>"` | `` | Default synchronous export returns {columns, rows, heights}; heights are metres. Relative imports and explicit inputs are tracked. Exclusive with heightmap/noise/mask/samples. |
| `params { <name> <value> ... }` | `{}` | Script parameters: scalars, arrays or nested blocks. Requires script. |
| `input "<name>" "<path>"` | `` | Repeated explicit script input, delivered as ctx.inputs[name] byte array; relative to the terrain. Requires script. |

<a id="file-terrain-generate.noise"></a>
### `file:terrain.generate.noise`

Terrain `generate.noise` block fields.

| Field | Default | Meaning |
|---|---|---|
| `octaves <integer 1..12>` | `6` | FBM octave count. |
| `frequency <number 0.001..10000>` | `3` | Features across the longer world side. |
| `lacunarity <number 1..8>` | `2` | Frequency multiplier per octave. |
| `gain <number 0..1>` | `0.5` | Amplitude multiplier per octave. |
| `warp <number 0..100>` | `0` | Domain-warp strength in features. |

<a id="file-terrain-generate.erosion"></a>
### `file:terrain.generate.erosion`

Terrain `generate.erosion` block fields.

| Field | Default | Meaning |
|---|---|---|
| `drops <integer 0..10000000>` | `40000` | Seeded droplet count. |
| `lifetime <integer 1..1000>` | `30` | Steps per droplet. |
| `radius <integer 1..16>` | `3` | Erosion brush radius in samples. |
| `inertia <number 0..1>` | `0.05` | Share of previous direction. |
| `capacity <number 0..1000>` | `4` | Sediment capacity. |
| `deposition <number 0..1>` | `0.3` | Excess sediment deposition fraction. |
| `erosion <number 0..1>` | `0.3` | Unused sediment capacity eroded per step. |
| `evaporation <number 0..1>` | `0.01` | Water evaporated per step. |
| `gravity <number 0..1000>` | `4` | Droplet acceleration. |

<a id="file-terrain-generate.thermal"></a>
### `file:terrain.generate.thermal`

Terrain `generate.thermal` block fields.

| Field | Default | Meaning |
|---|---|---|
| `talus <degrees 0..89>` | `35` | Angle above which material slumps. |
| `iterations <integer 0..10000>` | `20` | Thermal iterations. |
| `rate <number 0..1>` | `0.5` | Fraction of excess moved each iteration. |

<a id="file-terrain-generate.terraces"></a>
### `file:terrain.generate.terraces`

Terrain `generate.terraces` block fields.

| Field | Default | Meaning |
|---|---|---|
| `step <metres 0.001..1000000>` | `10` | Terrace step height. |
| `blend <number 0..1>` | `0.4` | Smooth fraction of each step. |

<a id="file-terrain-generate.rivers"></a>
### `file:terrain.generate.rivers`

Terrain `generate.rivers` block fields.

| Field | Default | Meaning |
|---|---|---|
| `threshold <integer 1..4294967295>` | `400` | Contributing samples at which a river starts. |
| `depth <metres 0..1000000>` | `2` | Full river depth. |
| `width <metres 0..1000000>` | `6` | Full river width. |

<a id="file-terrain-generate.channel"></a>
### `file:terrain.generate.channel`

Terrain `generate.channel` block fields.

| Field | Default | Meaning |
|---|---|---|
| `point <x> <y> <z>` | `` | Repeat at least two terrain-local metre points; consecutive xz positions must differ. Shared endpoints specify a network; no branch is inferred. |
| `width <metres 0.001..1000000>` | `required` | Full width, parabolic from line y minus depth to the original ground; only lowers. |
| `depth <metres 0..1000000>` | `0` | Bed below the line y; zero puts a valley floor on the line. |
| `river true \| false` | `false` | Require non-rising y downstream and count declared segment lengths in riverLength; intersections take the lower bed. |

<a id="file-terrain-generate.moisture"></a>
### `file:terrain.generate.moisture`

Terrain `generate.moisture` block fields.

| Field | Default | Meaning |
|---|---|---|
| `flow <number 0..1000000>` | `1` | Gathered-flow weight. |
| `scale <number 0.001..1000000000>` | `64` | Contributing samples for half the flow contribution. |
| `noise <number 0..1000000>` | `0.5` | Noise weight. |
| `frequency <number 0.001..10000>` | `6` | Moisture-noise features across terrain. |

<a id="file-terrain-generate.biome"></a>
### `file:terrain.generate.biome`

Terrain `generate.biome` block fields.

| Field | Default | Meaning |
|---|---|---|
| `layer <integer 0..15>` | `required` | Existing material layer to paint. |
| `weight <number 0..1>` | `1` | Rule weight. |
| `altitude <low> <high>` | `-1000000 1000000` | Ascending finite height band in metres. |
| `altitudeFade <metres 0..1000000>` | `2` | Height-band fade. |
| `slope <low> <high>` | `0 90` | Ascending finite slope band in degrees. |
| `slopeFade <degrees 0..90>` | `4` | Slope-band fade. |
| `moisture <low> <high>` | `0 1` | Ascending finite moisture band. |
| `moistureFade <number 0..1>` | `0.1` | Moisture-band fade. |


<a id="file-terrain-stamp"></a>
### `file:terrain.stamp`

| Field | Default | Meaning |
|---|---|---|
| `mode carve \| hole` | `carve` | Lower to mesh underside or cut covered cells; no runtime stamp id is persisted. |
| `depth <metres >= 0>` | `0` | Additional carve depth inside the footprint; its edge meets the underside, as at runtime. |
| `position <x> <y> <z>` | `0 0 0` | Translation in metres. |
| `quaternion <x> <y> <z> <w>` | `0 0 0 1` | Nonzero quaternion, normalized on import. |
| `scale <x> <y> <z>` | `1 1 1` | Nonzero scale; negative components bake reflected triangles. |
| `mirror none \| x \| y \| z` | `none` | Reflect geometry, correcting winding and tangent handedness. |
| `modelGeometry { ... }` | `one geometry required` | Geometry fields are the same as file:model.part.<geometry>. |
| `bufferGeometry { ... }` | `one geometry required` | Geometry fields are the same as file:model.part.<geometry>. |
| `boxGeometry { ... }` | `one geometry required` | Geometry fields are the same as file:model.part.<geometry>. |
| `planeGeometry { ... }` | `one geometry required` | Geometry fields are the same as file:model.part.<geometry>. |
| `sphereGeometry { ... }` | `one geometry required` | Geometry fields are the same as file:model.part.<geometry>. |
| `cylinderGeometry { ... }` | `one geometry required` | Geometry fields are the same as file:model.part.<geometry>. |
| `coneGeometry { ... }` | `one geometry required` | Geometry fields are the same as file:model.part.<geometry>. |
| `capsuleGeometry { ... }` | `one geometry required` | Geometry fields are the same as file:model.part.<geometry>. |
| `torusGeometry { ... }` | `one geometry required` | Geometry fields are the same as file:model.part.<geometry>. |
| `ringGeometry { ... }` | `one geometry required` | Geometry fields are the same as file:model.part.<geometry>. |
| `circleGeometry { ... }` | `one geometry required` | Geometry fields are the same as file:model.part.<geometry>. |
| `tubeGeometry { ... }` | `one geometry required` | Geometry fields are the same as file:model.part.<geometry>. |
| `shapeGeometry { ... }` | `one geometry required` | Geometry fields are the same as file:model.part.<geometry>. |
| `extrudeGeometry { ... }` | `one geometry required` | Geometry fields are the same as file:model.part.<geometry>. |
| `latheGeometry { ... }` | `one geometry required` | Geometry fields are the same as file:model.part.<geometry>. |
| `polyhedronGeometry { ... }` | `one geometry required` | Geometry fields are the same as file:model.part.<geometry>. |
| `icosahedronGeometry { ... }` | `one geometry required` | Geometry fields are the same as file:model.part.<geometry>. |
| `dodecahedronGeometry { ... }` | `one geometry required` | Geometry fields are the same as file:model.part.<geometry>. |
| `octahedronGeometry { ... }` | `one geometry required` | Geometry fields are the same as file:model.part.<geometry>. |
| `tetrahedronGeometry { ... }` | `one geometry required` | Geometry fields are the same as file:model.part.<geometry>. |
| `sweepGeometry { ... }` | `one geometry required` | Geometry fields are the same as file:model.part.<geometry>. |
| `csgGeometry { ... }` | `one geometry required` | Geometry fields are the same as file:model.part.<geometry>. |
| `mergeGeometries { ... }` | `one geometry required` | Geometry fields are the same as file:model.part.<geometry>. |

<a id="file-texture-array"></a>
### `file:texture-array`

Author `content/**/*.ts` with `export default file["texture-array"]({...})` from `@origingame/engine/author`. Project/module types are generated by `og api gen`; canonical output IDs are `<module>/<content-relative-stem>`. This domain does not accept OGT sources.

```ts
/** An ordered array of equally-sized external images. File resolution,
dimensions and encoder compatibility remain texture-array domain checks. */
export type Input = ({
readonly "kind"?: (InputDefinitions["TextureKind"]);
/** Explicit canonical paths relative to the owning module (e.g.
assets/tiles/grass.png), without sibling or asset-root fallback. */
readonly "layers": (ReadonlyArray<(string)> | (number extends ((string)) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
});
export interface InputDefinitions {
readonly "TextureKind": ("color" | "normal" | "linear" | "single" | "pixel") & (string);
}

```

Outputs: one texture array at the primary ID, plus exactly #layer0 through #layerN-1 as ordinary textures. Give 1..256 equally sized images in layer order. Each layer is an explicit canonical path relative to the owning module, such as assets/tiles/grass.png; there is no sibling/assets fallback. Image bytes participate in cook stamps, including warm-cache rebuilds. Encoding kinds are color, normal, linear, single and pixel.

<a id="file-tilemap"></a>
### `file:tilemap`

An authored `tilemap` asset, cooked by `tilemap`. Input: `.ogt` under assets/.

Field coverage: map, layer and object parser line tables, plus the cook generation block. Output: normal tilemap blob, loaded with og.load.tilemap.

| Field | Default | Meaning |
|---|---|---|
| `size <columns> <rows>` | `required` | Positive map dimensions in cells; generation may supply them at cook. |
| `tile <width> <height>` | `required` | Positive tile pixel dimensions. |
| `unit <number>` | `1` | World metres per tile height; tile aspect ratio is retained. |
| `orientation orthogonal \| isometric \| staggered \| hex [pointy \| flat]` | `orthogonal` | Cell placement; no orientation is inferred from tile art. |
| `tileset "<asset>"` | `required` | Repeat ordered tileset asset references; layer.tileset indexes this list. |
| `layer "<name>" { ... }` | `` | Repeat uniquely named tile layers. |
| `object "<name>" { ... }` | `` | Repeat authored objects in cell coordinates. |

<a id="file-tilemap-layer"></a>
### `file:tilemap.layer`

| Field | Default | Meaning |
|---|---|---|
| `tileset <index>` | `0` | Index into the map's tilesets. |
| `z <integer>` | `layer index` | Draw order. |
| `parallax <x> <y>` | `1 1` | Camera-relative parallax factors. |
| `offset <x> <y>` | `0 0` | World-space layer offset. |
| `opacity <number>` | `1` | Layer opacity. |
| `visible true \| false` | `true` | Initial visibility. |
| `collision true \| false` | `false` | Generate colliders from the tileset's collision declarations. |
| `colliderLayers <bits>` | `0` | Collision membership; requires collision true. |
| `colliderMask <bits>` | `0` | Collision mask; requires collision true. |
| `cellColliders true \| false` | `false` | Keep individual cell colliders; requires collision true. |
| `lit true \| false` | `true` | Receive 2D lighting. |
| `repeat x \| y \| x y` | `` | Repeat along the named axes. |
| `terrain "<name>"` | `` | Tileset autotile terrain; rows are occupancy 0/1 instead of tile ids. |
| `legend "<character>" <tile id>` | `` | Repeat character-to-id mappings for quoted rows; -1 is empty. |
| `row <tile id> ... \| "<characters>"` | `empty layer` | Repeat one row per map row, each exactly map width; nonnegative ids are zero based, negative ids empty. |

<a id="file-tilemap-object"></a>
### `file:tilemap.object`

| Field | Default | Meaning |
|---|---|---|
| `kind <name>` | `` | Game-authored object type string. |
| `rect <x> <y> <width> <height>` | `0 0 0 0` | Top-left and size in cells; y down is converted to world y up. |
| `at <x> <y>` | `0 0` | Position in cells, y down; does not change rect size. |
| `prop <key> <value>` | `` | Repeat string-valued game properties; values may be numbers, words or strings. |

<a id="file-tilemap-generate"></a>
### `file:tilemap.generate`

The script returns {width, height, layers: [{name, tiles}]}. Each layer names an authored layer; tiles are row-major tile ids (-1 empty), or 0/1 for a terrain layer. Generated layers cannot also supply rows. A size line, when present, must match. No tileset, collision rule or tile mapping is inferred.

| Field | Default | Meaning |
|---|---|---|
| `script "<file.ts>"` | `required` | Synchronous default export; static relative imports and explicit inputs are tracked. |
| `seed <u32>` | `0` | Explicit seed passed to the generator. |
| `params { ... }` | `{}` | Named scalar, array or nested object parameters. |
| `input "<name>" "<path>"` | `` | Repeated input bytes, relative to the authored asset, available as ctx.inputs[name]. |

<a id="file-tileset"></a>
### `file:tileset`

An authored `tileset` asset, cooked by `tileset`. Input: `.ogt` under assets/.

Field coverage: complete parser line tables. Atlas image, collision shapes, animation and autotiling rules are authored, never inferred.

| Field | Default | Meaning |
|---|---|---|
| `texture "<image>"` | `required` | Atlas texture asset. |
| `grid <tile width> <tile height> <columns> <rows>` | `required` | Positive pixel and grid dimensions. |
| `margin <pixels>` | `0` | Outer atlas margin. |
| `spacing <pixels>` | `0` | Spacing between tiles. |
| `filter nearest \| linear` | `nearest` | Atlas sampler. |
| `solid <tile id> ...` | `` | Mark complete tile rectangles as solid. |
| `tile <id> { ... }` | `` | Per-tile collision and animation. |
| `terrain <name> { ... }` | `` | Uniquely named explicit autotile rule set. |

<a id="file-tileset-tile"></a>
### `file:tileset.tile`

| Field | Default | Meaning |
|---|---|---|
| `polygon <x> <y> ...` | `` | Repeat collision polygons, at least three pixel-coordinate pairs. |
| `collide <x> <y> <width> <height>` | `` | Collision rectangle in pixels; positive size. |
| `solid ` | `` | The full tile collides. |
| `frames <tile id> ...` | `` | Nonempty animation frame ids in playback order. |
| `fps <number > 0>` | `` | Uniform frame rate; requires frames. |
| `durations <seconds > 0> ...` | `` | One duration per frame, overrides fps; requires frames. |

<a id="file-tileset-terrain"></a>
### `file:tileset.terrain`

| Field | Default | Meaning |
|---|---|---|
| `mode blob47 \| bitmask16` | `required` | Neighbour mask encoding. |
| `rule <mask> <tile id>` | `` | Repeat explicit mask-to-tile mappings, each mask only once. |
| `default <tile id>` | `required` | Tile used where no explicit rule matches. |
| `edge filled \| empty` | `filled` | Occupancy outside the map. |

<a id="file-timeline"></a>
### `file:timeline`

An authored `timeline` asset, cooked by `timeline`. Input: `.ogt` under assets/.

| Field | Default | Meaning |
|---|---|---|
| `name "<name>"` | `` | Timeline name. |
| `duration <seconds > 0>` | `last key end` | No key may lie after the duration. |
| `path "<name>" { ... }` | `` | Named scene-local path. |
| `track <kind> ["<entity>"] { ... }` | `` | Repeated tracks; transform/animation/light/visibility/blendshape/material require an entity, property/video allow one, other kinds take none. |

<a id="file-timeline-path"></a>
### `file:timeline.path`

| Field | Default | Meaning |
|---|---|---|
| `kind catmullRom \| bezier \| linear` | `catmullRom` | Spline kind. |
| `closed true \| false` | `false` | Join the last point to the first. |
| `tension <number>` | `0.5` | Catmull-Rom tangent scale. |
| `point <x> <y> <z>` | `required` | Repeated local points, metres; point count must satisfy the spline kind. |

<a id="file-timeline-track-animation"></a>
### `file:timeline.track.animation`

| Field | Default | Meaning |
|---|---|---|
| `key <seconds >= 0> { ... }` | `` | Repeated keys, sorted stably by time; nested fields depend on track kind. |
| `model "<asset>"` | `` | Explicit clip source, checked at cook. |

<a id="file-timeline-track-animation-key"></a>
### `file:timeline.track.animation.key`

| Field | Default | Meaning |
|---|---|---|
| `clip "<name>"` | `required` | Authored clip name. |
| `speed <number>` | `1` | Animation speed multiplier; nested timeline >0; video 0..4. |
| `blendIn <seconds >= 0>` | `0.2` | Clip transition duration. |
| `loop true \| false` | `context-dependent` | Animation/music default true; nested timeline false. |

<a id="file-timeline-track-audio"></a>
### `file:timeline.track.audio`

| Field | Default | Meaning |
|---|---|---|
| `key <seconds >= 0> { ... }` | `` | Repeated keys, sorted stably by time; nested fields depend on track kind. |

<a id="file-timeline-track-audio-key"></a>
### `file:timeline.track.audio.key`

| Field | Default | Meaning |
|---|---|---|
| `sound "<asset>"` | `required` | Cooked sound asset. |
| `volume <number >= 0>` | `1` | Linear gain. |
| `playbackRate <number > 0>` | `1` | Playback rate multiplier. |
| `bus master \| music \| sfx \| ui` | `context-dependent` | Audio defaults sfx; music defaults music. |

<a id="file-timeline-track-blendshape"></a>
### `file:timeline.track.blendshape`

| Field | Default | Meaning |
|---|---|---|
| `key <seconds >= 0> { ... }` | `` | Repeated keys, sorted stably by time; nested fields depend on track kind. |

<a id="file-timeline-track-blendshape-key"></a>
### `file:timeline.track.blendshape.key`

| Field | Default | Meaning |
|---|---|---|
| `ease <word or quoted easing expression>` | `linear` | Shared og-tween easing parser, including cubicBezier, steps and points. |
| `w0..w7 <number>` | `` | Morph weight by index; unspecified weights keep their values. |

<a id="file-timeline-track-camera"></a>
### `file:timeline.track.camera`

| Field | Default | Meaning |
|---|---|---|
| `key <seconds >= 0> { ... }` | `` | Repeated keys, sorted stably by time; nested fields depend on track kind. |
| `rail "<path>"` | `` | Follow a named path, exclusive with follow and key positions. |
| `follow "<entity>"` | `` | Follow an entity, exclusive with rail. |
| `blendIn <seconds >= 0>` | `0` | Entry blend duration. |
| `blendOut <seconds >= 0>` | `0` | Exit blend duration. |

<a id="file-timeline-track-camera-key"></a>
### `file:timeline.track.camera.key`

| Field | Default | Meaning |
|---|---|---|
| `position <x> <y> <z>` | `context-dependent` | Local metres; camera/transform keep unspecified values, vfx defaults to origin. |
| `target <x> <y> <z>` | `` | Look-at target, metres. |
| `at <number>` | `` | Rail parameter. |
| `fov <1..179 degrees>` | `` | Vertical field of view. |
| `roll <degrees>` | `` | Camera roll. |
| `focus <metres > 0>` | `` | Focus distance. |
| `focusOn "<entity>"` | `` | Focus target. |
| `focusRange <metres >= 0>` | `` | Focus range. |
| `ease <word or quoted easing expression>` | `linear` | Shared og-tween easing parser, including cubicBezier, steps and points. |

<a id="file-timeline-track-caption"></a>
### `file:timeline.track.caption`

| Field | Default | Meaning |
|---|---|---|
| `key <seconds >= 0> { ... }` | `` | Repeated keys, sorted stably by time; nested fields depend on track kind. |
| `captions "<file.srt\|vtt\|json>" [<offset seconds>]` | `` | Import cues; files enter cook dependencies. |
| `locale "<tag>" { captions ...; key ... }` | `` | Localized cues, no nested locale. |

<a id="file-timeline-track-caption-key"></a>
### `file:timeline.track.caption.key`

| Field | Default | Meaning |
|---|---|---|
| `end <seconds > key time>` | `context-dependent` | Absolute end; caption can derive it from following cues. |
| `text "<text>"` | `required` | Caption text, not visual styling. |
| `speaker "<speaker>"` | `` | Caption speaker. |
| `position bottom \| top` | `` | Caption placement. |
| `span <name> <position> <length> ["<color>"]` | `` | Repeated spans with unsigned character indices. |

<a id="file-timeline-track-event"></a>
### `file:timeline.track.event`

| Field | Default | Meaning |
|---|---|---|
| `key <seconds >= 0> { ... }` | `` | Repeated keys, sorted stably by time; nested fields depend on track kind. |

<a id="file-timeline-track-event-key"></a>
### `file:timeline.track.event.key`

| Field | Default | Meaning |
|---|---|---|
| `do "<action:arguments>"` | `required` | Command-channel action; action uses lowercase ASCII, digits and underscores. |
| `retroactive true \| false` | `false` | Fire across a seek as defined by timeline playback. |

<a id="file-timeline-track-hotspot"></a>
### `file:timeline.track.hotspot`

| Field | Default | Meaning |
|---|---|---|
| `key <seconds >= 0> { ... }` | `` | Repeated keys, sorted stably by time; nested fields depend on track kind. |

<a id="file-timeline-track-hotspot-key"></a>
### `file:timeline.track.hotspot.key`

| Field | Default | Meaning |
|---|---|---|
| `ease <word or quoted easing expression>` | `linear` | Shared og-tween easing parser, including cubicBezier, steps and points. |
| `id "<1..63 byte id>"` | `required` | Stable hotspot id. |
| `rect <x> <y> <width> <height>` | `` | Fractions from top-left, positive size within the frame; exclusive with off true. |
| `shape rect \| circle` | `rect` | Hit shape. |
| `kind choice \| inspect` | `choice` | Hotspot kind. |
| `label "<label>"` | `` | Hotspot label. |
| `off true \| false` | `false` | End this hotspot instead of providing rect. |

<a id="file-timeline-track-light"></a>
### `file:timeline.track.light`

| Field | Default | Meaning |
|---|---|---|
| `key <seconds >= 0> { ... }` | `` | Repeated keys, sorted stably by time; nested fields depend on track kind. |

<a id="file-timeline-track-light-key"></a>
### `file:timeline.track.light.key`

| Field | Default | Meaning |
|---|---|---|
| `ease <word or quoted easing expression>` | `linear` | Shared og-tween easing parser, including cubicBezier, steps and points. |

<a id="file-timeline-track-marker"></a>
### `file:timeline.track.marker`

| Field | Default | Meaning |
|---|---|---|
| `key <seconds >= 0> { ... }` | `` | Repeated keys, sorted stably by time; nested fields depend on track kind. |

<a id="file-timeline-track-marker-key"></a>
### `file:timeline.track.marker.key`

| Field | Default | Meaning |
|---|---|---|
| `name "<1..63 byte name>"` | `required` | Named timeline marker. |

<a id="file-timeline-track-material"></a>
### `file:timeline.track.material`

| Field | Default | Meaning |
|---|---|---|
| `key <seconds >= 0> { ... }` | `` | Repeated keys, sorted stably by time; nested fields depend on track kind. |
| `index <integer >= 0>` | `0` | Target material index. |

<a id="file-timeline-track-material-key"></a>
### `file:timeline.track.material.key`

| Field | Default | Meaning |
|---|---|---|
| `ease <word or quoted easing expression>` | `linear` | Shared og-tween easing parser, including cubicBezier, steps and points. |

<a id="file-timeline-track-music"></a>
### `file:timeline.track.music`

| Field | Default | Meaning |
|---|---|---|
| `key <seconds >= 0> { ... }` | `` | Repeated keys, sorted stably by time; nested fields depend on track kind. |

<a id="file-timeline-track-music-key"></a>
### `file:timeline.track.music.key`

| Field | Default | Meaning |
|---|---|---|
| `loop true \| false` | `context-dependent` | Animation/music default true; nested timeline false. |
| `sound "<asset>"` | `required` | Cooked sound asset. |
| `volume <number >= 0>` | `1` | Linear gain. |
| `bus master \| music \| sfx \| ui` | `context-dependent` | Audio defaults sfx; music defaults music. |
| `fadeIn <seconds >= 0>` | `0` | Fade in duration. |
| `fadeOut <seconds >= 0>` | `0` | Fade out duration. |
| `end <seconds > key time>` | `context-dependent` | Absolute end; caption can derive it from following cues. |

<a id="file-timeline-track-post"></a>
### `file:timeline.track.post`

| Field | Default | Meaning |
|---|---|---|
| `key <seconds >= 0> { ... }` | `` | Repeated keys, sorted stably by time; nested fields depend on track kind. |

<a id="file-timeline-track-post-key"></a>
### `file:timeline.track.post.key`

| Field | Default | Meaning |
|---|---|---|
| `ease <word or quoted easing expression>` | `linear` | Shared og-tween easing parser, including cubicBezier, steps and points. |

<a id="file-timeline-track-property"></a>
### `file:timeline.track.property`

| Field | Default | Meaning |
|---|---|---|
| `key <seconds >= 0> { ... }` | `` | Repeated keys, sorted stably by time; nested fields depend on track kind. |
| `field "<record.field>"` | `required` | Reflected target field, declared before keys. |

<a id="file-timeline-track-property-key"></a>
### `file:timeline.track.property.key`

| Field | Default | Meaning |
|---|---|---|
| `ease <word or quoted easing expression>` | `linear` | Shared og-tween easing parser, including cubicBezier, steps and points. |
| `value <1..4 numbers or word>` | `required` | Value of the declared reflected field. |

<a id="file-timeline-track-scene"></a>
### `file:timeline.track.scene`

| Field | Default | Meaning |
|---|---|---|
| `key <seconds >= 0> { ... }` | `` | Repeated keys, sorted stably by time; nested fields depend on track kind. |

<a id="file-timeline-track-scene-key"></a>
### `file:timeline.track.scene.key`

| Field | Default | Meaning |
|---|---|---|
| `scene "<asset>" \| none` | `required` | Place a scene or clear it. |

<a id="file-timeline-track-shot"></a>
### `file:timeline.track.shot`

| Field | Default | Meaning |
|---|---|---|
| `key <seconds >= 0> { ... }` | `` | Repeated keys, sorted stably by time; nested fields depend on track kind. |

<a id="file-timeline-track-shot-key"></a>
### `file:timeline.track.shot.key`

| Field | Default | Meaning |
|---|---|---|
| `ease <word or quoted easing expression>` | `linear` | Shared og-tween easing parser, including cubicBezier, steps and points. |
| `camera "<entity>"` | `` | Virtual camera, exclusive in intent with none. |
| `none ` | `` | Release virtual camera control. |
| `blend <seconds >= 0>` | `camera setting` | Zero cuts immediately. |

<a id="file-timeline-track-timeline"></a>
### `file:timeline.track.timeline`

| Field | Default | Meaning |
|---|---|---|
| `key <seconds >= 0> { ... }` | `` | Repeated keys, sorted stably by time; nested fields depend on track kind. |

<a id="file-timeline-track-timeline-key"></a>
### `file:timeline.track.timeline.key`

| Field | Default | Meaning |
|---|---|---|
| `speed <number>` | `1` | Animation speed multiplier; nested timeline >0; video 0..4. |
| `loop true \| false` | `context-dependent` | Animation/music default true; nested timeline false. |
| `play "<timeline asset>"` | `required` | Nested timeline; cook checks cycles and references. |

<a id="file-timeline-track-transform"></a>
### `file:timeline.track.transform`

| Field | Default | Meaning |
|---|---|---|
| `key <seconds >= 0> { ... }` | `` | Repeated keys, sorted stably by time; nested fields depend on track kind. |

<a id="file-timeline-track-transform-key"></a>
### `file:timeline.track.transform.key`

| Field | Default | Meaning |
|---|---|---|
| `position <x> <y> <z>` | `context-dependent` | Local metres; camera/transform keep unspecified values, vfx defaults to origin. |
| `ease <word or quoted easing expression>` | `linear` | Shared og-tween easing parser, including cubicBezier, steps and points. |
| `rotation <x> <y> <z>` | `` | Euler XYZ degrees. |
| `quaternion <x> <y> <z> <w>` | `` | Nonzero quaternion, normalized. |
| `scale <x> <y> <z>` | `` | Scale factors. |

<a id="file-timeline-track-transition"></a>
### `file:timeline.track.transition`

| Field | Default | Meaning |
|---|---|---|
| `key <seconds >= 0> { ... }` | `` | Repeated keys, sorted stably by time; nested fields depend on track kind. |

<a id="file-timeline-track-transition-key"></a>
### `file:timeline.track.transition.key`

| Field | Default | Meaning |
|---|---|---|
| `kind <transition kind>` | `fade` | Shared TransitionKind names. |
| `mode through \| in \| out` | `through` | Transition mode. |
| `duration <seconds >= 0>` | `1` | Transition duration. |
| `color <r> <g> <b>` | `0 0 0` | Linear transition colour. |
| `from left \| right \| top \| bottom` | `left` | Transition origin. |

<a id="file-timeline-track-vfx"></a>
### `file:timeline.track.vfx`

| Field | Default | Meaning |
|---|---|---|
| `key <seconds >= 0> { ... }` | `` | Repeated keys, sorted stably by time; nested fields depend on track kind. |

<a id="file-timeline-track-vfx-key"></a>
### `file:timeline.track.vfx.key`

| Field | Default | Meaning |
|---|---|---|
| `position <x> <y> <z>` | `context-dependent` | Local metres; camera/transform keep unspecified values, vfx defaults to origin. |
| `effect "<asset>"` | `required` | Effect asset. |
| `at "<entity>"` | `` | Effect anchor entity. |
| `scale <number > 0>` | `1` | Uniform effect size. |
| `seed <u32>` | `0` | Explicit effect seed. |

<a id="file-timeline-track-video"></a>
### `file:timeline.track.video`

| Field | Default | Meaning |
|---|---|---|
| `key <seconds >= 0> { ... }` | `` | Repeated keys, sorted stably by time; nested fields depend on track kind. |
| `layer over \| under` | `over` | Only for the timeline's own player, not entity players. |

<a id="file-timeline-track-video-key"></a>
### `file:timeline.track.video.key`

| Field | Default | Meaning |
|---|---|---|
| `speed <number>` | `1` | Animation speed multiplier; nested timeline >0; video 0..4. |
| `video "<asset>"` | `required` | Video asset. |
| `from <seconds >= 0>` | `0` | Source offset. |

<a id="file-timeline-track-visibility"></a>
### `file:timeline.track.visibility`

| Field | Default | Meaning |
|---|---|---|
| `key <seconds >= 0> { ... }` | `` | Repeated keys, sorted stably by time; nested fields depend on track kind. |

<a id="file-timeline-track-visibility-key"></a>
### `file:timeline.track.visibility.key`

| Field | Default | Meaning |
|---|---|---|
| `ease <word or quoted easing expression>` | `linear` | Shared og-tween easing parser, including cubicBezier, steps and points. |
| `visible true \| false` | `` | Entity visibility. |

Light/post/material key names are writable reflected fields of their targets, with 1..4 numeric lanes or an enum word; query their record with og api show. Blendshape accepts w0..w7. Values not set by a key retain their track state.

Transition kinds: cut, fade, wipe, dissolve, iris, slide.


<a id="file-ui-material"></a>
### `file:ui-material`

An authored `ui-material` asset, cooked by `ui-material`. Input: `.ogt` under assets/.

Field coverage: complete shared shader-family parser table, filtered by document type. Cook validates the same WGSL contract on every supported backend. See render or ui-materials for WGSL entry structures and callable functions.

| Field | Default | Meaning |
|---|---|---|
| `source "<WGSL file>"` | `` | One source file; exclusive with code. Cook tracks its bytes and inlines it. |
| `code "<WGSL text>"` | `` | Inline WGSL; exclusive with source. One input is required. |
| `param <name> <float \| vec2 \| vec3 \| vec4 \| color> [<defaults> ...]` | `zero components` | Repeat shader parameters; defaults are absent or exactly the type's finite component count. Color is linear RGB. Backend checks validate names, count and WGSL contract. |

<a id="file-vector"></a>
### `file:vector`

Author `content/**/*.ts` with `export default file["vector"]({...})` from `@origingame/engine/author`. Project/module types are generated by `og api gen`; canonical output IDs are `<module>/<content-relative-stem>`. This domain does not accept OGT sources.

```ts
/** Exactly one SVG source. SVG syntax/element support belongs to the vector
domain validator. No embedded OGT or untyped configuration is accepted. */
export type Input = (((({
/** Explicit canonical SVG path relative to the owning module
(e.g. assets/ui/emblem.svg), without sibling/assets fallback. */
readonly "source": (string);
/** Positive flattening error in SVG view-box units; omitted uses the
longer view-box side divided by 2000. */
readonly "tolerance"?: ((number) | (null));
})) & { readonly "svg"?: never; }) | ((({
readonly "svg": (string);
/** Positive flattening error in SVG view-box units. */
readonly "tolerance"?: ((number) | (null));
})) & { readonly "source"?: never; }));

```

Output: vector triangles loaded with og.load.vector. Exactly one source (an explicit canonical module-relative SVG path, such as assets/ui/emblem.svg) or inline svg string is required. The SVG subset and curve flattening remain the native tessellator's; tolerance is positive view-box units, omitted uses the longer view-box side / 2000. External SVG bytes participate in cook stamps; no sibling/assets fallback is used.

<a id="file-vfx"></a>
### `file:vfx`

An authored `vfx` asset, cooked by `vfx`. Input: `.ogt` under assets/.

Field coverage: parser line tables; see `effect-files` for nested blocks and constraints.

Alternatively, one exclusive generate block returns EffectSource {text}: the body declarations without type/version/header. The ordinary parser and asset links validate the result.


<a id="file-vfx-generate"></a>
### `file:vfx.generate`

| Field | Default | Meaning |
|---|---|---|
| `script "<file.ts>"` | `required` | Synchronous default export; static relative imports and explicit inputs are tracked. |
| `seed <u32>` | `0` | Explicit seed passed to the generator. |
| `params { ... }` | `{}` | Named scalar, array or nested object parameters. |
| `input "<name>" "<path>"` | `` | Repeated input bytes, relative to the authored asset, available as ctx.inputs[name]. |
| Field | Default | Meaning |
|---|---|---|
| `duration <seconds>` | `` | How long the effect lasts; its elements end within it. Required. |
| `loop true \| false` | `false` | Start over at the end. |
| `impact <seconds>` | `` | The same as `event <seconds> "impact"`; at most one. |
| `event <seconds> "<name>"` | `` | An og.VfxEvent with this name at this time. |
| `param <name> <default>` | `` | A number the game sets per instance (og.Vfx.params, in the order declared; at most 4): elements read it with `when` and `amount`. |
| `emitter "<id>" { ... }` | `` | An emitter asset (§1) playing its own cycle; `until` stops its emission and its particles live out. |
| `mesh sphere \| quad \| ring \| box \| cylinder \| "<model id>" { ... }` | `` | A mesh drawn with an effect material (§2); a ring builds its own shape at load. |
| `trail { ... }` | `` | A trail behind the element's point. |
| `beam { ... }` | `` | A beam from the element's point to `to`. |
| `light { ... }` | `` | A point light. |
| `decal { ... }` | `` | A decal projected down the element's -y onto every lit surface in its box: its `texture` and `normal` map, or a square of its colour without them. |
| `glow { ... }` | `` | A soft bright glow facing the eye (version 1 named it `flash`). |
| `shake { ... }` | `` | og.camera.shake, at once. |
| `sound "<id>" { ... }` | `` | A sound asset, played at once by a child entity with og.AudioSource, cut when the effect despawns. |
| `flipbook "<id>" { ... }` | `` | A camera-facing og.Sprite with og.Flipbook playing a sheet's first clip once (a `type vfx2d` pixel effect or any og.load.flipbook sheet); its og.FlipbookEvent events, the impact among them, reach the game as this effect's og.VfxEvent. |

<a id="file-vfx-beam"></a>
### `file:vfx.beam`

| Field | Default | Meaning |
|---|---|---|
| `at <seconds>` | `0` | When it starts, from the effect's start. |
| `life <seconds>` | `` | How long it lasts (an element that lasts takes `life` or `until`; an emitter runs its cycle and a flipbook its clip without either). |
| `until <seconds>` | `` | When it ends, instead of `life`. |
| `offset <x> <y> <z>` | `0 0 0` | Where it starts, in the effect's space. |
| `rotate <x> <y> <z>` | `0 0 0` | Degrees, in the effect's space. |
| `move <x> <y> <z>` | `0 0 0` | Metres per second it moves from its offset: an emitter spawns along the path, a mesh, trail or light goes with it. |
| `accelerate <x> <y> <z>` | `0 0 0` | Metres per second squared. |
| `when <param>` | `` | Plays only in an instance whose value of the param is above 0 (a critical hit's extra burst, a decal only on the ground). |
| `amount <param>` | `` | Multiplies how much of it there is by the param (at least 0): an emitter's share of particles born (at most 1), a mesh's, glow's, decal's or flipbook's size, a trail's or beam's width, a light's intensity, a shake's trauma, a sound's volume. |
| `stretch <param>` | `` | Multiplies its length alone by the param (at least 0), along its own y after `rotate`: a jet plume that lengthens with thrust without growing wider. A beam's `to` is multiplied instead: a laser as long as the distance to its mark, no wider. Applied with `amount`, which scales it whole (a beam's width). |
| `effect "<id>"` | `` | Its effect material (§2); a mesh needs one. |
| `color <r> <g> <b> [<a>]` | `1 1 1 1` | A linear colour; above 1 blooms (a light's alpha is ignored). |
| `width <metres>` | `0.2` | Metres across. |
| `to <x> <y> <z>` | `0 1 0` | Its far end, from the element's point. |
| `segments <n>` | `1` | 1 to 256: more let `jitter` bend it. |
| `jitter <metres>` | `0` | Sideways jitter, new every tick (lightning). |

<a id="file-vfx-decal"></a>
### `file:vfx.decal`

| Field | Default | Meaning |
|---|---|---|
| `at <seconds>` | `0` | When it starts, from the effect's start. |
| `life <seconds>` | `` | How long it lasts (an element that lasts takes `life` or `until`; an emitter runs its cycle and a flipbook its clip without either). |
| `until <seconds>` | `` | When it ends, instead of `life`. |
| `offset <x> <y> <z>` | `0 0 0` | Where it starts, in the effect's space. |
| `rotate <x> <y> <z>` | `0 0 0` | Degrees, in the effect's space. |
| `move <x> <y> <z>` | `0 0 0` | Metres per second it moves from its offset: an emitter spawns along the path, a mesh, trail or light goes with it. |
| `accelerate <x> <y> <z>` | `0 0 0` | Metres per second squared. |
| `when <param>` | `` | Plays only in an instance whose value of the param is above 0 (a critical hit's extra burst, a decal only on the ground). |
| `amount <param>` | `` | Multiplies how much of it there is by the param (at least 0): an emitter's share of particles born (at most 1), a mesh's, glow's, decal's or flipbook's size, a trail's or beam's width, a light's intensity, a shake's trauma, a sound's volume. |
| `color <r> <g> <b> [<a>]` | `1 1 1 1` | A linear colour; above 1 blooms (a light's alpha is ignored). |
| `texture "<id>"` | `` | Its image (a texture's id): the colour times `color`, the alpha its coverage. Without one it is a square of `color`. |
| `normal "<id>"` | `` | A tangent-space normal map (a texture's id; +x along u, +y up the image, as glTF) that bends the surface under it: a crater's rim, a crack, a bullet hole. |
| `normalScale <strength>` | `1` | How strongly `normal` bends the surface, 0 to 4. |
| `uv <u0> <v0> <u1> <v1>` | `0 0 1 1` | Its region of `texture` and `normal`, v down: one mark of a sheet of several. |
| `emissive <r> <g> <b>` | `0 0 0` | Linear light it adds where it covers, times the image (embers in a scorch, a glowing rune); above 1 blooms. It fades with the decal. |
| `roughness <0..1>` | `` | The surface's roughness under it (soot is rough, a puddle smooth); without the line the surface keeps its own. |
| `metallic <0..1>` | `` | The surface's metalness under it; without the line the surface keeps its own. |
| `wet true \| false` | `false` | Darkens the surface by its colour and flattens it toward the decal's normal instead of covering it: a puddle, a splash mark, blood soaking in. Set `roughness` low for standing water. |
| `angleFade <degrees>` | `75` | Surfaces turned further than this from the projection are not covered (the last 15 degrees fade), 0 to 90. |
| `turn <degrees> \| <min> <max>` | `0` | Turns it about the projection; a range picks one angle per instance from its seed, so repeated marks do not look stamped. |
| `size <s> \| <x> <y> <z>` | `1 1 1` | x, the depth it reaches down its -y, and z. |
| `fade <seconds>` | `0` | Fades out over its last this many seconds. |

<a id="file-vfx-emitter"></a>
### `file:vfx.emitter`

| Field | Default | Meaning |
|---|---|---|
| `at <seconds>` | `0` | When it starts, from the effect's start. |
| `life <seconds>` | `` | How long it lasts (an element that lasts takes `life` or `until`; an emitter runs its cycle and a flipbook its clip without either). |
| `until <seconds>` | `` | When it ends, instead of `life`. |
| `offset <x> <y> <z>` | `0 0 0` | Where it starts, in the effect's space. |
| `rotate <x> <y> <z>` | `0 0 0` | Degrees, in the effect's space. |
| `move <x> <y> <z>` | `0 0 0` | Metres per second it moves from its offset: an emitter spawns along the path, a mesh, trail or light goes with it. |
| `accelerate <x> <y> <z>` | `0 0 0` | Metres per second squared. |
| `when <param>` | `` | Plays only in an instance whose value of the param is above 0 (a critical hit's extra burst, a decal only on the ground). |
| `amount <param>` | `` | Multiplies how much of it there is by the param (at least 0): an emitter's share of particles born (at most 1), a mesh's, glow's, decal's or flipbook's size, a trail's or beam's width, a light's intensity, a shake's trauma, a sound's volume. |
| `seed <u32>` | `0` | Added to the effect's seed. |

<a id="file-vfx-flipbook"></a>
### `file:vfx.flipbook`

| Field | Default | Meaning |
|---|---|---|
| `at <seconds>` | `0` | When it starts, from the effect's start. |
| `life <seconds>` | `` | How long it lasts (an element that lasts takes `life` or `until`; an emitter runs its cycle and a flipbook its clip without either). |
| `until <seconds>` | `` | When it ends, instead of `life`. |
| `offset <x> <y> <z>` | `0 0 0` | Where it starts, in the effect's space. |
| `rotate <x> <y> <z>` | `0 0 0` | Degrees, in the effect's space. |
| `move <x> <y> <z>` | `0 0 0` | Metres per second it moves from its offset: an emitter spawns along the path, a mesh, trail or light goes with it. |
| `accelerate <x> <y> <z>` | `0 0 0` | Metres per second squared. |
| `when <param>` | `` | Plays only in an instance whose value of the param is above 0 (a critical hit's extra burst, a decal only on the ground). |
| `amount <param>` | `` | Multiplies how much of it there is by the param (at least 0): an emitter's share of particles born (at most 1), a mesh's, glow's, decal's or flipbook's size, a trail's or beam's width, a light's intensity, a shake's trauma, a sound's volume. |
| `blend additive \| alpha \| subtractive \| multiply` | `` | A mesh adds by default, a flipbook blends alpha; a flipbook takes additive or alpha only. |
| `color <r> <g> <b> [<a>]` | `1 1 1 1` | A linear colour; above 1 blooms (a light's alpha is ignored). |
| `size <metres>` | `1` | Its height; the width follows the frame's shape. |

<a id="file-vfx-glow"></a>
### `file:vfx.glow`

| Field | Default | Meaning |
|---|---|---|
| `at <seconds>` | `0` | When it starts, from the effect's start. |
| `life <seconds>` | `` | How long it lasts (an element that lasts takes `life` or `until`; an emitter runs its cycle and a flipbook its clip without either). |
| `until <seconds>` | `` | When it ends, instead of `life`. |
| `offset <x> <y> <z>` | `0 0 0` | Where it starts, in the effect's space. |
| `rotate <x> <y> <z>` | `0 0 0` | Degrees, in the effect's space. |
| `move <x> <y> <z>` | `0 0 0` | Metres per second it moves from its offset: an emitter spawns along the path, a mesh, trail or light goes with it. |
| `accelerate <x> <y> <z>` | `0 0 0` | Metres per second squared. |
| `when <param>` | `` | Plays only in an instance whose value of the param is above 0 (a critical hit's extra burst, a decal only on the ground). |
| `amount <param>` | `` | Multiplies how much of it there is by the param (at least 0): an emitter's share of particles born (at most 1), a mesh's, glow's, decal's or flipbook's size, a trail's or beam's width, a light's intensity, a shake's trauma, a sound's volume. |
| `stretch <param>` | `` | Multiplies its length alone by the param (at least 0), along its own y after `rotate`: a jet plume that lengthens with thrust without growing wider. A beam's `to` is multiplied instead: a laser as long as the distance to its mark, no wider. Applied with `amount`, which scales it whole (a beam's width). |
| `color <r> <g> <b> [<a>]` | `1 1 1 1` | A linear colour; above 1 blooms (a light's alpha is ignored). |
| `size <value>` | `1` | Metres across over its life. |

<a id="file-vfx-light"></a>
### `file:vfx.light`

| Field | Default | Meaning |
|---|---|---|
| `at <seconds>` | `0` | When it starts, from the effect's start. |
| `life <seconds>` | `` | How long it lasts (an element that lasts takes `life` or `until`; an emitter runs its cycle and a flipbook its clip without either). |
| `until <seconds>` | `` | When it ends, instead of `life`. |
| `offset <x> <y> <z>` | `0 0 0` | Where it starts, in the effect's space. |
| `rotate <x> <y> <z>` | `0 0 0` | Degrees, in the effect's space. |
| `move <x> <y> <z>` | `0 0 0` | Metres per second it moves from its offset: an emitter spawns along the path, a mesh, trail or light goes with it. |
| `accelerate <x> <y> <z>` | `0 0 0` | Metres per second squared. |
| `when <param>` | `` | Plays only in an instance whose value of the param is above 0 (a critical hit's extra burst, a decal only on the ground). |
| `amount <param>` | `` | Multiplies how much of it there is by the param (at least 0): an emitter's share of particles born (at most 1), a mesh's, glow's, decal's or flipbook's size, a trail's or beam's width, a light's intensity, a shake's trauma, a sound's volume. |
| `color <r> <g> <b> [<a>]` | `1 1 1 1` | A linear colour; above 1 blooms (a light's alpha is ignored). |
| `intensity <value>` | `1` | Candela over its life. |
| `distance <metres>` | `5` | Where it reaches zero. |

<a id="file-vfx-mesh"></a>
### `file:vfx.mesh`

| Field | Default | Meaning |
|---|---|---|
| `at <seconds>` | `0` | When it starts, from the effect's start. |
| `life <seconds>` | `` | How long it lasts (an element that lasts takes `life` or `until`; an emitter runs its cycle and a flipbook its clip without either). |
| `until <seconds>` | `` | When it ends, instead of `life`. |
| `offset <x> <y> <z>` | `0 0 0` | Where it starts, in the effect's space. |
| `rotate <x> <y> <z>` | `0 0 0` | Degrees, in the effect's space. |
| `move <x> <y> <z>` | `0 0 0` | Metres per second it moves from its offset: an emitter spawns along the path, a mesh, trail or light goes with it. |
| `accelerate <x> <y> <z>` | `0 0 0` | Metres per second squared. |
| `when <param>` | `` | Plays only in an instance whose value of the param is above 0 (a critical hit's extra burst, a decal only on the ground). |
| `amount <param>` | `` | Multiplies how much of it there is by the param (at least 0): an emitter's share of particles born (at most 1), a mesh's, glow's, decal's or flipbook's size, a trail's or beam's width, a light's intensity, a shake's trauma, a sound's volume. |
| `stretch <param>` | `` | Multiplies its length alone by the param (at least 0), along its own y after `rotate`: a jet plume that lengthens with thrust without growing wider. A beam's `to` is multiplied instead: a laser as long as the distance to its mark, no wider. Applied with `amount`, which scales it whole (a beam's width). |
| `effect "<id>"` | `` | Its effect material (§2); a mesh needs one. |
| `scale <value>` | `1` | Uniform scale over its life. |
| `spin <x> <y> <z>` | `0 0 0` | Degrees per second. |
| `face camera` | `` | Turned to face the eye as a billboard is (a glow ring seen from any side): its spin and scale kept, the effect's and its own rotation left out. Without it a mesh lies as it is placed. |
| `blend additive \| alpha \| subtractive \| multiply` | `` | A mesh adds by default, a flipbook blends alpha; a flipbook takes additive or alpha only. |
| `color <r> <g> <b> [<a>]` | `1 1 1 1` | A linear colour; above 1 blooms (a light's alpha is ignored). |

<a id="file-vfx-ring"></a>
### `file:vfx.ring`

| Field | Default | Meaning |
|---|---|---|
| `at <seconds>` | `0` | When it starts, from the effect's start. |
| `life <seconds>` | `` | How long it lasts (an element that lasts takes `life` or `until`; an emitter runs its cycle and a flipbook its clip without either). |
| `until <seconds>` | `` | When it ends, instead of `life`. |
| `offset <x> <y> <z>` | `0 0 0` | Where it starts, in the effect's space. |
| `rotate <x> <y> <z>` | `0 0 0` | Degrees, in the effect's space. |
| `move <x> <y> <z>` | `0 0 0` | Metres per second it moves from its offset: an emitter spawns along the path, a mesh, trail or light goes with it. |
| `accelerate <x> <y> <z>` | `0 0 0` | Metres per second squared. |
| `when <param>` | `` | Plays only in an instance whose value of the param is above 0 (a critical hit's extra burst, a decal only on the ground). |
| `amount <param>` | `` | Multiplies how much of it there is by the param (at least 0): an emitter's share of particles born (at most 1), a mesh's, glow's, decal's or flipbook's size, a trail's or beam's width, a light's intensity, a shake's trauma, a sound's volume. |
| `inner <radius>` | `` | The inner radius (the outer is 1). |
| `band <0..1>` | `` | Where its middle row lies between the edges. |
| `height <outer> <inner>` | `` | Its edges raised along its axis: a cone, a tube. |
| `arc <start> <length>` | `` | Degrees from +x. |
| `segments <n>` | `` | 3 to 256 around it. |
| `colors <outer rgba> <band rgba> <inner rgba>` | `` | Its vertex colours; u runs along the arc and v from the outer edge (0) to the inner (1). |

<a id="file-vfx-shake"></a>
### `file:vfx.shake`

| Field | Default | Meaning |
|---|---|---|
| `at <seconds>` | `0` | When it starts, from the effect's start. |
| `life <seconds>` | `` | How long it lasts (an element that lasts takes `life` or `until`; an emitter runs its cycle and a flipbook its clip without either). |
| `until <seconds>` | `` | When it ends, instead of `life`. |
| `offset <x> <y> <z>` | `0 0 0` | Where it starts, in the effect's space. |
| `rotate <x> <y> <z>` | `0 0 0` | Degrees, in the effect's space. |
| `move <x> <y> <z>` | `0 0 0` | Metres per second it moves from its offset: an emitter spawns along the path, a mesh, trail or light goes with it. |
| `accelerate <x> <y> <z>` | `0 0 0` | Metres per second squared. |
| `when <param>` | `` | Plays only in an instance whose value of the param is above 0 (a critical hit's extra burst, a decal only on the ground). |
| `amount <param>` | `` | Multiplies how much of it there is by the param (at least 0): an emitter's share of particles born (at most 1), a mesh's, glow's, decal's or flipbook's size, a trail's or beam's width, a light's intensity, a shake's trauma, a sound's volume. |
| `trauma <0..1>` | `0.2` | og.camera.shake's trauma. |

<a id="file-vfx-sound"></a>
### `file:vfx.sound`

| Field | Default | Meaning |
|---|---|---|
| `at <seconds>` | `0` | When it starts, from the effect's start. |
| `life <seconds>` | `` | How long it lasts (an element that lasts takes `life` or `until`; an emitter runs its cycle and a flipbook its clip without either). |
| `until <seconds>` | `` | When it ends, instead of `life`. |
| `offset <x> <y> <z>` | `0 0 0` | Where it starts, in the effect's space. |
| `rotate <x> <y> <z>` | `0 0 0` | Degrees, in the effect's space. |
| `move <x> <y> <z>` | `0 0 0` | Metres per second it moves from its offset: an emitter spawns along the path, a mesh, trail or light goes with it. |
| `accelerate <x> <y> <z>` | `0 0 0` | Metres per second squared. |
| `when <param>` | `` | Plays only in an instance whose value of the param is above 0 (a critical hit's extra burst, a decal only on the ground). |
| `amount <param>` | `` | Multiplies how much of it there is by the param (at least 0): an emitter's share of particles born (at most 1), a mesh's, glow's, decal's or flipbook's size, a trail's or beam's width, a light's intensity, a shake's trauma, a sound's volume. |
| `volume <gain>` | `1` | Its volume. |
| `playbackRate <rate> \| <min> <max>` | `1` | A rate picked from the range by the effect's seed. |

<a id="file-vfx-trail"></a>
### `file:vfx.trail`

| Field | Default | Meaning |
|---|---|---|
| `at <seconds>` | `0` | When it starts, from the effect's start. |
| `life <seconds>` | `` | How long it lasts (an element that lasts takes `life` or `until`; an emitter runs its cycle and a flipbook its clip without either). |
| `until <seconds>` | `` | When it ends, instead of `life`. |
| `offset <x> <y> <z>` | `0 0 0` | Where it starts, in the effect's space. |
| `rotate <x> <y> <z>` | `0 0 0` | Degrees, in the effect's space. |
| `move <x> <y> <z>` | `0 0 0` | Metres per second it moves from its offset: an emitter spawns along the path, a mesh, trail or light goes with it. |
| `accelerate <x> <y> <z>` | `0 0 0` | Metres per second squared. |
| `when <param>` | `` | Plays only in an instance whose value of the param is above 0 (a critical hit's extra burst, a decal only on the ground). |
| `amount <param>` | `` | Multiplies how much of it there is by the param (at least 0): an emitter's share of particles born (at most 1), a mesh's, glow's, decal's or flipbook's size, a trail's or beam's width, a light's intensity, a shake's trauma, a sound's volume. |
| `effect "<id>"` | `` | Its effect material (§2); a mesh needs one. |
| `color <r> <g> <b> [<a>]` | `1 1 1 1` | A linear colour; above 1 blooms (a light's alpha is ignored). |
| `width <head> [<tail>]` | `0.2 0` | Metres across at its head and its tail. |
| `lifetime <seconds>` | `0.4` | How long a point stays in the trail. |
| `orbit <radius> <turns per second>` | `0 0` | Circles the element's y. |
| `rise <metres per second>` | `0` | Climbs the element's y. |

<a id="file-vfx2d"></a>
### `file:vfx2d`

An authored `vfx2d` asset, cooked by `flipbook`. Input: `.ogt` under assets/.

OGT header followed by inline JSON or an external recipe, not an ordinary nested OGT grammar. Unknown JSON notes are ignored. JSON fields below share the recipe reader's recognised field registry. See pixel-vfx for layer-specific defaults and enum choices.

| Field | Default | Meaning |
|---|---|---|
| `duration <seconds > 0>` | `frames / fps` | Override clip duration. |
| `loop true \| false` | `recipe` | Clip playback mode. |
| `impact <seconds >= 0>` | `recipe or coverage jump` | One impact marker; must lie within duration. |
| `event <seconds >= 0> "<name>"` | `` | Repeated markers, at most 16 including impact; names 1..63 bytes. |
| `recipe "<path.json>"` | `inline JSON` | Relative file or asset id, exclusive with inline JSON. |

<a id="file-vfx2d-recipe"></a>
### `file:vfx2d.recipe`

| Field | Default | Meaning |
|---|---|---|
| `name string` | `Untitled effect` | Display name. |
| `w integer, 8..512` | `64` | Canvas pixels. |
| `h integer, 8..512` | `64` | Canvas pixels. |
| `frames integer, 1..120` | `16` | Frame count. |
| `fps integer, 1..60` | `15` | Frames per second. |
| `seed integer, 0..2147483647` | `1` | Deterministic seed. |
| `loop boolean` | `false` | Periodic animation. |
| `ramps {name: ramp}` | `Forge ramps` | Ramp is a named ramp, hue:<degrees>, or hex colour array. |
| `hue number, degrees` | `0` | Ramp hue rotation. |
| `sat number` | `1` | Saturation multiplier. |
| `outline none \| outside \| outside8 \| inside` | `none` | Pixel outline mode. |
| `outlineColor hex colour or null` | `null` | Fixed outline colour. |
| `dither number, 0..1` | `0` | Ordered dither. |
| `palette named palette \| hex colour array \| null` | `null` | Quantise to an explicit palette. |
| `cleanup boolean` | `true` | Clean isolated pixels and holes. |
| `pivot [x,y], 0..1` | `[0.5,0.5]` | Top-left canvas fractions. |
| `impact number, 0..1` | `coverage jump` | Normalised impact time. |
| `layers [layer, ...]` | `required` | Layers paint in authored order. |

<a id="file-vfx2d-layer-burst"></a>
### `file:vfx2d.layer.burst`

| Field | Default | Meaning |
|---|---|---|
| `type layer type name` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `ramp ramp name \| $name \| hex colour array` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `t0 number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `t1 number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `x number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `y number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `angle number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `squash number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `gain number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `cut number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `gamma number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `dither number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `glow number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `glowFalloff number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `blend string; see pixel-vfx for enumerated values` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `hidden boolean` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `seed number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `hue number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `ease string; see pixel-vfx for enumerated values` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `cool number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `drift number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `erode number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `erodeStart number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `evolve number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `freq number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `grow number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `hollow number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `r0 number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `r1 number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `rough number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `wobble number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |

<a id="file-vfx2d-layer-ring"></a>
### `file:vfx2d.layer.ring`

| Field | Default | Meaning |
|---|---|---|
| `type layer type name` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `ramp ramp name \| $name \| hex colour array` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `t0 number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `t1 number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `x number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `y number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `angle number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `squash number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `gain number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `cut number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `gamma number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `dither number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `glow number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `glowFalloff number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `blend string; see pixel-vfx for enumerated values` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `hidden boolean` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `seed number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `hue number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `ease string; see pixel-vfx for enumerated values` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `breakup number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `fade number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `freq number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `inner number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `r0 number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `r1 number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `th number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `thEnd number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |

<a id="file-vfx2d-layer-flame"></a>
### `file:vfx2d.layer.flame`

| Field | Default | Meaning |
|---|---|---|
| `type layer type name` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `ramp ramp name \| $name \| hex colour array` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `t0 number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `t1 number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `x number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `y number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `angle number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `squash number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `gain number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `cut number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `gamma number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `dither number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `glow number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `glowFalloff number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `blend string; see pixel-vfx for enumerated values` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `hidden boolean` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `seed number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `hue number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `ease string; see pixel-vfx for enumerated values` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `flick number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `freq number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `hgt number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `hollow number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `rough number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `speed number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `taper number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `w number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |

<a id="file-vfx2d-layer-beam"></a>
### `file:vfx2d.layer.beam`

| Field | Default | Meaning |
|---|---|---|
| `type layer type name` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `ramp ramp name \| $name \| hex colour array` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `t0 number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `t1 number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `x number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `y number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `angle number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `squash number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `gain number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `cut number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `gamma number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `dither number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `glow number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `glowFalloff number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `blend string; see pixel-vfx for enumerated values` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `hidden boolean` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `seed number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `hue number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `ease string; see pixel-vfx for enumerated values` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `cap number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `freq number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `grow number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `len number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `speed number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `th number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `wob number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |

<a id="file-vfx2d-layer-slash"></a>
### `file:vfx2d.layer.slash`

| Field | Default | Meaning |
|---|---|---|
| `type layer type name` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `ramp ramp name \| $name \| hex colour array` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `t0 number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `t1 number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `x number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `y number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `angle number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `squash number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `gain number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `cut number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `gamma number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `dither number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `glow number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `glowFalloff number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `blend string; see pixel-vfx for enumerated values` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `hidden boolean` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `seed number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `hue number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `ease string; see pixel-vfx for enumerated values` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `a0 number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `a1 number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `erode number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `r number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `sweep number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `th number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `trail number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |

<a id="file-vfx2d-layer-orb"></a>
### `file:vfx2d.layer.orb`

| Field | Default | Meaning |
|---|---|---|
| `type layer type name` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `ramp ramp name \| $name \| hex colour array` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `t0 number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `t1 number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `x number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `y number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `angle number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `squash number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `gain number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `cut number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `gamma number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `dither number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `glow number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `glowFalloff number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `blend string; see pixel-vfx for enumerated values` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `hidden boolean` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `seed number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `hue number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `ease string; see pixel-vfx for enumerated values` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `freq number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `grow number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `pulse number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `pulseRate number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `r number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `rim number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `spin number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |

<a id="file-vfx2d-layer-comet"></a>
### `file:vfx2d.layer.comet`

| Field | Default | Meaning |
|---|---|---|
| `type layer type name` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `ramp ramp name \| $name \| hex colour array` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `t0 number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `t1 number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `x number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `y number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `angle number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `squash number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `gain number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `cut number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `gamma number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `dither number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `glow number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `glowFalloff number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `blend string; see pixel-vfx for enumerated values` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `hidden boolean` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `seed number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `hue number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `ease string; see pixel-vfx for enumerated values` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `freq number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `from [x,y]` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `len number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `r number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `rough number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `speed number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `until number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |

<a id="file-vfx2d-layer-vortex"></a>
### `file:vfx2d.layer.vortex`

| Field | Default | Meaning |
|---|---|---|
| `type layer type name` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `ramp ramp name \| $name \| hex colour array` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `t0 number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `t1 number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `x number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `y number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `angle number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `squash number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `gain number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `cut number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `gamma number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `dither number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `glow number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `glowFalloff number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `blend string; see pixel-vfx for enumerated values` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `hidden boolean` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `seed number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `hue number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `ease string; see pixel-vfx for enumerated values` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `arms number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `core string; see pixel-vfx for enumerated values` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `dir number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `freq number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `r number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `rIn number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `spin number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `twist number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |

<a id="file-vfx2d-layer-pillar"></a>
### `file:vfx2d.layer.pillar`

| Field | Default | Meaning |
|---|---|---|
| `type layer type name` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `ramp ramp name \| $name \| hex colour array` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `t0 number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `t1 number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `x number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `y number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `angle number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `squash number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `gain number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `cut number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `gamma number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `dither number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `glow number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `glowFalloff number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `blend string; see pixel-vfx for enumerated values` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `hidden boolean` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `seed number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `hue number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `ease string; see pixel-vfx for enumerated values` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `freq number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `grow number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `hgt number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `rough number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `speed number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `w number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |

<a id="file-vfx2d-layer-rays"></a>
### `file:vfx2d.layer.rays`

| Field | Default | Meaning |
|---|---|---|
| `type layer type name` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `ramp ramp name \| $name \| hex colour array` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `t0 number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `t1 number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `x number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `y number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `angle number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `squash number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `gain number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `cut number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `gamma number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `dither number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `glow number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `glowFalloff number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `blend string; see pixel-vfx for enumerated values` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `hidden boolean` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `seed number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `hue number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `ease string; see pixel-vfx for enumerated values` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `count number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `inner number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `jitter number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `r number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `spin number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `twinkle number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `width number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |

<a id="file-vfx2d-layer-glyph"></a>
### `file:vfx2d.layer.glyph`

| Field | Default | Meaning |
|---|---|---|
| `type layer type name` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `ramp ramp name \| $name \| hex colour array` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `t0 number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `t1 number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `x number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `y number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `angle number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `squash number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `gain number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `cut number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `gamma number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `dither number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `glow number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `glowFalloff number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `blend string; see pixel-vfx for enumerated values` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `hidden boolean` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `seed number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `hue number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `ease string; see pixel-vfx for enumerated values` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `draw number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `fill number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `r number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `rings [number,...]` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `runes number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `spin number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `star number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `step number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `ticks number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |

<a id="file-vfx2d-layer-puffs"></a>
### `file:vfx2d.layer.puffs`

| Field | Default | Meaning |
|---|---|---|
| `type layer type name` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `ramp ramp name \| $name \| hex colour array` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `t0 number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `t1 number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `x number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `y number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `angle number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `squash number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `gain number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `cut number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `gamma number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `dither number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `glow number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `glowFalloff number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `blend string; see pixel-vfx for enumerated values` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `hidden boolean` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `seed number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `hue number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `ease string; see pixel-vfx for enumerated values` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `cone number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `count number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `decay number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `dir number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `dissolve number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `drift number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `emit number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `flat number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `freq number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `life number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `r0 number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `r1 number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `rise number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `speed number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `spread number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |

<a id="file-vfx2d-layer-particles"></a>
### `file:vfx2d.layer.particles`

| Field | Default | Meaning |
|---|---|---|
| `type layer type name` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `ramp ramp name \| $name \| hex colour array` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `t0 number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `t1 number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `x number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `y number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `angle number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `squash number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `gain number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `cut number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `gamma number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `dither number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `glow number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `glowFalloff number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `blend string; see pixel-vfx for enumerated values` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `hidden boolean` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `seed number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `hue number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `ease string; see pixel-vfx for enumerated values` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `area number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `areaX number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `areaY number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `both boolean` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `count number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `dir number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `drag number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `driftX number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `emit number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `fadeIn number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `fallScale number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `gravity number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `ground number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `heat number \| [min,max]` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `implodeR number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `jitter number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `life number \| [min,max]` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `mode string; see pixel-vfx for enumerated values` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `orbitR number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `shape string; see pixel-vfx for enumerated values` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `size number \| [min,max]` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `spanX number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `spanY number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `speed number \| [min,max]` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `spread number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `swirl number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `turns number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `twinkle number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `wiggle number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `wiggleRate number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |

<a id="file-vfx2d-layer-bolt"></a>
### `file:vfx2d.layer.bolt`

| Field | Default | Meaning |
|---|---|---|
| `type layer type name` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `ramp ramp name \| $name \| hex colour array` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `t0 number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `t1 number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `x number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `y number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `angle number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `squash number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `gain number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `cut number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `gamma number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `dither number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `glow number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `glowFalloff number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `blend string; see pixel-vfx for enumerated values` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `hidden boolean` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `seed number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `hue number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `ease string; see pixel-vfx for enumerated values` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `branch number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `count number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `depth number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `fadeStart number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `flicker number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `hold number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `jag number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `len number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `mode string; see pixel-vfx for enumerated values` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `spread number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `thick number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `x0 number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `x1 number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `y0 number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `y1 number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |

<a id="file-vfx2d-layer-cracks"></a>
### `file:vfx2d.layer.cracks`

| Field | Default | Meaning |
|---|---|---|
| `type layer type name` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `ramp ramp name \| $name \| hex colour array` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `t0 number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `t1 number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `x number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `y number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `angle number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `squash number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `gain number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `cut number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `gamma number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `dither number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `glow number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `glowFalloff number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `blend string; see pixel-vfx for enumerated values` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `hidden boolean` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `seed number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `hue number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `ease string; see pixel-vfx for enumerated values` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `count number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `fadeStart number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `grow number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `len number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |

<a id="file-vfx2d-layer-spikes"></a>
### `file:vfx2d.layer.spikes`

| Field | Default | Meaning |
|---|---|---|
| `type layer type name` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `ramp ramp name \| $name \| hex colour array` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `t0 number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `t1 number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `x number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `y number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `angle number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `squash number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `gain number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `cut number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `gamma number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `dither number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `glow number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `glowFalloff number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `blend string; see pixel-vfx for enumerated values` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `hidden boolean` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `seed number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `hue number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `ease string; see pixel-vfx for enumerated values` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `count number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `fan number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `grow number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `hgt number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `hold number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `lean number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `spread number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `width number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |

<a id="file-vfx2d-layer-star"></a>
### `file:vfx2d.layer.star`

| Field | Default | Meaning |
|---|---|---|
| `type layer type name` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `ramp ramp name \| $name \| hex colour array` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `t0 number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `t1 number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `x number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `y number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `angle number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `squash number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `gain number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `cut number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `gamma number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `dither number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `glow number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `glowFalloff number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `blend string; see pixel-vfx for enumerated values` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `hidden boolean` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `seed number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `hue number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `ease string; see pixel-vfx for enumerated values` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `hollow number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `inner number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `jitter number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `points number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `pop number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `r number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `sharp number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |

<a id="file-vfx2d-layer-streaks"></a>
### `file:vfx2d.layer.streaks`

| Field | Default | Meaning |
|---|---|---|
| `type layer type name` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `ramp ramp name \| $name \| hex colour array` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `t0 number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `t1 number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `x number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `y number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `angle number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `squash number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `gain number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `cut number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `gamma number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `dither number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `glow number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `glowFalloff number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `blend string; see pixel-vfx for enumerated values` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `hidden boolean` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `seed number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `hue number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `ease string; see pixel-vfx for enumerated values` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `count number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `curve number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `dir number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `gap number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `len number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `mode string; see pixel-vfx for enumerated values` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `r number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `spanY number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `speed number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `stagger number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `th number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `turns number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |

<a id="file-vfx2d-layer-bubble"></a>
### `file:vfx2d.layer.bubble`

| Field | Default | Meaning |
|---|---|---|
| `type layer type name` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `ramp ramp name \| $name \| hex colour array` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `t0 number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `t1 number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `x number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `y number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `angle number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `squash number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `gain number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `cut number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `gamma number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `dither number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `glow number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `glowFalloff number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `blend string; see pixel-vfx for enumerated values` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `hidden boolean` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `seed number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `hue number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `ease string; see pixel-vfx for enumerated values` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `cell number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `fill number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `hex number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `r number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `shimmer number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `th number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |

<a id="file-vfx2d-layer-flash"></a>
### `file:vfx2d.layer.flash`

| Field | Default | Meaning |
|---|---|---|
| `type layer type name` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `ramp ramp name \| $name \| hex colour array` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `t0 number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `t1 number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `x number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `y number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `angle number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `squash number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `gain number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `cut number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `gamma number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `dither number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `glow number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `glowFalloff number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `blend string; see pixel-vfx for enumerated values` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `hidden boolean` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `seed number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `hue number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `ease string; see pixel-vfx for enumerated values` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `dur number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `r number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |

<a id="file-vfx2d-layer-text"></a>
### `file:vfx2d.layer.text`

| Field | Default | Meaning |
|---|---|---|
| `type layer type name` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `ramp ramp name \| $name \| hex colour array` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `t0 number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `t1 number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `x number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `y number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `angle number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `squash number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `gain number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `cut number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `gamma number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `dither number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `glow number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `glowFalloff number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `blend string; see pixel-vfx for enumerated values` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `hidden boolean` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `seed number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `hue number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `ease string; see pixel-vfx for enumerated values` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `text string` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `fadeStart number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `pop boolean` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `rise number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `scale number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |

<a id="file-vfx2d-layer-tornado"></a>
### `file:vfx2d.layer.tornado`

| Field | Default | Meaning |
|---|---|---|
| `type layer type name` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `ramp ramp name \| $name \| hex colour array` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `t0 number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `t1 number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `x number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `y number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `angle number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `squash number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `gain number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `cut number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `gamma number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `dither number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `glow number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `glowFalloff number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `blend string; see pixel-vfx for enumerated values` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `hidden boolean` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `seed number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `hue number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `ease string; see pixel-vfx for enumerated values` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `freq number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `hgt number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `rough number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `spin number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `sway number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `twist number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `w0 number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |
| `w1 number` | `renderer; see pixel-vfx` | Positions and distances use half the shorter canvas dimension; angles are degrees, times are normalised 0..1 unless stated in pixels/frames. Not every common transform changes every layer. |

<a id="file-video"></a>
### `file:video`

An authored `video` asset, cooked by `video`. Input: `.ogt` under assets/.

Field coverage: complete video parser field table. Sources require ffmpeg/ffprobe. Clip output is an ordinary video with frames and optional sound; stream output is chunked AV1 with declared renditions. Stream-only controls are refused on clips.

| Field | Default | Meaning |
|---|---|---|
| `source "<mp4/webm/mov>"` | `required` | Footage beside this file or under assets/; absolute and parent paths are refused. |
| `fps <integer 1..120>` | `30` | Frames per second kept. |
| `maxSeconds <seconds > 0>` | `8 (clip); whole source (stream)` | Keep source from its beginning for this duration. |
| `size <width> <height>` | `source capped at width 1024 (clip), 1920 (stream)` | Each dimension 2..4096; explicit resize. |
| `kind color \| pixel` | `color` | Frame texture processing kind. |
| `audio true \| false` | `true` | Keep source audio as a sound subasset when present. |
| `audioLocale "<locale>" "<audio file>"` | `` | Repeat alternate localized audio tracks; unique case-insensitive alphanumeric/hyphen/underscore tags. |
| `loudness <LUFS -30..0>` | `off` | Normalize source sound with ffmpeg loudnorm. |
| `aiGenerated true \| false` | `false` | Recorded provenance flag, not a generation request. |
| `placeholder true \| false` | `false` | Recorded stand-in flag, reported by og flow. |
| `alpha true \| false` | `false` | Retain source alpha; refuses a source without alpha. |
| `stream true \| false` | `false` | Encode chunked AV1 instead of a clip frame array. |
| `keyframe <seconds 0.1..10>` | `1` | Stream keyframe interval. |
| `chunk <seconds 0.5..60>` | `6` | Stream chunk interval, rounded to whole keyframe intervals. |
| `crf <integer 0..63>` | `32` | Stream AV1 constant rate factor. |
| `crf720 <integer 0..63>` | `min(crf + 6, 63)` | 720p rendition constant rate factor. |
| `posterSize 0 \| <integer 64..4096>` | `480` | Stream poster's longest side; zero keeps full resolution. |
| `renditions hevc h264 \| none` | `hevc when encoder available` | Explicit list of browser codec renditions beside AV1; none appears alone. |
| `lowRendition true \| false` | `false` | Also encode a 480p stream rendition. |

<a id="guide-file-types-2"></a>
### `file-types §2` Other engineering sources

<a id="file-project"></a>
### `file:project`

og-project.json is the ordinary project marker and sole name/engine/look authority. project.ts exports defineProject({...}); it is restricted bootstrap configuration, not an asset or a game entry. Run og check to generate contextual declarations in .og/types/author/. See start and conventions.

```ts
/** One root manifest for small and large projects. Omitted simulation
settings remain the game's responsibility, not inferred cook choices. */
export type ProjectInput = ({
readonly "budgets"?: (((ProjectInputDefinitions["Budgets"])) | ((null)));
readonly "conform"?: (((ProjectInputDefinitions["Conform"])) | ((null)));
readonly "cook"?: (((ProjectInputDefinitions["ProjectCook"])) | ((null)));
readonly "create"?: (((ProjectInputDefinitions["Creation"])) | ((null)));
/** Direct dependencies of the implicit base source module. */
readonly "dependsOn"?: (ReadonlyArray<(string)> | (number extends ((string)) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
/** Project-relative TypeScript game entry; bootstrap never executes it. */
readonly "entry": (string);
readonly "flow"?: (((ProjectInputDefinitions["ProjectFlow"])) | ((null)));
readonly "id": (string);
/** Explicit input globs for the base module, relative to its root. */
readonly "inputs"?: (ReadonlyArray<(string)> | (number extends ((string)) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "modules"?: (ReadonlyArray<(ProjectInputDefinitions["ModuleInput"])> | (number extends ((ProjectInputDefinitions["ModuleInput"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "packs"?: (ReadonlyArray<(ProjectInputDefinitions["DeliveryPack"])> | (number extends ((ProjectInputDefinitions["DeliveryPack"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
/** Named median-time limits in milliseconds, not cook quality choices. */
readonly "perf"?: (Readonly<Record<string, (number)>>);
readonly "publish"?: (((ProjectInputDefinitions["Publish"])) | ((null)));
readonly "roots"?: (ReadonlyArray<(ProjectInputDefinitions["ContentRef"])> | (number extends ((ProjectInputDefinitions["ContentRef"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "saveVersion"?: ((number) | (null));
readonly "scene"?: ((string) | (null));
readonly "scenes"?: (Readonly<Record<string, (ProjectInputDefinitions["ContentRef"])>>);
/** Pure record declaration files, never a running game entry. */
readonly "schemas"?: (ReadonlyArray<(string)> | (number extends ((string)) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
/** Named command scripts used by UI and style capture tools. */
readonly "screens"?: (Readonly<Record<string, (string)>>);
readonly "seed"?: ((number) | (null));
/** Pure Yarn expression functions, checked before dialogue compilation. */
readonly "storyFunctions"?: (Readonly<Record<string, (ProjectInputDefinitions["StoryFunction"])>>);
readonly "tick"?: ((number) | (null));
});
export interface ProjectInputDefinitions {
/** Explicit overrides of the existing content-report limits. Absence leaves
that limit's interpretation with the report, not the author evaluator. */
readonly "Budgets": ({
readonly "assetBytes"?: ((number) | (null));
readonly "bones"?: ((number) | (null));
readonly "bvhShare"?: ((number) | (null));
readonly "clips"?: ((number) | (null));
readonly "emitterParticles"?: ((number) | (null));
readonly "gpuBytes"?: ((number) | (null));
readonly "navPolygons"?: ((number) | (null));
readonly "sceneEntities"?: ((number) | (null));
readonly "soundSeconds"?: ((number) | (null));
readonly "textureSize"?: ((number) | (null));
readonly "totalBytes"?: ((number) | (null));
readonly "triangles"?: ((number) | (null));
readonly "vertices"?: ((number) | (null));
readonly "videoSeconds"?: ((number) | (null));
});
readonly "ColorEncoding": ("etc1s" | "uastc") & (string);
readonly "Conform": ({
readonly "off"?: (ReadonlyArray<(string)> | (number extends ((string)) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
});
/** A content identity, never a runtime handle or an instruction to evaluate
another entry. The project graph validates names, existence, subresources,
module access and target kinds. A field declares its allowed kinds with
`x-og-ref-kinds` schema metadata, not a second kind inside this token.
`x-og-ref-factories` instead requires an original declaration (no suffix),
for source consumers such as parameterized model programs or theme recipes. */
readonly "ContentRef": ({
readonly "$ogRef": (string);
});
readonly "Controls": ("touch" | "keyboard-mouse" | "gamepad") & (string);
/** Creation metadata describes the chosen shell. No genre composition or
balancing is selected by bootstrap; open catalogue IDs remain strings. */
readonly "Creation": ({
readonly "form"?: (((ProjectInputDefinitions["Form"])) | ((null)));
readonly "genre"?: ((string) | (null));
readonly "input"?: (((ProjectInputDefinitions["Controls"])) | ((null)));
readonly "lang"?: ((ReadonlyArray<(string)> | (number extends ((string)) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never)) | (null));
readonly "look"?: ((string) | (null));
readonly "music"?: ((string) | (null));
readonly "platform"?: (((ProjectInputDefinitions["Platform"])) | ((null)));
readonly "players"?: (((ProjectInputDefinitions["Players"])) | ((null)));
readonly "ui"?: ((string) | (null));
readonly "variant"?: ((string) | (null));
readonly "view"?: ((string) | (null));
});
/** An output delivery unit, not a module, stream group or world cell. */
readonly "DeliveryPack": ({
readonly "dependsOn"?: (ReadonlyArray<(string)> | (number extends ((string)) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "id": (string);
/** Namespaced content patterns. Overlap is an error, never first-wins. */
readonly "include"?: (ReadonlyArray<(string)> | (number extends ((string)) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "optional"?: (boolean);
readonly "roots"?: (ReadonlyArray<(ProjectInputDefinitions["ContentRef"])> | (number extends ((ProjectInputDefinitions["ContentRef"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
});
readonly "Fit": ("fill" | "letterbox") & (string);
readonly "FlowScreen": ("engine" | "game") & (string);
readonly "Form": ("2d" | "2.5d" | "3d") & (string);
readonly "Fullscreen": ("off" | "gesture") & (string);
readonly "Gate": ("tap" | "none") & (string);
readonly "Isolation": ("required" | "preferred" | "off") & (string);
readonly "Loading": ("engine" | "game" | "page") & (string);
readonly "LoadingLabels": ({
readonly "fetch"?: ((string) | (null));
readonly "present"?: ((string) | (null));
readonly "prime"?: ((string) | (null));
readonly "transcode"?: ((string) | (null));
});
/** A namespace and source root, independent of delivery and streaming. */
readonly "ModuleInput": ({
readonly "dependsOn"?: (ReadonlyArray<(string)> | (number extends ((string)) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "id": (string);
readonly "inputs"?: (ReadonlyArray<(string)> | (number extends ((string)) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
/** Canonical project-relative directory. Moving it preserves IDs. */
readonly "root": (string);
});
readonly "Orientation": ("any" | "landscape" | "portrait") & (string);
readonly "Pipelines": ("auto" | "sync" | "async" | "prewarm") & (string);
readonly "Platform": ("phone-portrait" | "phone-landscape" | "desktop" | "multi") & (string);
readonly "Players": ("single" | "local-multiplayer" | "online-lockstep") & (string);
readonly "Probe": ("player" | "all") & (string);
readonly "ProjectCook": ({
readonly "clusters"?: ((boolean) | (null));
readonly "colorTextures"?: (((ProjectInputDefinitions["ColorEncoding"])) | ((null)));
/** Explicit prebuilt bundle inputs. Names must remain globally unique. */
readonly "uses"?: (ReadonlyArray<(string)> | (number extends ((string)) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
});
readonly "ProjectFlow": ({
readonly "screen"?: (((ProjectInputDefinitions["FlowScreen"])) | ((null)));
readonly "source": (ProjectInputDefinitions["ContentRef"]);
});
readonly "Publish": ({
readonly "aspect"?: ((string) | (null));
readonly "aspectMax"?: ((string) | (null));
readonly "aspectMin"?: ((string) | (null));
readonly "assets"?: ((string) | (null));
readonly "colorTextures"?: (((ProjectInputDefinitions["ColorEncoding"])) | ((null)));
readonly "content"?: ((string) | (null));
readonly "dprCap"?: ((number) | (null));
readonly "fit"?: (((ProjectInputDefinitions["Fit"])) | ((null)));
readonly "fullscreen"?: (((ProjectInputDefinitions["Fullscreen"])) | ((null)));
readonly "gate"?: (((ProjectInputDefinitions["Gate"])) | ((null)));
readonly "icon"?: ((string) | (null));
readonly "isolation"?: (((ProjectInputDefinitions["Isolation"])) | ((null)));
readonly "keep"?: (ReadonlyArray<(string)> | (number extends ((string)) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "loading"?: (((ProjectInputDefinitions["Loading"])) | ((null)));
readonly "loadingLabels"?: (((ProjectInputDefinitions["LoadingLabels"])) | ((null)));
readonly "madeWith"?: ((boolean) | (null));
readonly "orientation"?: (((ProjectInputDefinitions["Orientation"])) | ((null)));
readonly "out"?: ((string) | (null));
readonly "pipelines"?: (((ProjectInputDefinitions["Pipelines"])) | ((null)));
readonly "probe"?: (((ProjectInputDefinitions["Probe"])) | ((null)));
readonly "shortName"?: ((string) | (null));
readonly "telemetry"?: (((ProjectInputDefinitions["Telemetry"])) | ((null)));
});
readonly "StoryFunction": ({
readonly "body": (string);
/** Declaration order is the function's argument order. */
readonly "parameters"?: (ReadonlyArray<(ProjectInputDefinitions["StoryParameter"])> | (number extends ((ProjectInputDefinitions["StoryParameter"])) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never));
readonly "returns": (ProjectInputDefinitions["StoryType"]);
});
readonly "StoryParameter": ({
readonly "kind": (ProjectInputDefinitions["StoryType"]);
readonly "name": (string);
});
readonly "StoryType": ("number" | "string" | "bool") & (string);
readonly "Telemetry": ("on" | "off") & (string);
}
```
<a id="file-import"></a>
### `file:import`

A source's .ogt sidecar or directory import.ogt: input settings, not an independent asset. See loading.

Directory import.ogt contains repeated `match "<pattern>" ... { ... }` blocks (patterns are directory-relative). Matching ancestors merge in order, then the file's own sidecar overrides by key. Root `conform off` opts out of the project art contract; a model's optional `fracture` block is below. Rive uses its own root artboard/font declarations.


<a id="file-import-model"></a>
### `file:import.model`

| Field | Default | Meaning |
|---|---|---|
| `lods <fraction> ...` | `1 0.5 0.25` | At most eight strictly decreasing positive fractions, first 1. |
| `lodError <0..1, >0>` | `0.05` | Largest simplification error relative to mesh extent. |
| `raycast true \| false` | `true` | Bake raycast BVH; false builds it on first use. |
| `collider true \| false \| convex \| vhacd` | `false` | Bake triangle, convex or decomposed collision. |
| `impostor true \| false` | `false` | Bake octahedral impostor. |
| `impostorGrid <integer 2..32>` | `8` | Impostor view grid. |
| `impostorSize <integer 16..512>` | `128` | Impostor frame pixels; total atlas width cannot exceed 4096. |
| `impostorBelow true \| false` | `false` | Include lower hemisphere. |
| `stream_textures true \| false` | `false` | Stream high mip levels, keeping the shared mip tail resident. |
| `process { ... }` | `off` | Shared static-mesh preparation; see file:model.process. |
| `relaxSkin true \| false` | `false` | Relax only skin-probe stretched regions before welding and LODs. Position seams share weights; at most 12 passes, with changed vertices and remaining stretch reported. |
| `front "+x" \| "-x" \| "+y" \| "-y" \| "+z" \| "-z"` | `source orientation` | Source forward axis, rotated to engine +Z. Must be perpendicular to up. |
| `up "+x" \| "-x" \| "+y" \| "-y" \| "+z" \| "-z"` | `+y when front is given` | Source up axis, rotated to engine +Y. |
| `voxScale <metres per voxel > 0>` | `1` | VOX sources only. |
| `voxOrigin center \| bottom` | `center` | VOX origin; bottom places the lowest voxel at y=0. |
| `voxFps <1..120>` | `10` | VOX animation frames per second. |
| `voxClip <name> <first u32> <last u32>` | `source clips` | VOX frame range, both ends inclusive; repeated unique names. |

<a id="file-import-texture"></a>
### `file:import.texture`

| Field | Default | Meaning |
|---|---|---|
| `kind color \| normal \| linear \| single \| pixel` | `color` | Image data interpretation and encoding. |
| `stream true \| false` | `false` | Stream high mip levels. |
| `normal auto \| "<image path>"` | `none` | 2D normal map generated from silhouette/luminance, or an authored tangent-space map beside the texture. |
| `sway clumps \| tiers ["<parts map>"]` | `none` | 2D plant part segmentation; optional same-size colour-labelled map keeps authored parts. |

<a id="file-import-font"></a>
### `file:import.font`

| Field | Default | Meaning |
|---|---|---|
| `pixel <integer 4..64>` | `outline font` | Pixel grid per em. Does not change other fonts' charset or mode. |

<a id="file-import-sound"></a>
### `file:import.sound`

| Field | Default | Meaning |
|---|---|---|
| `stream true \| false` | `auto above 1 MiB` | QOA chunk streaming; Opus streams codec blocks instead. |
| `chunk <seconds 1..60>` | `4` | QOA chunk duration rounded up to full codec frames. |
| `codec qoa \| opus` | `qoa` | Opus resamples to 48kHz and uses at most the first two channels. |
| `bitrate <16..256>` | `96 stereo / 64 mono` | Opus kb/s, rounded to nearest whole number. |

<a id="file-import-tilemap"></a>
### `file:import.tilemap`

| Field | Default | Meaning |
|---|---|---|
| `unit <number>` | `1` | World units per tile width. |
| `level <identifier>` | `first level` | LDtk level identifier. |
| `solid <i64> ...` | `1` | LDtk solid IntGrid values. |
| `tileset "<asset.ogt>"` | `source tilesets` | Replace every source tileset with this authored asset. |

<a id="file-import-rive"></a>
### `file:import.rive`

| Field | Default | Meaning |
|---|---|---|
| `artboards <name> ...` | `all` | Artboards to cook, named as in the source. |
| `artboard <name> { ... }` | `source defaults` | Per-artboard settings. |
| `font <name> { source "<path.ttf>" }` | `embedded font` | Supply an external font relative to assets/. |

<a id="file-import-rive-artboard"></a>
### `file:import.rive.artboard`

| Field | Default | Meaning |
|---|---|---|
| `state-machine <name>` | `source default` | Initial state machine. |
| `role <accessibility role>` | `none` | UI accessibility role; see ui accessibility vocabulary. |
| `name <string>` | `source name` | Accessible name. |
| `fit contain \| cover \| fill \| none \| scale-down \| fit-width \| fit-height` | `source default` | Artboard fit. |
| `align center \| top \| bottom \| left \| right \| top-left \| top-right \| bottom-left \| bottom-right` | `source default` | Artboard alignment. |

<a id="file-import-animation"></a>
### `file:import.animation`

| Field | Default | Meaning |
|---|---|---|
| `clip "<name>" { channel ... }` | `` | Explicit named-node keys. Names cannot collide with the source clips; same sampler and codec as glTF. |
| `rate <1..120 Hz>` | `source key rate` | Resample every clip at this rate. |
| `event "<clip>" "<marker>" <seconds>` | `` | Repeat explicit markers. |
| `rig "<asset>"` | `detected` | Humanoid rig map under assets/. |
| `library "<model>"` | `` | Repeat animation libraries to retarget to this skeleton; own clips win on name collisions. |
| `mirror "<clip>" ["<new name>"]` | `` | Repeat mirrored clips; default name is <clip>.mirrored. |
| `footIk true \| false` | `true` | Retarget planted feet against the ground. |
| `compress true \| false` | `false` | Quantize and reduce keys instead of storing f32 samples. |
| `tolerance <number > 0, <= 0.1>` | `0.0001` | Maximum compression error per channel lane in its units. |

<a id="file-import-animation-clip"></a>
### `file:import.animation.clip`

| Field | Default | Meaning |
|---|---|---|
| `channel "<node>" translation \| rotation \| scale \| weights \| visibility { ... }` | `required` | Repeated channels; node names must resolve uniquely. Metres, xyzw unit quaternions, scale factors, morph weights or 0/1 visibility. |

<a id="file-import-animation-clip-channel"></a>
### `file:import.animation.clip.channel`

| Field | Default | Meaning |
|---|---|---|
| `interpolation step \| linear \| cubicSpline` | `linear` | glTF interpolation; visibility requires step. Cubic keys contain in-tangent, value, out-tangent; tangents are per second. |
| `key <seconds >= 0> <values...>` | `required` | Strictly increasing times. Translation/scale have 3 lanes, rotation 4, weights one per morph target, visibility 1; cubic keys have three times as many values. |

<a id="file-import-fracture"></a>
### `file:import.fracture`

| Field | Default | Meaning |
|---|---|---|
| `style voronoi \| concrete \| brick \| wood \| glass` | `voronoi` | Fracture mechanism, not a gameplay rule. |
| `chunks <2..256>` | `16` | Support chunk count. |
| `seed <u32>` | `1` | Deterministic cut pattern. |
| `site <x> <y> <z>` | `` | Repeat 2..128 explicit Voronoi centres in model metres; requires style voronoi and pattern volume, replaces automatic support-cell sampling. Site order fixes cell ids. Children retain their declared generator. |
| `children <0..32>` | `0` | Child pieces per support chunk. |
| `pattern auto \| volume \| slab` | `auto` | 3D cells or shards through the thinnest axis; auto detects thin input. |
| `brick <length> <height>` | `0.25 0.1` | Brick dimensions, each at least 0.01 metres. |
| `grain x \| y \| z` | `longest axis` | Wood grain axis. |
| `rings <1..8>` | `2` | Glass concentric cracks. |
| `impact <x> <y> <z> <radius>` | `` | Gather the pattern around an authored model-space impact; radius positive. |
| `anchor bottom [ends] \| ends \| none \| plane <nx> <ny> <nz> <distance>` | `bottom` | Support surfaces; plane normal must be nonzero. |
| `interiorScale <number > 0>` | `1` | Cut-face UV repeats per metre. |
| `interior { ... }` | `darkened main material` | Cut-face material using model.material fields. |
| `mortar { ... }` | `` | Debris material using model.material fields. |
<a id="file-model-program"></a>
### `file:model-program`

.shape.json: model program operations; consumed by model. See model.


<a id="file-model-program-surface"></a>
### `file:model-program.surface`

| Field | Default | Meaning |
|---|---|---|
| `density number` | `0 (kernel default)` | Texels per model unit. |
| `seed u32` | `0` | Surface random seed. |
| `normalScale number` | `0 (kernel default)` | Relief-to-normal strength. |
| `layers [layer, ...]` | `[]` | Ordered procedural paint layers. |

<a id="file-model-program-surface-layer"></a>
### `file:model-program.surface.layer`

| Field | Default | Meaning |
|---|---|---|
| `pattern noise \| ridged \| craters \| strata \| stripes \| panels \| rivets \| veins \| edges \| cavities \| facing \| gradient` | `required` | Shared paint mechanism. |
| `scale number` | `0 (kernel default)` | Pattern scale in model units. |
| `width number` | `0 (kernel default)` | Pattern width. |
| `cell number` | `0 (kernel default)` | Pattern cell size. |
| `inset number` | `0 (kernel default)` | Panel inset. |
| `axis signed axis string \| [x,y,z]` | `+Z` | Projection direction. |
| `octaves u32` | `0 (kernel default)` | Noise octave count. |
| `threshold number` | `pattern default` | Pattern threshold. |
| `softness number` | `0 (kernel default)` | Threshold softness. |
| `seed u32` | `0` | Layer random seed. |
| `amount number` | `1` | Layer blend weight. |
| `breakup number` | `0` | Pattern breakup. |
| `color hex string \| [r,g,b]` | `unchanged` | Layer colour. |
| `jitter number` | `0` | Colour variation. |
| `roughness number` | `unchanged` | Layer roughness. |
| `metalness number` | `unchanged` | Layer metallic factor. |
| `emissive hex string \| [r,g,b]` | `unchanged` | Emitted colour. |
| `glow number` | `1` | Emission strength. |
| `relief number` | `0` | Surface displacement. |
The operation and profile key sets below are the exact tables used to reject unknown JSON fields; parameter expressions are resolved before dispatch. For value types, units and construction examples see model. These are JSON objects, not OGT blocks. The op discriminator selects an operation, and a one-key profile object selects a profile.

<a id="file-model-program-op-frame"></a>
### `file:model-program.op.frame`

| Field | Default | Meaning |
|---|---|---|
| `origin [x,y,z]; origin in model units, others directions` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `normal [x,y,z]; origin in model units, others directions` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `up [x,y,z]; origin in model units, others directions` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |

<a id="file-model-program-op-offset"></a>
### `file:model-program.op.offset`

| Field | Default | Meaning |
|---|---|---|
| `frame {origin:[x,y,z], x:[x,y,z], y:[x,y,z], z:[x,y,z]}; orthonormal frame` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `distance number; model units` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |

<a id="file-model-program-op-translate"></a>
### `file:model-program.op.translate`

| Field | Default | Meaning |
|---|---|---|
| `frame {origin:[x,y,z], x:[x,y,z], y:[x,y,z], z:[x,y,z]}; orthonormal frame` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `x number; model units` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `y number; model units` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `z number; model units` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |

<a id="file-model-program-op-turn"></a>
### `file:model-program.op.turn`

| Field | Default | Meaning |
|---|---|---|
| `frame {origin:[x,y,z], x:[x,y,z], y:[x,y,z], z:[x,y,z]}; orthonormal frame` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `rx number; radians` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `ry number; radians` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `rz number; radians` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |

<a id="file-model-program-op-compose"></a>
### `file:model-program.op.compose`

| Field | Default | Meaning |
|---|---|---|
| `frame {origin:[x,y,z], x:[x,y,z], y:[x,y,z], z:[x,y,z]}; orthonormal frame` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `local {origin:[x,y,z], x:[x,y,z], y:[x,y,z], z:[x,y,z]}; orthonormal frame` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |

<a id="file-model-program-op-bbox"></a>
### `file:model-program.op.bbox`

| Field | Default | Meaning |
|---|---|---|
| `ids part id \| [part id, ...]` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `sides string \| [string, ...]; named faces/edges/sides` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |

<a id="file-model-program-op-face"></a>
### `file:model-program.op.face`

| Field | Default | Meaning |
|---|---|---|
| `id u32 part id \| {id:u32}` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `name string; font/map names are tracked source assets` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |

<a id="file-model-program-op-edge"></a>
### `file:model-program.op.edge`

| Field | Default | Meaning |
|---|---|---|
| `id u32 part id \| {id:u32}` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `name string; font/map names are tracked source assets` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |

<a id="file-model-program-op-local"></a>
### `file:model-program.op.local`

| Field | Default | Meaning |
|---|---|---|
| `id u32 part id \| {id:u32}` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `frame {origin:[x,y,z], x:[x,y,z], y:[x,y,z], z:[x,y,z]}; orthonormal frame` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |

<a id="file-model-program-op-anchor"></a>
### `file:model-program.op.anchor`

| Field | Default | Meaning |
|---|---|---|
| `id u32 part id \| {id:u32}` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `name string; font/map names are tracked source assets` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |

<a id="file-model-program-op-placement"></a>
### `file:model-program.op.placement`

| Field | Default | Meaning |
|---|---|---|
| `id u32 part id \| {id:u32}` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |

<a id="file-model-program-op-grid"></a>
### `file:model-program.op.grid`

| Field | Default | Meaning |
|---|---|---|
| `nx u32; zero requests the mechanism's automatic choice where supported` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `ny u32; zero requests the mechanism's automatic choice where supported` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `px number; model units` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `py number; model units` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `stagger boolean` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `on {origin:[x,y,z], x:[x,y,z], y:[x,y,z], z:[x,y,z]}; orthonormal frame` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |

<a id="file-model-program-op-ring"></a>
### `file:model-program.op.ring`

| Field | Default | Meaning |
|---|---|---|
| `n u32; zero requests the mechanism's automatic choice where supported` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `radius number; model units` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `start number; radians` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `angle number; radians` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `on {origin:[x,y,z], x:[x,y,z], y:[x,y,z], z:[x,y,z]}; orthonormal frame` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |

<a id="file-model-program-op-along"></a>
### `file:model-program.op.along`

| Field | Default | Meaning |
|---|---|---|
| `path {points:[[x,y,z], ...], closed:boolean}` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `pitch number; model units` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `count u32; zero requests the mechanism's automatic choice where supported` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |

<a id="file-model-program-op-snap"></a>
### `file:model-program.op.snap`

| Field | Default | Meaning |
|---|---|---|
| `ids part id \| [part id, ...]` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `points [[x,y,z], ...]; model units` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `direction [x,y,z]; origin in model units, others directions` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |

<a id="file-model-program-op-names"></a>
### `file:model-program.op.names`

| Field | Default | Meaning |
|---|---|---|
| `id u32 part id \| {id:u32}` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |

<a id="file-model-program-op-measure"></a>
### `file:model-program.op.measure`

| Field | Default | Meaning |
|---|---|---|
| `posed {jointName:number, ...}; optional pose for measurement` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |

<a id="file-model-program-op-units"></a>
### `file:model-program.op.units`

| Field | Default | Meaning |
|---|---|---|
| `unit "mm" \| "cm" \| "m"` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |

<a id="file-model-program-op-uvs"></a>
### `file:model-program.op.uvs`

| Field | Default | Meaning |
|---|---|---|

<a id="file-model-program-op-box"></a>
### `file:model-program.op.box`

| Field | Default | Meaning |
|---|---|---|
| `name string; font/map names are tracked source assets` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `width number; model units` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `depth number; model units` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `height number; model units` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `uvs boolean` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `on {origin:[x,y,z], x:[x,y,z], y:[x,y,z], z:[x,y,z]}; orthonormal frame` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |

<a id="file-model-program-op-cylinder"></a>
### `file:model-program.op.cylinder`

| Field | Default | Meaning |
|---|---|---|
| `name string; font/map names are tracked source assets` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `radius number; model units` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `radiusTop number; model units` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `height number; model units` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `segments u32; zero requests the mechanism's automatic choice where supported` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `uvs boolean` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `on {origin:[x,y,z], x:[x,y,z], y:[x,y,z], z:[x,y,z]}; orthonormal frame` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |

<a id="file-model-program-op-cone"></a>
### `file:model-program.op.cone`

| Field | Default | Meaning |
|---|---|---|
| `name string; font/map names are tracked source assets` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `radius number; model units` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `height number; model units` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `segments u32; zero requests the mechanism's automatic choice where supported` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `uvs boolean` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `on {origin:[x,y,z], x:[x,y,z], y:[x,y,z], z:[x,y,z]}; orthonormal frame` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |

<a id="file-model-program-op-sphere"></a>
### `file:model-program.op.sphere`

| Field | Default | Meaning |
|---|---|---|
| `name string; font/map names are tracked source assets` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `radius number; model units` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `axis "+Y" \| "+Z"` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `segments u32; zero requests the mechanism's automatic choice where supported` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `uvs boolean` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `on {origin:[x,y,z], x:[x,y,z], y:[x,y,z], z:[x,y,z]}; orthonormal frame` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |

<a id="file-model-program-op-dome"></a>
### `file:model-program.op.dome`

| Field | Default | Meaning |
|---|---|---|
| `name string; font/map names are tracked source assets` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `radius number; model units` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `height number; model units` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `axis "+Y" \| "+Z"` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `segments u32; zero requests the mechanism's automatic choice where supported` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `uvs boolean` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `on {origin:[x,y,z], x:[x,y,z], y:[x,y,z], z:[x,y,z]}; orthonormal frame` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |

<a id="file-model-program-op-tube"></a>
### `file:model-program.op.tube`

| Field | Default | Meaning |
|---|---|---|
| `name string; font/map names are tracked source assets` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `outer number; model units` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `inner number; model units` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `height number; model units` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `segments u32; zero requests the mechanism's automatic choice where supported` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `uvs boolean` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `on {origin:[x,y,z], x:[x,y,z], y:[x,y,z], z:[x,y,z]}; orthonormal frame` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |

<a id="file-model-program-op-torus"></a>
### `file:model-program.op.torus`

| Field | Default | Meaning |
|---|---|---|
| `name string; font/map names are tracked source assets` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `radius number; model units` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `tube number; model units` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `angle number; radians` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `axis "+Y" \| "+Z"` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `segments u32; zero requests the mechanism's automatic choice where supported` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `tubeSegments u32; zero requests the mechanism's automatic choice where supported` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `uvs boolean` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `on {origin:[x,y,z], x:[x,y,z], y:[x,y,z], z:[x,y,z]}; orthonormal frame` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |

<a id="file-model-program-op-wedge"></a>
### `file:model-program.op.wedge`

| Field | Default | Meaning |
|---|---|---|
| `name string; font/map names are tracked source assets` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `width number; model units` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `height number; model units` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `depth number; model units` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `uvs boolean` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `on {origin:[x,y,z], x:[x,y,z], y:[x,y,z], z:[x,y,z]}; orthonormal frame` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |

<a id="file-model-program-op-extrude"></a>
### `file:model-program.op.extrude`

| Field | Default | Meaning |
|---|---|---|
| `name string; font/map names are tracked source assets` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `profile one-key profile object; see file:model-program.profile.*` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `height number; model units` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `twist number; radians` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `taper number; dimensionless` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `bothSides boolean` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `until {part: id, face: string}; extrude until the selected face` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `uvs boolean` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `on {origin:[x,y,z], x:[x,y,z], y:[x,y,z], z:[x,y,z]}; orthonormal frame` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |

<a id="file-model-program-op-revolve"></a>
### `file:model-program.op.revolve`

| Field | Default | Meaning |
|---|---|---|
| `name string; font/map names are tracked source assets` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `profile one-key profile object; see file:model-program.profile.*` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `angle number; radians` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `axis "+Y" \| "+Z"` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `segments u32; zero requests the mechanism's automatic choice where supported` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `uvs boolean` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `on {origin:[x,y,z], x:[x,y,z], y:[x,y,z], z:[x,y,z]}; orthonormal frame` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |

<a id="file-model-program-op-sweep"></a>
### `file:model-program.op.sweep`

| Field | Default | Meaning |
|---|---|---|
| `name string; font/map names are tracked source assets` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `profile one-key profile object; see file:model-program.profile.*` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `path point/piece array; arc {through,to}, tangentArc {to}, spline {points}; 3D also {helix:{radius,pitch,turns}}` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `up [x,y,z]; origin in model units, others directions` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `twist number; radians` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `scaleEnd number; dimensionless` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `uvs boolean` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `on {origin:[x,y,z], x:[x,y,z], y:[x,y,z], z:[x,y,z]}; orthonormal frame` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |

<a id="file-model-program-op-loft"></a>
### `file:model-program.op.loft`

| Field | Default | Meaning |
|---|---|---|
| `name string; font/map names are tracked source assets` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `sections [{profile, z:number}, ...] \| [{profile, at:frame}, ...]; no mixed placement forms` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `samples u32; zero requests the mechanism's automatic choice where supported` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `steps u32; zero requests the mechanism's automatic choice where supported` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `curve "auto" \| "straight" \| "catmullRom" \| "spline"; placed sections only straight/catmullRom` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `closed boolean` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `uvs boolean` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `on {origin:[x,y,z], x:[x,y,z], y:[x,y,z], z:[x,y,z]}; orthonormal frame` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |

<a id="file-model-program-op-thread"></a>
### `file:model-program.op.thread`

| Field | Default | Meaning |
|---|---|---|
| `name string; font/map names are tracked source assets` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `radius number; model units` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `pitch number; model units` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `length number; model units` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `depth number; model units` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `uvs boolean` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `on {origin:[x,y,z], x:[x,y,z], y:[x,y,z], z:[x,y,z]}; orthonormal frame` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |

<a id="file-model-program-op-wire"></a>
### `file:model-program.op.wire`

| Field | Default | Meaning |
|---|---|---|
| `name string; font/map names are tracked source assets` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `path point/piece array; arc {through,to}, tangentArc {to}, spline {points}; 3D also {helix:{radius,pitch,turns}}` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `diameter number; model units` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `closed boolean` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `fillet number; model units` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `fillets [number, ...]; one model-unit radius per path point` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `segments u32; zero requests the mechanism's automatic choice where supported` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `uvs boolean` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `on {origin:[x,y,z], x:[x,y,z], y:[x,y,z], z:[x,y,z]}; orthonormal frame` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |

<a id="file-model-program-op-body"></a>
### `file:model-program.op.body`

| Field | Default | Meaning |
|---|---|---|
| `name string; font/map names are tracked source assets` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `plan number \| [[y,value], ...] \| {points: [[y,value],...], smooth: boolean}; model units` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `top number \| [[y,value], ...] \| {points: [[y,value],...], smooth: boolean}; model units` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `bottom number \| [[y,value], ...] \| {points: [[y,value],...], smooth: boolean}; model units` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `n number \| [[y,value], ...] \| {points: [[y,value],...], smooth: boolean}; model units` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `uvs boolean` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `on {origin:[x,y,z], x:[x,y,z], y:[x,y,z], z:[x,y,z]}; orthonormal frame` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |

<a id="file-model-program-op-knurl"></a>
### `file:model-program.op.knurl`

| Field | Default | Meaning |
|---|---|---|
| `name string; font/map names are tracked source assets` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `diameter number; model units` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `height number; model units` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `pitch number; model units` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `straight boolean` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `uvs boolean` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `on {origin:[x,y,z], x:[x,y,z], y:[x,y,z], z:[x,y,z]}; orthonormal frame` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |

<a id="file-model-program-op-gear"></a>
### `file:model-program.op.gear`

| Field | Default | Meaning |
|---|---|---|
| `name string; font/map names are tracked source assets` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `teeth u32; zero requests the mechanism's automatic choice where supported` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `module number; model units` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `pressureAngle number; radians` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `width number; model units` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `bore number; model units` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `hub number; model units` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `hubHeight number; model units` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `uvs boolean` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `on {origin:[x,y,z], x:[x,y,z], y:[x,y,z], z:[x,y,z]}; orthonormal frame` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |

<a id="file-model-program-op-at"></a>
### `file:model-program.op.at`

| Field | Default | Meaning |
|---|---|---|
| `id u32 part id \| {id:u32}` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `frame {origin:[x,y,z], x:[x,y,z], y:[x,y,z], z:[x,y,z]}; orthonormal frame` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |

<a id="file-model-program-op-transform"></a>
### `file:model-program.op.transform`

| Field | Default | Meaning |
|---|---|---|
| `id u32 part id \| {id:u32}` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `frame {origin:[x,y,z], x:[x,y,z], y:[x,y,z], z:[x,y,z]}; orthonormal frame` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |

<a id="file-model-program-op-group"></a>
### `file:model-program.op.group`

| Field | Default | Meaning |
|---|---|---|
| `name string; font/map names are tracked source assets` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `children part id \| [part id, ...]` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |

<a id="file-model-program-op-wrap"></a>
### `file:model-program.op.wrap`

| Field | Default | Meaning |
|---|---|---|
| `name string; font/map names are tracked source assets` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `parts part id \| [part id, ...]` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |

<a id="file-model-program-op-assembly"></a>
### `file:model-program.op.assembly`

| Field | Default | Meaning |
|---|---|---|
| `name string; font/map names are tracked source assets` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `children part id \| [part id, ...]` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `anchors {name: frame, ...}` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |

<a id="file-model-program-op-attach"></a>
### `file:model-program.op.attach`

| Field | Default | Meaning |
|---|---|---|
| `to u32 part id \| {id:u32}` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `name string; font/map names are tracked source assets` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `children part id \| [part id, ...]` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |

<a id="file-model-program-op-copy"></a>
### `file:model-program.op.copy`

| Field | Default | Meaning |
|---|---|---|
| `id u32 part id \| {id:u32}` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `name string; font/map names are tracked source assets` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |

<a id="file-model-program-op-mirror"></a>
### `file:model-program.op.mirror`

| Field | Default | Meaning |
|---|---|---|
| `id u32 part id \| {id:u32}` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `plane {origin:[x,y,z], x:[x,y,z], y:[x,y,z], z:[x,y,z]}; orthonormal frame` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `name string; font/map names are tracked source assets` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |

<a id="file-model-program-op-cut"></a>
### `file:model-program.op.cut`

| Field | Default | Meaning |
|---|---|---|
| `target u32 part id \| {id:u32}` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `tool u32 part id \| {id:u32}` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `check boolean` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |

<a id="file-model-program-op-join"></a>
### `file:model-program.op.join`

| Field | Default | Meaning |
|---|---|---|
| `target u32 part id \| {id:u32}` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `tool u32 part id \| {id:u32}` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `check boolean` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |

<a id="file-model-program-op-intersect"></a>
### `file:model-program.op.intersect`

| Field | Default | Meaning |
|---|---|---|
| `target u32 part id \| {id:u32}` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `tool u32 part id \| {id:u32}` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `check boolean` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |

<a id="file-model-program-op-merge"></a>
### `file:model-program.op.merge`

| Field | Default | Meaning |
|---|---|---|
| `target u32 part id \| {id:u32}` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `tool u32 part id \| {id:u32}` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |

<a id="file-model-program-op-delete"></a>
### `file:model-program.op.delete`

| Field | Default | Meaning |
|---|---|---|
| `id u32 part id \| {id:u32}` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |

<a id="file-model-program-op-seat"></a>
### `file:model-program.op.seat`

| Field | Default | Meaning |
|---|---|---|
| `host u32 part id \| {id:u32}` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `part u32 part id \| {id:u32}` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |

<a id="file-model-program-op-scale"></a>
### `file:model-program.op.scale`

| Field | Default | Meaning |
|---|---|---|
| `id u32 part id \| {id:u32}` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `x number; scale factor (zero keeps one)` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `y number; scale factor (zero keeps one)` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `z number; scale factor (zero keeps one)` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |

<a id="file-model-program-op-hole"></a>
### `file:model-program.op.hole`

| Field | Default | Meaning |
|---|---|---|
| `id u32 part id \| {id:u32}` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `diameter number; model units` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `at frame \| [frame, ...]; model units` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `depth number; model units` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `counterbore [number,number]; range in joint units, counterbore [diameter,depth], countersink [diameter,radians]` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `countersink [number,number]; range in joint units, counterbore [diameter,depth], countersink [diameter,radians]` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |

<a id="file-model-program-op-pocket"></a>
### `file:model-program.op.pocket`

| Field | Default | Meaning |
|---|---|---|
| `id u32 part id \| {id:u32}` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `profile one-key profile object; see file:model-program.profile.*` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `depth number; model units` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `at frame \| [frame, ...]; model units` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |

<a id="file-model-program-op-split"></a>
### `file:model-program.op.split`

| Field | Default | Meaning |
|---|---|---|
| `id u32 part id \| {id:u32}` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `frame {origin:[x,y,z], x:[x,y,z], y:[x,y,z], z:[x,y,z]}; orthonormal frame` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `keep "above" \| "below"` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |

<a id="file-model-program-op-shell"></a>
### `file:model-program.op.shell`

| Field | Default | Meaning |
|---|---|---|
| `id u32 part id \| {id:u32}` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `thickness number; model units` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `open string \| [string, ...]; named faces/edges/sides` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |

<a id="file-model-program-op-dent"></a>
### `file:model-program.op.dent`

| Field | Default | Meaning |
|---|---|---|
| `id u32 part id \| {id:u32}` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `at [[x,y,z], ...]; model units` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `radius number; model units` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `depth number; model units` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |

<a id="file-model-program-op-subdivide"></a>
### `file:model-program.op.subdivide`

| Field | Default | Meaning |
|---|---|---|
| `id u32 part id \| {id:u32}` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `levels u32; zero requests the mechanism's automatic choice where supported` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `sharp string \| [string, ...]; named faces/edges/sides` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |

<a id="file-model-program-op-displace"></a>
### `file:model-program.op.displace`

| Field | Default | Meaning |
|---|---|---|
| `id u32 part id \| {id:u32}` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `noise "fbm" \| "ridged" \| "crater"` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `scale number; noise frequency scale` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `amount number; model units` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `octaves u32; zero requests the mechanism's automatic choice where supported` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `seed u32; zero requests the mechanism's automatic choice where supported` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `density number; dimensionless` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |

<a id="file-model-program-op-fillet"></a>
### `file:model-program.op.fillet`

| Field | Default | Meaning |
|---|---|---|
| `id u32 part id \| {id:u32}` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `radius number; model units` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `edges string \| [string, ...]; named faces/edges/sides` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `segments u32; zero requests the mechanism's automatic choice where supported` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `profile number; bevel profile factor` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |

<a id="file-model-program-op-chamfer"></a>
### `file:model-program.op.chamfer`

| Field | Default | Meaning |
|---|---|---|
| `id u32 part id \| {id:u32}` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `distance number; model units` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `distance2 number; model units` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `angle number; radians` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `edges string \| [string, ...]; named faces/edges/sides` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `segments u32; zero requests the mechanism's automatic choice where supported` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |

<a id="file-model-program-op-soften"></a>
### `file:model-program.op.soften`

| Field | Default | Meaning |
|---|---|---|
| `id u32 part id \| {id:u32}` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `radius number; model units` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `angle number; radians` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `edges string \| [string, ...]; named faces/edges/sides` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `segments u32; zero requests the mechanism's automatic choice where supported` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |

<a id="file-model-program-op-material"></a>
### `file:model-program.op.material`

| Field | Default | Meaning |
|---|---|---|
| `ids part id \| [part id, ...]` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `name string; font/map names are tracked source assets` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `color "#rrggbb" \| [r,g,b]; sRGB` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `metalness number; dimensionless` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `roughness number; dimensionless` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `emissive "#rrggbb" \| [r,g,b]; sRGB` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `opacity number; dimensionless` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `map string; font/map names are tracked source assets` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `mapSize number; model units` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `clearcoat number; dimensionless` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `clearcoatRoughness number; dimensionless` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `sheenColor "#rrggbb" \| [r,g,b]; sRGB` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `sheenRoughness number; dimensionless` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `transmission number; dimensionless` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `iridescence number; dimensionless` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `surface {density?, seed?, normalScale?, layers?:[layer,...]}; see file:model-program.surface` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |

<a id="file-model-program-op-joint"></a>
### `file:model-program.op.joint`

| Field | Default | Meaning |
|---|---|---|
| `child u32 part id \| {id:u32}` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `parent u32 part id \| {id:u32}` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `name string; font/map names are tracked source assets` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `kind "revolute" \| "continuous" \| "prismatic" \| "fixed" \| "floating"` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `on {origin:[x,y,z], x:[x,y,z], y:[x,y,z], z:[x,y,z]}; orthonormal frame` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `axis [x,y,z]; local direction` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `range [number,number]; range in joint units, counterbore [diameter,depth], countersink [diameter,radians]` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `mimic {joint:string, multiplier:number}` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |

<a id="file-model-program-op-pose"></a>
### `file:model-program.op.pose`

| Field | Default | Meaning |
|---|---|---|
| `joint string; font/map names are tracked source assets` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `value number; radians for rotary joints, model units for prismatic joints` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `clip string; font/map names are tracked source assets` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `time number; seconds` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |

<a id="file-model-program-op-clip"></a>
### `file:model-program.op.clip`

| Field | Default | Meaning |
|---|---|---|
| `name string; font/map names are tracked source assets` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `keys [{time: seconds, pose: {jointName: number}}]; strictly increasing times` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |

<a id="file-model-program-op-skin"></a>
### `file:model-program.op.skin`

| Field | Default | Meaning |
|---|---|---|
| `id u32 part id \| {id:u32}` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `bones [{name:string, parent?:string, pivot?:[x,y,z] or frame, regions?:[faceName,...]}, ...]; parent first` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `joints [{name:string, parent?:string, at:[x,y,z]}, ...]; bone-heat skeleton, exclusive with bones` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `blend number; dimensionless` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `maxEdge number; model units` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |

<a id="file-model-program-op-humanoid"></a>
### `file:model-program.op.humanoid`

| Field | Default | Meaning |
|---|---|---|
| `id u32 part id \| {id:u32}` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `bind boolean` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |

<a id="file-model-program-op-morph"></a>
### `file:model-program.op.morph`

| Field | Default | Meaning |
|---|---|---|
| `id u32 part id \| {id:u32}` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `name string; font/map names are tracked source assets` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `target part id; exclusive with deltas/regions` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `deltas [[x,y,z], ...]; model units` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `regions [string, ...]; named faces` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `offset [x,y,z]; displacement in model units / scale factors` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `scale [x,y,z]; displacement in model units / scale factors` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `weight number; dimensionless` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |

<a id="file-model-program-profile-rect"></a>
### `file:model-program.profile.rect`

| Field | Default | Meaning |
|---|---|---|
| `width number; model units` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `height number; model units` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `radius number; model units` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |

<a id="file-model-program-profile-circle"></a>
### `file:model-program.profile.circle`

| Field | Default | Meaning |
|---|---|---|
| `radius number; model units` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `segments u32; zero requests the mechanism's automatic choice where supported` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |

<a id="file-model-program-profile-ellipse"></a>
### `file:model-program.profile.ellipse`

| Field | Default | Meaning |
|---|---|---|
| `rx number; model units` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `ry number; model units` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `segments u32; zero requests the mechanism's automatic choice where supported` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |

<a id="file-model-program-profile-slot"></a>
### `file:model-program.profile.slot`

| Field | Default | Meaning |
|---|---|---|
| `length number; model units` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `width number; model units` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |

<a id="file-model-program-profile-polygon"></a>
### `file:model-program.profile.polygon`

| Field | Default | Meaning |
|---|---|---|
| `sides u32; polygon side count` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `radius number; model units` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `flats number; model units` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |

<a id="file-model-program-profile-poly"></a>
### `file:model-program.profile.poly`

| Field | Default | Meaning |
|---|---|---|
| `points [[x,y], ...]; poly also accepts {x,y,radius?,bulge?} (bulge in half-chords)` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |

<a id="file-model-program-profile-gear"></a>
### `file:model-program.profile.gear`

| Field | Default | Meaning |
|---|---|---|
| `teeth u32; zero requests the mechanism's automatic choice where supported` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `module number; model units` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `pressureAngle number; radians` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |

<a id="file-model-program-profile-union"></a>
### `file:model-program.profile.union`

| Field | Default | Meaning |
|---|---|---|

<a id="file-model-program-profile-subtract"></a>
### `file:model-program.profile.subtract`

| Field | Default | Meaning |
|---|---|---|

<a id="file-model-program-profile-intersect"></a>
### `file:model-program.profile.intersect`

| Field | Default | Meaning |
|---|---|---|

<a id="file-model-program-profile-offset"></a>
### `file:model-program.profile.offset`

| Field | Default | Meaning |
|---|---|---|
| `profile one-key profile object; see file:model-program.profile.*` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `distance number; model units` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |

<a id="file-model-program-profile-translate"></a>
### `file:model-program.profile.translate`

| Field | Default | Meaning |
|---|---|---|
| `profile one-key profile object; see file:model-program.profile.*` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `x number; model units` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `y number; model units` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |

<a id="file-model-program-profile-rotate"></a>
### `file:model-program.profile.rotate`

| Field | Default | Meaning |
|---|---|---|
| `profile one-key profile object; see file:model-program.profile.*` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `angle number; radians` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |

<a id="file-model-program-profile-scale"></a>
### `file:model-program.profile.scale`

| Field | Default | Meaning |
|---|---|---|
| `profile one-key profile object; see file:model-program.profile.*` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `x number; scale factor (zero keeps one)` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `y number; scale factor (zero keeps one)` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |

<a id="file-model-program-profile-superellipse"></a>
### `file:model-program.profile.superellipse`

| Field | Default | Meaning |
|---|---|---|
| `rx number; model units` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `ry number; model units` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `exponent number; dimensionless` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `segments u32; zero requests the mechanism's automatic choice where supported` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |

<a id="file-model-program-profile-spline"></a>
### `file:model-program.profile.spline`

| Field | Default | Meaning |
|---|---|---|
| `points [[x,y], ...]; poly also accepts {x,y,radius?,bulge?} (bulge in half-chords)` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |

<a id="file-model-program-profile-path"></a>
### `file:model-program.profile.path`

| Field | Default | Meaning |
|---|---|---|

<a id="file-model-program-profile-stroke"></a>
### `file:model-program.profile.stroke`

| Field | Default | Meaning |
|---|---|---|
| `path point/piece array; arc {through,to}, tangentArc {to}, spline {points}; 3D also {helix:{radius,pitch,turns}}` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `width number; model units` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `join "round" \| "miter" \| "bevel"` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `roundCaps boolean` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `closed boolean` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |

<a id="file-model-program-profile-hull"></a>
### `file:model-program.profile.hull`

| Field | Default | Meaning |
|---|---|---|

<a id="file-model-program-profile-mirror"></a>
### `file:model-program.profile.mirror`

| Field | Default | Meaning |
|---|---|---|
| `profile one-key profile object; see file:model-program.profile.*` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `axis "x" \| "y"` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |

<a id="file-model-program-profile-text"></a>
### `file:model-program.profile.text`

| Field | Default | Meaning |
|---|---|---|
| `text string; font/map names are tracked source assets` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `font string; font/map names are tracked source assets` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `size number; model units` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
| `spacing number; model units` | `see model mechanism` | Numeric fields also accept $parameter expressions; geometry constraints are checked by the shared kernel. Model programs use Z up; cooked models convert to metres/Y up. |
<a id="file-theme"></a>
### `file:theme`

theme.json: compiles to pack.json and UI art. See themes and theme-tokens.

| Field | Default | Meaning |
|---|---|---|
| `about string` | `inherited` | Author note. |
| `extends theme name` | `inherited` | Resolve parent first; cycles fail. |
| `with [mixin name, ...]` | `inherited` | Merge mixins in order after extends, before own keys; mixins cannot themselves extend or mix in. |
| `color {source?,accent?,neutral?,danger?,variant?,scheme?}` | `inherited` | String-valued palette settings; scheme is light or dark. |
| `material dimension name` | `inherited` | Select a dimension value; no new artwork or gameplay is inferred. |
| `shape dimension name` | `inherited` | Select a dimension value; no new artwork or gameplay is inferred. |
| `type dimension name` | `inherited` | Select a dimension value; no new artwork or gameplay is inferred. |
| `motion dimension name` | `inherited` | Select a dimension value; no new artwork or gameplay is inferred. |
| `ornament name \| {slot: name}` | `inherited` | One ornament or one per button/panel/plate/slot/chip. |
| `effects dimension name` | `inherited` | Select a dimension value; no new artwork or gameplay is inferred. |
| `sound dimension name` | `inherited` | Select a dimension value; no new artwork or gameplay is inferred. |
| `icons dimension name` | `inherited` | Select a dimension value; no new artwork or gameplay is inferred. |
| `density dimension name` | `inherited` | Select a dimension value; no new artwork or gameplay is inferred. |
| `pack pack object` | `inherited` | Hand-authored pack overlay applied after compiled theme and generated art; see file:pack. |

<a id="file-theme-color"></a>
### `file:theme.color`

| Field | Default | Meaning |
|---|---|---|
| `source string` | `inherited` | Palette input; source/accent/neutral/danger are colours, variant selects palette derivation, scheme is light or dark. |
| `accent string` | `inherited` | Palette input; source/accent/neutral/danger are colours, variant selects palette derivation, scheme is light or dark. |
| `neutral string` | `inherited` | Palette input; source/accent/neutral/danger are colours, variant selects palette derivation, scheme is light or dark. |
| `danger string` | `inherited` | Palette input; source/accent/neutral/danger are colours, variant selects palette derivation, scheme is light or dark. |
| `variant string` | `inherited` | Palette input; source/accent/neutral/danger are colours, variant selects palette derivation, scheme is light or dark. |
| `scheme string` | `inherited` | Palette input; source/accent/neutral/danger are colours, variant selects palette derivation, scheme is light or dark. |
<a id="file-pack"></a>
### `file:pack`

pack.json: UI style pack. See themes.

| Field | Default | Meaning |
|---|---|---|
| `about any JSON` | `` | Ignored author note. |
| `scheme light \| dark` | `inherited` | Default colour scheme. |
| `seeds {name: CSS colour or {light,dark}}` | `inherited` | Palette seeds; accepted names are listed below. |
| `theme {section: values}` | `inherited` | Tailwind theme overrides. Leaves may be exact {light,dark} pairs. |
| `contrast {section: values}` | `inherited` | Theme overrides for prefers-contrast:more. |
| `variants {Component.slot[.variant]: class string}` | `{}` | Append classes to named component slots; empty strings declare slots. |
| `fonts [path, ...]` | `[]` | Font files relative to this pack, or shared asset paths. Font metadata comes from the files. |
| `icons {name: path}` | `{}` | Named image paths. |
| `iconSet path` | `none` | Icons asset (.ogt), below individually named icons. |
| `images {kebab-name: nineSlice or {light,dark}}` | `{}` | Named nine-slice art. |
| `sounds {event: sound, Component.slot[.variant]: {event: sound}}` | `{}` | Pack-wide or slot-local interaction sounds. |
| `haptics {event: preset, Component.slot[.variant]: {event: preset}}` | `{}` | Pack-wide or slot-local haptic names. |

<a id="file-pack-images-nineSlice"></a>
### `file:pack.images.nineSlice`

| Field | Default | Meaning |
|---|---|---|
| `src path` | `required` | Source image. |
| `slice number >= 0 \| [top,right,bottom,left]` | `required` | Insets into the source image, in pixels. |
| `width number >= 0 \| [top,right,bottom,left]` | `slice` | Drawn border widths. |
| `fill boolean` | `true` | Paint the centre. |
| `repeat stretch \| tile` | `stretch` | Border repeat mode. |
| `tint boolean` | `false` | Tint art with the element colour. |

<a id="file-pack-sounds-sound"></a>
### `file:pack.sounds.sound`

| Field | Default | Meaning |
|---|---|---|
| `clips [path, ...]` | `[]` | Audio clips relative to this pack or shared assets; empty only at volume 0. |
| `volume number \| [low,high]` | `1` | Gain or sampling range. |
| `pitch number \| [low,high]` | `1` | Playback rate or sampling range. |

Theme sections: colors, spacing, rem, breakpoints, container, fontSize, fontWeight, fontFamily, tracking, leading, radius, shadow, insetShadow, dropShadow, textShadow, blur, perspective, aspect, ease, duration, opacity, stroke, scale. Palette seeds: primary, accent, neutral, danger, warn, success, info, team, hud. Sound/haptic events: press, hover, select, confirm, cancel, back, open, close, error, reward, toggle, tab, levelup, purchase, countdown, tick, step, swipe, notify, unlock. Haptic presets: tick, tap, thud, double, buzz, success, warning, error, none. Scheme pairs have exactly light and dark. Theme values follow the Tailwind theme contract, not OGT syntax.

<a id="file-font-extension"></a>
### `file:font-extension`

extension.json of kind font, with TTF/OTF sources. See localization.

| Field | Default | Meaning |
|---|---|---|
| `kind font` | `required` | Extension kind discriminator. |
| `name string` | `required` | Pack name. |
| `languages [language, ...]` | `[]` | Language tags the pack covers. |
| `scripts [ISO 15924 code, ...]` | `[]` | Scripts, including language-independent packs such as emoji Zsye. |
| `faces [path, ...]` | `[]` | TTF/OTF paths relative to the pack; each face retains its own import settings. |
| `hyphenation [language.hyph, ...]` | `[]` | Dictionaries shipped only for reached hyphens:auto and project locales. |
<a id="file-tsx"></a>
### `file:tsx`

TSX is the UI source; TS/TSX bundling is not a cook generator. See ui-tsx.

<a id="file-probe-record"></a>
### `file:probe-record`

cooked/probes/<key>.ogt: derived bake record, not an authored input. See render.

| Field | Default | Meaning |
|---|---|---|
| `position "<x>" "<y>" "<z>"` | `0 0 0` | World position in metres, stored as quoted numeric strings. |
| `box "<x>" "<y>" "<z>"` | `0 0 0` | World box size in metres, stored as quoted numeric strings. |
| `room "<hash>"` | `` | Captured room content hash. |
| `commands "<commands>"` | `` | Commands which reached the capture. |
| `asset "<id>" "<hex hash>"` | `` | Repeated captured model output hashes. |
| `code "<hex hash>"` | `none` | Source hash when code built the captured room. |
Removed style.ogt and .svelte are rejected, not compatibility formats. For native source extensions, use the loading guide; a decoder supported as a parent asset's input is not necessarily an independent cook type.
