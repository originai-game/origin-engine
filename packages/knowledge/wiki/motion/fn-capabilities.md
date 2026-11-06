<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`motion`](../motion.md) · [index](../index.md)

<a id="fn-capabilities"></a>

### `og.motion.capabilities`

Where the model runs: gpu once its weights are resident, none before (loading) or when they failed to load. Asking starts the load when no agent has yet.

```ts
og.motion.capabilities(world: World): MotionCapabilities
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |

**Returns** `MotionCapabilities`: Never refused.

Binding symbol `og_motion_capabilities` · publish builds: kept

```ogt
fn capabilities {
  doc "Where the model runs: gpu once its weights are resident, none before (loading) or when they failed to load. Asking starts the load when no agent has yet."
  param world World "The world."
  returns MotionCapabilities "Never refused."
}
```

<sub>`api.ogt` line 27065</sub>
