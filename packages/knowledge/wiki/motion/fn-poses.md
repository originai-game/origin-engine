<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`motion`](../motion.md) · [index](../index.md)

<a id="fn-poses"></a>

### `og.motion.poses`

The model's pose library, by name: idle, walk, run, jump, crouch, wave, cheer. Fixed by the engine's version, known before the weights are resident, so a goal's pose is checked the same everywhere.

```ts
og.motion.poses(): readonly string[]
```

**Returns** `readonly string[]`: The names.

Binding symbol `og_motion_poses` · publish builds: kept

```ogt
fn poses {
  doc "The model's pose library, by name: idle, walk, run, jump, crouch, wave, cheer. Fixed by the engine's version, known before the weights are resident, so a goal's pose is checked the same everywhere."
  returns List.String "The names."
}
```

<sub>Module `motion` line 346</sub>
