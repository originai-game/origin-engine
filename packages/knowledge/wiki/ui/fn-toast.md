<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-toast"></a>

### `og.ui.toast`

Queue a toast: a row of the UI's $toasts store ({ id, text, tone, leaving }) for its time, then leaving for a fast duration; at most three are up (older ones leave first). It draws nothing itself: the built-in notifications screen shows $toasts after its own items, so load and push it once (og.load.document(world, "@ui/screens/notifications.svelte"), then og.ui.push(world, "notifications", { items: [] })), and a press on one dismisses it. Callable anywhere (systems, commands, start); it is world state, so snapshots carry it.

Also known as: snackbar, transient notification, popup message, auto-dismissing message

```ts
og.ui.toast(world: World, text: string, desc?: ToastDesc): number
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world whose UI shows it. |
| `text` | `string` | no | The message: a few words. |
| `desc` | `ToastDesc` | yes | Tone and time. |

**Returns** `number`: Its id, for dismissToast.

Binding symbol `og_ui_toast` · publish builds: kept

```ogt
fn toast {
  doc "Queue a toast: a row of the UI's $toasts store ({ id, text, tone, leaving }) for its time, then leaving for a fast duration; at most three are up (older ones leave first). It draws nothing itself: the built-in notifications screen shows $toasts after its own items, so load and push it once (og.load.document(world, \"@ui/screens/notifications.svelte\"), then og.ui.push(world, \"notifications\", { items: [] })), and a press on one dismisses it. Callable anywhere (systems, commands, start); it is world state, so snapshots carry it."
  aliases "snackbar, transient notification, popup message, auto-dismissing message"
  param world World "The world whose UI shows it."
  param text String "The message: a few words."
  param desc ToastDesc "Tone and time."
  returns u32 "Its id, for dismissToast."
}
```

<sub>Module `ui` line 545</sub>
