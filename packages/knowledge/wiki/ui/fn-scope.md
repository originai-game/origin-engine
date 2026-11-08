<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-scope"></a>

### `og.ui.scope`

The entry to named svelte/store-compatible presentation data and retained document mounts. Rust owns values, the dependency graph and mounted elements. No Svelte compiler, runes, TS elements or asynchronous primitives. See docs/ui.md for a worked example and If you know Svelte.

```ts
og.ui.scope(world: World, id: string): UiScope
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world that owns the presentation checkpoint. |
| `id` | `string` | no | An explicit, nonempty stable scope name. |

**Returns** `UiScope`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn scope {
  host false
  doc "The entry to named svelte/store-compatible presentation data and retained document mounts. Rust owns values, the dependency graph and mounted elements. No Svelte compiler, runes, TS elements or asynchronous primitives. See docs/ui.md for a worked example and If you know Svelte."
  param world World "The world that owns the presentation checkpoint."
  param id String "An explicit, nonempty stable scope name."
  returns UiScope
}
```

<sub>Module `ui` line 3792</sub>
