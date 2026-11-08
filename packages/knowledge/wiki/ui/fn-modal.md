<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-modal"></a>

### `og.ui.modal`

A dialog built by `build` while visible: scrim, centred panel, focus kept inside, Back and the backdrop dismiss it unless locked. Set your open flag false on `dismissed`.

Also known as: modal dialog, popup window, overlay window, Unity modal

```ts
og.ui.modal(id: string, open: boolean, desc: ModalDesc, build: UiBuild): ModalResult
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `open` | `boolean` | no | Whether it should be open. |
| `desc` | `ModalDesc` | no | Title, width and dismissal rules. |
| `build` | `UiBuild` | no | Makes its content. |

**Returns** `ModalResult`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn modal {
  doc "A dialog built by `build` while visible: scrim, centred panel, focus kept inside, Back and the backdrop dismiss it unless locked. Set your open flag false on `dismissed`."
  aliases "modal dialog, popup window, overlay window, Unity modal"
  host false
  param id String "Its id."
  param open bool "Whether it should be open."
  param desc ModalDesc "Title, width and dismissal rules."
  param build UiBuild "Makes its content."
  returns ModalResult
}
```

<sub>Module `ui` line 1315</sub>
