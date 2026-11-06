<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`story`](../story.md) · [index](../index.md)

<a id="fn-barks"></a>

### `og.story.barks`

The barks being said, by entity: draw each over its entity (a speech bubble) until it ends.

```ts
og.story.barks(world: World): readonly BarkView[]
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |

**Returns** `readonly BarkView[]`.

Binding symbol `og_story_barks` · publish builds: kept

```ogt
fn barks {
  doc "The barks being said, by entity: draw each over its entity (a speech bubble) until it ends."
  param world World "The world."
  returns List.BarkView
}
```

<sub>`api.ogt` line 14724</sub>
