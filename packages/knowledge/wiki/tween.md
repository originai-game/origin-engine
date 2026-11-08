<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `tween` module

Tweens: animate any numeric field of a component (og.Transform.position, og.Sprite.color, og.Sprite.opacity, a game's own f32) from its value to a target over time with an easing curve, a delay, repeat and yoyo; chain them in sequences or run them in parallel. Advanced by the engine's og.tween system after the game's systems and before og.anim and physics, by scaled time (slow motion, hit stop) unless unscaled.

Running tweens are world state (the og.tween state block): snapshots, restore and the world hash cover them. og.TweenCompleted reports a tween or group that ends on its own; a cancelled one, or one whose entity or component went away, ends silently.

Feel presets are tweens too: og.tween.squash (squash and stretch that recovers) and og.tween.flash (a hit flash that fades).

Timelines are GSAP's: `const tl = og.tween.timeline(world, { defaults: { duration: 0.4, ease: "back.out" }, repeat: 1, yoyo: true })`, then `tl.to(enemies, { field: "position.y", to: 2, stagger: { each: 0.05, from: "center" } }).to(door, { field: "position.x", to: 3 }, "<0.1").call("door.open").addLabel("out", "+=0.5")`. Targets are entities and one numeric field per call (`field` and `to`, as og.tween.to), not JavaScript objects; vectors take one value per lane. Position is GSAP's: seconds, `"<"` (the start of what was added last), `">"` (its end), `"+=1"` and `"-=0.5"` (from the timeline's end), a label, `"label+=0.2"`, `"<50%"` (half way through what was added last) and `"-=25%"` (a quarter of what is inserted); none appends at the end. A label a position names that does not exist is added at the end.

A timeline renders from its playhead, never accumulated: play, pause, reverse, seek, progress and timeScale move it, and what they change is written in the next og.tween run (this tick, from a system before it). A child is written only when its own time changes, so a finished tween leaves its field to the game. from and fromTo write their start values on the timeline's first render, before their time (GSAP's immediateRender). A timeline stays when it completes (og.TweenCompleted is sent), so it can be reversed, restarted or extended, until kill() or `autoKill`. A refused call (a dead entity, a field that is not numeric, bad position text) adds nothing and its reason is the timeline's `error`; the call still answers the timeline, so chains read as GSAP's.

Callbacks are names, not closures: `onStart`, `onUpdate`, `onComplete` and `onRepeat` on a tween or the timeline, and `.call(name)`, send og.TweenEvent with that name, read by a system in the tick after. A seek or progress jump sends none. Timelines are world state: snapshots and restore cover the whole timeline, its playhead, labels and captured values, and a `random` stagger shuffles from the world's og.Rng seed (a fork: the main stream does not move; a system needs og.Rng among its reads).

- **TypeScript**: functions are the namespace `og.tween`; components, resources and events are tokens on `og`.
- **Publish builds**: kept (members may override).
- **Index**: [every module and guide](index.md).

```ogt
module tween {
  doc "Tweens: animate any numeric field of a component (og.Transform.position, og.Sprite.color, og.Sprite.opacity, a game's own f32) from its value to a target over time with an easing curve, a delay, repeat and yoyo; chain them in sequences or run them in parallel. Advanced by the engine's og.tween system after the game's systems and before og.anim and physics, by scaled time (slow motion, hit stop) unless unscaled."
  doc "Running tweens are world state (the og.tween state block): snapshots, restore and the world hash cover them. og.TweenCompleted reports a tween or group that ends on its own; a cancelled one, or one whose entity or component went away, ends silently."
  doc "Feel presets are tweens too: og.tween.squash (squash and stretch that recovers) and og.tween.flash (a hit flash that fades)."
  doc "Timelines are GSAP's: `const tl = og.tween.timeline(world, { defaults: { duration: 0.4, ease: \"back.out\" }, repeat: 1, yoyo: true })`, then `tl.to(enemies, { field: \"position.y\", to: 2, stagger: { each: 0.05, from: \"center\" } }).to(door, { field: \"position.x\", to: 3 }, \"<0.1\").call(\"door.open\").addLabel(\"out\", \"+=0.5\")`. Targets are entities and one numeric field per call (`field` and `to`, as og.tween.to), not JavaScript objects; vectors take one value per lane. Position is GSAP's: seconds, `\"<\"` (the start of what was added last), `\">\"` (its end), `\"+=1\"` and `\"-=0.5\"` (from the timeline's end), a label, `\"label+=0.2\"`, `\"<50%\"` (half way through what was added last) and `\"-=25%\"` (a quarter of what is inserted); none appends at the end. A label a position names that does not exist is added at the end."
  doc "A timeline renders from its playhead, never accumulated: play, pause, reverse, seek, progress and timeScale move it, and what they change is written in the next og.tween run (this tick, from a system before it). A child is written only when its own time changes, so a finished tween leaves its field to the game. from and fromTo write their start values on the timeline's first render, before their time (GSAP's immediateRender). A timeline stays when it completes (og.TweenCompleted is sent), so it can be reversed, restarted or extended, until kill() or `autoKill`. A refused call (a dead entity, a field that is not numeric, bad position text) adds nothing and its reason is the timeline's `error`; the call still answers the timeline, so chains read as GSAP's."
  doc "Callbacks are names, not closures: `onStart`, `onUpdate`, `onComplete` and `onRepeat` on a tween or the timeline, and `.call(name)`, send og.TweenEvent with that name, read by a system in the tick after. A seek or progress jump sends none. Timelines are world state: snapshots and restore cover the whole timeline, its playhead, labels and captured values, and a `random` stagger shuffles from the world's og.Rng seed (a fork: the main stream does not move; a system needs og.Rng among its reads)."
  # declarations follow, each on its own page
```

<sub>Module `tween` line 1</sub>

## Contents

| Name | Kind | Signature | Summary |
|---|---|---|---|
| [`Tween`](tween/type-Tween.md) | handle |  | A running tween or group. |
| [`Tween()`](#column-Tween) | column kind |  | A Tween handle field. |
| [`Ease`](tween/type-Ease.md) | enum |  | An easing curve from the start (0) to the end (1). |
| [`Ease()`](#column-Ease) | column kind |  | A shared timing value: a tag and four payload words (20 bytes), hashed and snapshotted with its record. |
| [`TweenDesc`](tween/type-TweenDesc.md) | desc struct |  | One tween of one field. |
| [`TweenFromDesc`](tween/type-TweenFromDesc.md) | desc struct |  | Tween from an explicit value to the field captured when og.tween.from is called, before any delay. |
| [`TweenGroupDesc`](tween/type-TweenGroupDesc.md) | desc struct |  | How a sequence or parallel group runs. |
| [`og.TweenCompleted`](tween/record-TweenCompleted.md) | event |  | A tween or group ended on its own (read in the tick after). |
| [`og.TweenEvent`](tween/record-TweenEvent.md) | event |  | A timeline callback: a call crossed, or a tween or timeline started, updated, completed or repeated (read in the tick after). |
| [`TweenDefaults`](tween/type-TweenDefaults.md) | desc struct |  | What a timeline's tweens take when they leave it out (GSAP's `defaults`). |
| [`TimelineDesc`](tween/type-TimelineDesc.md) | desc struct |  | How a timeline plays (og.tween.timeline). |
| [`StaggerDesc`](tween/type-StaggerDesc.md) | desc struct |  | GSAP's stagger object: each target's start offset by its distance from `from`. |
| [`TweenKeyframe`](tween/type-TweenKeyframe.md) | desc struct |  | One step of a tween's keyframes: from where the step before ended to `to`. |
| [`TimelineTweenVars`](tween/type-TimelineTweenVars.md) | desc struct |  | One timeline tween of one field of every target (TweenTimeline.to, from, fromTo and set): GSAP's vars, with the field named. |
| [`TweenTimeline`](tween/type-TweenTimeline.md) | interface |  | A GSAP timeline (og.tween.timeline): build it with to, from, fromTo, set, call, addLabel and add, each placed by a position and answering the timeline so calls chain; control it with play, pause, rev… |
| [`SquashDesc`](tween/type-SquashDesc.md) | desc struct |  | Squash and stretch: the entity's og.Transform.scale jumps to rest times impact and eases back to rest. |
| [`FlashDesc`](tween/type-FlashDesc.md) | desc struct |  | A hit flash: the entity's og.Sprite (else og.Mesh, else og.Model) flashColor set and its flash faded from amount to 0. |
| [`og.tween.to`](tween/fn-to.md) | function | `(world: World, desc?: TweenDesc): Result<Tween>` | Start a tween. Systems may call it; it first moves in this tick's og.tween run. |
| [`og.tween.from`](tween/fn-from.md) | function | `(world: World, desc?: TweenFromDesc): Result<Tween>` | Animate from desc.from to the field's value captured at this call, before the delay. |
| [`og.tween.fromTo`](tween/fn-fromTo.md) | function | `(world: World, desc?: TweenDesc): Result<Tween>` | Animate from explicit desc.from to desc.to; both are required. |
| [`og.tween.sequence`](tween/fn-sequence.md) | function | `(world: World, steps: readonly TweenDesc[], desc?: TweenGroupDesc): Result<Tween>` | Start tweens one after another (each with its own delay, duration and repeat); the time one leaves over goes to the next. |
| [`og.tween.parallel`](tween/fn-parallel.md) | function | `(world: World, tweens: readonly TweenDesc[], desc?: TweenGroupDesc): Result<Tween>` | Start tweens together; the group ends with the last one. |
| [`og.tween.timeline`](tween/fn-timeline.md) | function | `(world: World, desc?: TimelineDesc): TweenTimeline` | A GSAP timeline: tweens, calls, labels and nested timelines placed on its own clock, played, paused, reversed and sought as one. |
| [`og.tween.cancel`](tween/fn-cancel.md) | function | `(world: World, tween: Tween): boolean` | Stop a tween where it is (no event). |
| [`og.tween.cancelEntity`](tween/fn-cancelEntity.md) | function | `(world: World, entity: Entity): number` | Stop every tween of an entity (no events), say before starting a new reaction. |
| [`og.tween.active`](tween/fn-active.md) | function | `(world: World, tween: Tween): boolean` | Whether a tween is still running. |
| [`og.tween.squash`](tween/fn-squash.md) | function | `(world: World, desc?: SquashDesc): Result<Tween>` | Squash and stretch the entity's og.Transform.scale (cancels its earlier squash). |
| [`og.tween.flash`](tween/fn-flash.md) | function | `(world: World, desc?: FlashDesc): Result<Tween>` | Flash the entity's og.Sprite, its og.Mesh, or its og.Model (a glTF instance: this instance only, the others sharing its materials stay as they are). |

## Column kinds

Each is a factory, `Tween(value?, options?)`, that makes one field of a component or resource layout.

| Column | Value | Typed array | Stride | Doc |
|---|---|---|---|---|
| <a id="column-Tween"></a>`Tween` | `Tween` | `Uint32Array` | 2 | A Tween handle field. |
| <a id="column-Ease"></a>`Ease` | `Ease` | `Uint32Array` | 5 | A shared timing value: a tag and four payload words (20 bytes), hashed and snapshotted with its record. Accepts every named or parameterized Ease string. Raw views contain the canonical tag/payload, not string indices. |

```ogt
column Tween {
  doc "A Tween handle field."
  array Uint32Array
  stride 2
  value Tween
}
```

<sub>Module `tween` line 11</sub>

```ogt
column Ease {
  doc "A shared timing value: a tag and four payload words (20 bytes), hashed and snapshotted with its record. Accepts every named or parameterized Ease string. Raw views contain the canonical tag/payload, not string indices."
  array Uint32Array
  stride 5
  value Ease
}
```

<sub>Module `tween` line 46</sub>
