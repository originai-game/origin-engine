<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-defineAction"></a>

### `og.ui.defineAction`

Define (or redefine) a UI action. confirm (Enter, Space, A), back (Escape, B), tabNext (E, ], RB), tabPrev (Q, [, LB), details (X, Y) and rotate (R, X) are defined by default.

```ts
og.ui.defineAction(world: World, desc: UiActionDesc): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `desc` | `UiActionDesc` | no | The action. |

**Returns** `Result<void>`.

Binding symbol `og_ui_define_action` · publish builds: kept

```ogt
fn defineAction {
  doc "Define (or redefine) a UI action. confirm (Enter, Space, A), back (Escape, B), tabNext (E, ], RB), tabPrev (Q, [, LB), details (X, Y) and rotate (R, X) are defined by default."
  param world World "The world."
  param desc UiActionDesc "The action."
  returns Result.Void
}
```

<sub>Module `ui` line 8066</sub>
