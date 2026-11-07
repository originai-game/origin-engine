<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-particles"></a>

### `og.ui.particles`

Emit particles from the element built just before (a control, or a container just opened). Call every tick it emits; a burst throws on the ticks desc.fire is set. og.ui.burst is a preset of it.

```ts
og.ui.particles(desc?: ParticlesDesc): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `desc` | `ParticlesDesc` | yes | What it throws. |

**Returns** `void`.

Binding symbol `og_ui_particles` · publish builds: kept

```ogt
fn particles {
  doc "Emit particles from the element built just before (a control, or a container just opened). Call every tick it emits; a burst throws on the ticks desc.fire is set. og.ui.burst is a preset of it."
  param desc ParticlesDesc "What it throws."
  returns Void
}
```

<sub>Module `ui` line 7806</sub>
