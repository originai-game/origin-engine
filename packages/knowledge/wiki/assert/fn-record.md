<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`assert`](../assert.md) · [index](../index.md)

<a id="fn-record"></a>

### `og.assert.record`

The one call the members above make into the engine: a site's first call, a violation or a first firing. Game code calls those, not this. With null, it only answers the report's epoch.

```ts
og.assert.record(world: World, record: AssertRecord | null): number
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world the assertion was called in. |
| `record` | `AssertRecord \| null` | no | The call, or null. |

**Returns** `number`: The report's epoch: it moves on each time the probe's assertions member starts a new report.

Binding symbol `og_assert_record` · publish builds: dropped

```ogt
fn record {
  doc "The one call the members above make into the engine: a site's first call, a violation or a first firing. Game code calls those, not this. With null, it only answers the report's epoch."
  param world World "The world the assertion was called in."
  param record Option.AssertRecord "The call, or null."
  returns u32 "The report's epoch: it moves on each time the probe's assertions member starts a new report."
}
```

<sub>Module `assert` line 215</sub>
