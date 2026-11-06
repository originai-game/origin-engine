<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-iconButton"></a>

### `og.ui.iconButton`

An icon button: a square (or round) button of the control height around one icon, named by its label (its tooltip, and what a screen reader says), with a badge or a dot. It moves and is dressed as pushButton, `Button.base[icon]`. Probe: `<id>`, `<id>/Icon`, `<id>/Badge`.

```ts
og.ui.iconButton(id: string, icon: string, desc?: IconButtonDesc): Response
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `icon` | `string` | no | The icon's name (the style's set; a drawn mark where it has none). |
| `desc` | `IconButtonDesc` | yes | Its name, role, size and badge. |

**Returns** `Response`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn iconButton {
  doc "An icon button: a square (or round) button of the control height around one icon, named by its label (its tooltip, and what a screen reader says), with a badge or a dot. It moves and is dressed as pushButton, `Button.base[icon]`. Probe: `<id>`, `<id>/Icon`, `<id>/Badge`."
  host false
  param id String "Its id."
  param icon String "The icon's name (the style's set; a drawn mark where it has none)."
  param desc IconButtonDesc "Its name, role, size and badge."
  returns Response
}
```

<sub>`api.ogt` line 19241</sub>
