<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`load`](../load.md) · [index](../index.md)

<a id="fn-style"></a>

### `og.load.style`

Load a cooked UI style for UiDesc.style: the directory whose `pack.json` it reads. A game's style is `styles/<name>` (a `pack.json`, or a `theme.json` the cook compiles into one beside it); an engine theme is `@themes/<name>` (`minimal`, `tactical`, `tactical-arctic`, `fantasy`, `scifi`, `anime`, `cartoon`, `candy`, `ink`, `pixel`, `storybook`), in the bundle only when a look's `ui` line names it. A `style.ogt` path is refused. Loaded once per path.

See `loading §6`.

```ts
og.load.style(world: World, path: string): Result<Style>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world whose UI uses it. |
| `path` | `string` | no | The style directory, such as `styles/main` or `@themes/tactical`. |

**Returns** `Result<Style>`: Refused for a directory without a `pack.json` in the bundle (run og cook; an engine theme no look names), a `style.ogt` path, or a pack that fails to parse.

Binding symbol `og_load_style` · publish builds: kept

```ogt
fn style {
  doc "Load a cooked UI style for UiDesc.style: the directory whose `pack.json` it reads. A game's style is `styles/<name>` (a `pack.json`, or a `theme.json` the cook compiles into one beside it); an engine theme is `@themes/<name>` (`minimal`, `tactical`, `tactical-arctic`, `fantasy`, `scifi`, `anime`, `cartoon`, `candy`, `ink`, `pixel`, `storybook`), in the bundle only when a look's `ui` line names it. A `style.ogt` path is refused. Loaded once per path."
  guide "loading §6"
  param world World "The world whose UI uses it."
  param path String "The style directory, such as `styles/main` or `@themes/tactical`."
  returns Result.Style "Refused for a directory without a `pack.json` in the bundle (run og cook; an engine theme no look names), a `style.ogt` path, or a pack that fails to parse."
}
```

<sub>Module `load` line 422</sub>
