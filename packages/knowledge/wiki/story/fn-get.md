<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`story`](../story.md) · [index](../index.md)

<a id="fn-get"></a>

### `og.story.get`

A story variable (a `global` of any loaded dialogue or quest): its value, else its declared default.

```ts
og.story.get(world: World, name: string): unknown
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `name` | `string` | no | The variable. |

**Returns** `unknown`: A number, a boolean, a string, or null for one nothing declares.

Binding symbol `og_story_get` · publish builds: kept

```ogt
fn get {
  doc "A story variable (a `global` of any loaded dialogue or quest): its value, else its declared default."
  param world World "The world."
  param name String "The variable."
  returns Json "A number, a boolean, a string, or null for one nothing declares."
}
```

<sub>`api.ogt` line 14730</sub>
