<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-emitter"></a>

### `og.ui.emitter`

An ambient particle emitter (docs/ui.md §5.5) laid over its parent's area: sparkles on a rare card, embers on a forge, snow over a winter menu, confetti on a win. It takes no input, is decoration to a screen reader, and stops under reduced motion. Built on og.ui.particles; every particle is the engine's, seeded and in the snapshot. Probe: `<id>`.

```ts
og.ui.emitter(id: string, desc?: EmitterDesc): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `desc` | `EmitterDesc` | yes | Its look and rate. |

**Returns** `void`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn emitter {
  doc "An ambient particle emitter (docs/ui.md §5.5) laid over its parent's area: sparkles on a rare card, embers on a forge, snow over a winter menu, confetti on a win. It takes no input, is decoration to a screen reader, and stops under reduced motion. Built on og.ui.particles; every particle is the engine's, seeded and in the snapshot. Probe: `<id>`."
  host false
  param id String "Its id."
  param desc EmitterDesc "Its look and rate."
  returns Void
}
```

<sub>`api.ogt` line 19263</sub>
