<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-nameplate"></a>

### `og.ui.nameplate`

A nameplate to the bar of World of Warcraft and Final Fantasy XIV: the name in its reaction's colour with a level badge and an icon, a title under it, a slim health bar with its absorb and a cast bar under that, an elite or boss frame, the target larger and outlined and kept on top; crowds stack instead of overlapping, it fades behind walls and sheds detail with distance (title, then name).

Parts: Nameplate.frame[<reaction>][targeted][<class>], Nameplate.name, Nameplate.title, Nameplate.level, Nameplate.bar, Nameplate.fill[<reaction>], Nameplate.shield, Nameplate.cast. States: group lod (full, compact, bar), group selection (none, targeted). Accessibility: text named name, value level and health. Probe ids: <id>, <id>/Name, <id>/Bar/Fill, <id>/Cast.

```ts
og.ui.nameplate(id: string, desc?: NameplateDesc): AnchorInfo
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `desc` | `NameplateDesc` | yes | The unit. |

**Returns** `AnchorInfo`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn nameplate {
  doc "A nameplate to the bar of World of Warcraft and Final Fantasy XIV: the name in its reaction's colour with a level badge and an icon, a title under it, a slim health bar with its absorb and a cast bar under that, an elite or boss frame, the target larger and outlined and kept on top; crowds stack instead of overlapping, it fades behind walls and sheds detail with distance (title, then name)."
  doc "Parts: Nameplate.frame[<reaction>][targeted][<class>], Nameplate.name, Nameplate.title, Nameplate.level, Nameplate.bar, Nameplate.fill[<reaction>], Nameplate.shield, Nameplate.cast. States: group lod (full, compact, bar), group selection (none, targeted). Accessibility: text named name, value level and health. Probe ids: <id>, <id>/Name, <id>/Bar/Fill, <id>/Cast."
  host false
  param id String "Its id."
  param desc NameplateDesc "The unit."
  returns AnchorInfo
}
```

<sub>Module `ui` line 6586</sub>
