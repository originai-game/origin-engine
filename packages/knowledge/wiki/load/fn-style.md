<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`load`](../load.md) · [index](../index.md)

<a id="fn-style"></a>

### `og.load.style`

Load the exact canonical pack content ID for UiDesc.style. Export file.pack or file.theme from content/**/*.ts; both cook to native packs. Built-ins use explicit @themes/<name> or @styles/<name>, bundled when a look names them. No filename suffix or directory is inferred. Loaded once per ID.

```ts
og.load.style(world: World, path: string): Result<Style>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world whose UI uses it. |
| `path` | `string` | no | The exact canonical pack ID, such as base/styles/main or @themes/tactical. |

**Returns** `Result<Style>`: Refused if the exact ID is absent, is not a native pack, or its native fields fail validation. Use matching engine tools and hosts.

Binding symbol `og_load_style` · publish builds: kept

```ogt
fn style {
  doc "Load the exact canonical pack content ID for UiDesc.style. Export file.pack or file.theme from content/**/*.ts; both cook to native packs. Built-ins use explicit @themes/<name> or @styles/<name>, bundled when a look names them. No filename suffix or directory is inferred. Loaded once per ID."
  param world World "The world whose UI uses it."
  param path String "The exact canonical pack ID, such as base/styles/main or @themes/tactical."
  returns Result.Style "Refused if the exact ID is absent, is not a native pack, or its native fields fail validation. Use matching engine tools and hosts."
}
```

<sub>Module `load` line 417</sub>
