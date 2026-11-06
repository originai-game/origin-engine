<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-iconMark"></a>

### `og.ui.iconMark`

An icon of the style's set (og.ui.icon) in a tone, on a plate, with a badge or a dot: the foundation every icon in a control is. An icon the set lacks is drawn as a mark of the same name where the controls know one (tick, cross, plus, minus, search, chevrons, lock, star, warning, info, eye, reset, globe), else a dot: never a missing glyph. Parts: `Icon.plate[circle]`, `Icon.badge`, `Icon.dot`. Probe: `<id>/Glyph`, `<id>/Badge` (its text).

```ts
og.ui.iconMark(id: string, name: string, desc?: IconMarkDesc): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `name` | `string` | no | The icon's name in the style's set. |
| `desc` | `IconMarkDesc` | yes | Its size, tone, plate, badge and name. |

**Returns** `void`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn iconMark {
  doc "An icon of the style's set (og.ui.icon) in a tone, on a plate, with a badge or a dot: the foundation every icon in a control is. An icon the set lacks is drawn as a mark of the same name where the controls know one (tick, cross, plus, minus, search, chevrons, lock, star, warning, info, eye, reset, globe), else a dot: never a missing glyph. Parts: `Icon.plate[circle]`, `Icon.badge`, `Icon.dot`. Probe: `<id>/Glyph`, `<id>/Badge` (its text)."
  host false
  param id String "Its id."
  param name String "The icon's name in the style's set."
  param desc IconMarkDesc "Its size, tone, plate, badge and name."
  returns Void
}
```

<sub>`api.ogt` line 18911</sub>
