<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-forceState"></a>

### `og.ui.forceState`

Force a state group of an element (the probe and agents render and check each state this way); null lets it go.

```ts
og.ui.forceState(path: string, group: string, state: string | null): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `path` | `string` | no | The element's full path. |
| `group` | `string` | no | The group. |
| `state` | `string \| null` | no | The state; null releases it. |

**Returns** `void`.

Binding symbol `og_ui_force_state` · publish builds: kept

```ogt
fn forceState {
  doc "Force a state group of an element (the probe and agents render and check each state this way); null lets it go."
  param path String "The element's full path."
  param group String "The group."
  param state Option.String "The state; null releases it."
  returns Void
}
```

<sub>Module `ui` line 2282</sub>
