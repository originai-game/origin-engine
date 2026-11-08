<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-searchBox"></a>

### `og.ui.searchBox`

A search box with suggestions: a field with a lens, a clear button and, while it has the focus and something matches, a list of suggestions under it (recent searches while it is empty) whose matching part is bold, laid over what follows the field (it never pushes it down), or over what comes before it when below it would run past the screen's bottom. Down (or a pad's down) moves into the list, Enter or a press chooses; Escape (Back) closes the list and leaves the field focused, and typing or a press on the field opens it again. Typing stays in the field (the list takes no focus scope). Parts: `Search.frame[focused]`, `Search.list`, `Search.option[hover|focused]` (the query's match in bold). Role textbox with a listbox of options. Probe: `<id>/Frame/Field`, `<id>/List/Option#i`.

```ts
og.ui.searchBox(id: string, query: string, desc?: SearchBoxDesc): SearchResult
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `query` | `string` | no | The query from game state. |
| `desc` | `SearchBoxDesc` | yes | Its suggestions and look. |

**Returns** `SearchResult`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn searchBox {
  doc "A search box with suggestions: a field with a lens, a clear button and, while it has the focus and something matches, a list of suggestions under it (recent searches while it is empty) whose matching part is bold, laid over what follows the field (it never pushes it down), or over what comes before it when below it would run past the screen's bottom. Down (or a pad's down) moves into the list, Enter or a press chooses; Escape (Back) closes the list and leaves the field focused, and typing or a press on the field opens it again. Typing stays in the field (the list takes no focus scope). Parts: `Search.frame[focused]`, `Search.list`, `Search.option[hover|focused]` (the query's match in bold). Role textbox with a listbox of options. Probe: `<id>/Frame/Field`, `<id>/List/Option#i`."
  host false
  param id String "Its id."
  param query String "The query from game state."
  param desc SearchBoxDesc "Its suggestions and look."
  returns SearchResult
}
```

<sub>Module `ui` line 4045</sub>
