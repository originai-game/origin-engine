<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `tween` module

Tweens: animate any numeric field of a component (og.Transform.position, og.Sprite.color, og.Sprite.opacity, a game's own f32) from its value to a target over time with an easing curve, a delay, loops and yoyo; chain them in sequences or run them in parallel. Advanced by the engine's og.tween system after the game's systems and before og.anim and physics, by scaled time (slow motion, hit stop) unless unscaled.

Running tweens are world state (the og.tween state block): snapshots, restore and the world hash cover them. og.TweenCompleted reports a tween or group that ends on its own; a cancelled one, or one whose entity or component went away, ends silently.

Feel presets are tweens too: og.tween.squash (squash and stretch that recovers) and og.tween.flash (a hit flash that fades).

- **TypeScript**: functions are the namespace `og.tween`; components, resources and events are tokens on `og`.
- **Publish builds**: kept (members may override).
- **Index**: [every module and guide](index.md).

```ogt
module tween {
  doc "Tweens: animate any numeric field of a component (og.Transform.position, og.Sprite.color, og.Sprite.opacity, a game's own f32) from its value to a target over time with an easing curve, a delay, loops and yoyo; chain them in sequences or run them in parallel. Advanced by the engine's og.tween system after the game's systems and before og.anim and physics, by scaled time (slow motion, hit stop) unless unscaled."
  doc "Running tweens are world state (the og.tween state block): snapshots, restore and the world hash cover them. og.TweenCompleted reports a tween or group that ends on its own; a cancelled one, or one whose entity or component went away, ends silently."
  doc "Feel presets are tweens too: og.tween.squash (squash and stretch that recovers) and og.tween.flash (a hit flash that fades)."
  # declarations follow, each on its own page
```

<sub>Module `tween` line 1</sub>

## Contents

| Name | Kind | Signature | Summary |
|---|---|---|---|
| [`Tween`](tween/type-Tween.md) | handle |  | A running tween or group. |
| [`Tween()`](#column-Tween) | column kind |  | A Tween handle field. |
| [`Ease`](tween/type-Ease.md) | enum |  | An easing curve from the start (0) to the end (1). |
| [`Ease()`](#column-Ease) | column kind |  | An Ease field: the variant's index. |
| [`TweenDesc`](tween/type-TweenDesc.md) | desc struct |  | One tween of one field. |
| [`TweenGroupDesc`](tween/type-TweenGroupDesc.md) | desc struct |  | How a sequence or parallel group runs. |
| [`og.TweenCompleted`](tween/record-TweenCompleted.md) | event |  | A tween or group ended on its own (read in the tick after). |
| [`SquashDesc`](tween/type-SquashDesc.md) | desc struct |  | Squash and stretch: the entity's og.Transform.scale jumps to rest times impact and eases back to rest. |
| [`FlashDesc`](tween/type-FlashDesc.md) | desc struct |  | A hit flash: the entity's og.Sprite (else og.Mesh, else og.Model) flashColor set and its flash faded from amount to 0. |
| [`og.tween.to`](tween/fn-to.md) | function | `(world: World, desc?: TweenDesc): Result<Tween>` | Start a tween. Systems may call it; it first moves in this tick's og.tween run. |
| [`og.tween.sequence`](tween/fn-sequence.md) | function | `(world: World, steps: readonly TweenDesc[], desc?: TweenGroupDesc): Result<Tween>` | Start tweens one after another (each with its own delay, duration and loops); the time one leaves over goes to the next. |
| [`og.tween.parallel`](tween/fn-parallel.md) | function | `(world: World, tweens: readonly TweenDesc[], desc?: TweenGroupDesc): Result<Tween>` | Start tweens together; the group ends with the last one. |
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
| <a id="column-Ease"></a>`Ease` | `Ease` | `Uint32Array` | 1 | An Ease field: the variant's index. |

```ogt
column Tween {
  doc "A Tween handle field."
  array Uint32Array
  stride 2
  value Tween
}
```

<sub>Module `tween` line 8</sub>

```ogt
column Ease {
  doc "An Ease field: the variant's index."
  array Uint32Array
  stride 1
  value Ease
}
```

<sub>Module `tween` line 41</sub>
