# og-tween

Tweens for the engine (`docs/design.md` §2.4, the E4 round): eased,
delayed, looping, yoyo, sequenced and parallel animation of any reflected
numeric component field, advanced by one scheduled system, `og.tween`,
whose state block carries every running tween through snapshots, restore
and the world hash. Game feel presets (squash and stretch, the hit flash)
are tweens too.

```rust
let tweens = og_tween::install(&mut world, &component_names)?;           // after the game's systems
let id = world.id_by_name("og.Transform")?;
let y = Target::resolve(world.schema(id)?, id.index(), e.to_bits(), "position.y")?;
let h = tweens.start(Node::Leaf(Leaf::new(y, vec![2.0]).duration(0.3).ease(Ease::BackOut)), false, 0)?;
let scale = Target::resolve(world.schema(id)?, id.index(), e.to_bits(), "scale")?;
tweens.start(presets::squash(scale, &Squash::default()), false, 0)?;    // impact scale, eased back to rest
world.step()?;                                                           // og.TweenCompleted { tween: h, .. } when it ends
```

Dependencies: og-core, og-ecs. No serde, builds for `wasm32-unknown-unknown`.

## API map

| Item | What it is |
|---|---|
| `install(&mut World, components) -> TweenSystem` | registers `og.TweenCompleted`, schedules `og.tween` here declaring a write to every listed component, adds the `og.tween` state block |
| `TweenSystem` | `start(node, unscaled, tag) -> handle` (never 0), `cancel(handle) -> bool`, `cancel_entity(entity) -> u32`, `is_active(handle)`, `count()` |
| `Target::resolve(schema, component, entity, path)` | `position`, `position.y`, `color.r`, `opacity`: f32, f64, i32, u32 (rounded), Vec2/3/4 (lanes `x y z w` or `r g b a`), Quat (whole, normalised lerp on the shorter arc). Bool, enum, handle, matrix and string fields are refused by name |
| `Leaf` | `to` (one value fills every lane), `from` (default: the field when the tween starts, after its delay), `relative`, `duration` (0.3 s), `delay`, `ease` (quadOut), `loops` (0 once, n more, -1 forever), `yoyo` |
| `Node::Leaf`, `Node::group(Sequence \| Parallel, children, loops)` | a sequence hands the time a child leaves over to the next; a parallel group ends with its last child; a looping group replays the spans its children resolved in the first run |
| `Ease`, `EASES`, `EASE_NAMES` | linear and quad, cubic, quart, expo, back, elastic, bounce × in, out, inOut (Penner's curves over og-core's deterministic `pow` and `sin`) |
| `presets::squash(scale_target, &Squash { impact, rest, duration, ease })` | the scale jumps to rest × impact (default 1.25 × 0.8) and eases back (backOut, 0.18 s) |
| `presets::flash(amount_target, colour_target, &Flash { color, amount, duration, ease })` | the colour set at once, the amount from `amount` to 0 (quadIn, 0.12 s) |
| `Spring { damping, response_ms }`, `step(SpringState { value, velocity }, target, dt)` | a damped spring by the tick, the design tokens' `motion.spring.*` pair (stiffness `(2π/response)²`, damping `2ζ·2π/response`), four semi-implicit Euler substeps of `+ − × ÷` only, snapped to rest within `1e-3` and `1e-2`/s: og-ui's hover lift and press squash move on it |
| `TweenCompleted { tween, entity, tag }` | sent when a started tween or group ends on its own; never for a cancelled one or one whose target went away |

## Invariants

- **Time**: a tween advances by the world's scaled tick (`og.TimeScale`:
  slow motion, hit stop) unless started `unscaled` (UI motion that must
  keep moving through a hit stop). Elapsed time is kept in f64 and a tween
  of n ticks ends on its n-th tick (a microsecond of tolerance absorbs the
  rounding of `dt`).
- **Order**: tweens advance in handle order, once per tick, where the
  system is scheduled (the runtime puts it after the game's systems and
  before og-anim and physics, so a tweened Transform is what physics and
  the frame see that tick). A tween started by a system before it moves in
  the same tick.
- **Targets**: a tween writes only its target's bytes (read and written
  through `SystemContext::get_bytes` / `set_bytes`), so other fields and
  other tweens of other fields of the same component are untouched; two
  tweens of the same field write in handle order (the later one wins).
- **State**: the state block holds every running tween (target, values,
  curve, elapsed, resolved start and end, group progress) in handle order,
  little-endian (`src/codec.rs`), so equal state gives equal bytes and a
  restore continues mid-tween.

## Tests

`tests/tweens.rs`: every ease's endpoints and shape; one lane moving
alone, the completion sent once; delay, relative, loops with yoyo ending at
the start, integer rounding, one value for a vector; a sequence's leftover
time, a parallel group waiting for its last child and replaying its spans
on loop; scale 0.5 and a hit stop holding scaled tweens while an unscaled
one runs; cancelled and orphaned tweens ending without events; a snapshot
mid-group restoring into a fresh world with the same hash 60 ticks later;
refused targets and values. Each was broken once
(`tests/evidence/E4/break-and-restore.md`).
