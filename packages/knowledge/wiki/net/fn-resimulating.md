<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`net`](../net.md) · [index](../index.md)

<a id="fn-resimulating"></a>

### `og.net.resimulating`

Whether the tick running is a rollback's re-simulation: the world was rewound and this tick runs again, so its presentation already happened once. The engine already plays each sound once and snaps the view; a game that triggers its own presentation from a system (a screen shake, a haptic) checks this first. False in lockstep and alone. Callable from a system.

```ts
og.net.resimulating(world: World): boolean
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |

**Returns** `boolean`.

Binding symbol `og_net_resimulating` · publish builds: kept

```ogt
fn resimulating {
  doc "Whether the tick running is a rollback's re-simulation: the world was rewound and this tick runs again, so its presentation already happened once. The engine already plays each sound once and snaps the view; a game that triggers its own presentation from a system (a screen shake, a haptic) checks this first. False in lockstep and alone. Callable from a system."
  param world World "The world."
  returns bool
}
```

<sub>`api.ogt` line 26364</sub>
