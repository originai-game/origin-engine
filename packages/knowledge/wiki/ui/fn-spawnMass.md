<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-spawnMass"></a>

### `og.ui.spawnMass`

Start instances of a template on this tick: each plays its template's timeline over its lifetime and leaves after it (damage numbers, pickups). They live in the UI's state, so a snapshot or a replay carries them.

```ts
og.ui.spawnMass(template: number, instances: readonly MassInstance[]): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `template` | `number` | no | From defineMass. |
| `instances` | `readonly MassInstance[]` | no | The instances. |

**Returns** `void`.

Binding symbol `og_ui_spawn_mass` · publish builds: kept

```ogt
fn spawnMass {
  doc "Start instances of a template on this tick: each plays its template's timeline over its lifetime and leaves after it (damage numbers, pickups). They live in the UI's state, so a snapshot or a replay carries them."
  param template u32 "From defineMass."
  param instances List.MassInstance "The instances."
  returns Void
}
```

<sub>Module `ui` line 7437</sub>
