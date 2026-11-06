<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-defineTimeline"></a>

### `og.ui.defineTimeline`

Define a timeline by name: configuration, like the theme, defined once at start. Plays and state groups name it.

```ts
og.ui.defineTimeline(name: string, timeline?: UiTimeline): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `name` | `string` | no | Its name. |
| `timeline` | `UiTimeline` | yes | Its tracks, events and labels. |

**Returns** `void`.

Binding symbol `og_ui_define_timeline` · publish builds: kept

```ogt
fn defineTimeline {
  doc "Define a timeline by name: configuration, like the theme, defined once at start. Plays and state groups name it."
  param name String "Its name."
  param timeline UiTimeline "Its tracks, events and labels."
  returns Void
}
```

<sub>`api.ogt` line 20946</sub>
