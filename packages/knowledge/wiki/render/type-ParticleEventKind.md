<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="type-ParticleEventKind"></a>

### `ParticleEventKind` (enum)

What happened to a particle (og.ParticleEvent.kind; an emitter's `events` and `subEmitter` lines name them).

A string: one of the variants below. The zero value is `"birth"`.

| Variant | Doc |
|---|---|
| `"birth"` | It was spawned. |
| `"death"` | It reached the end of its lifetime (a particle the emitter's ring reuses before then sends none). |
| `"collision"` | It hit a collider faster than 0.5 m/s into it (one settling on the ground sends none): an og.Collider box or sphere in the tick (the emitter's `collision` line, world space). |

```ogt
enum ParticleEventKind {
  doc "What happened to a particle (og.ParticleEvent.kind; an emitter's `events` and `subEmitter` lines name them)."
  variant birth "It was spawned."
  variant death "It reached the end of its lifetime (a particle the emitter's ring reuses before then sends none)."
  variant collision "It hit a collider faster than 0.5 m/s into it (one settling on the ground sends none): an og.Collider box or sphere in the tick (the emitter's `collision` line, world space)."
}
```

<sub>Module `render` line 2162</sub>
