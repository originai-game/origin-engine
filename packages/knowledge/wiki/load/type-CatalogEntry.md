<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`load`](../load.md) · [index](../index.md)

<a id="type-CatalogEntry"></a>

### `CatalogEntry` (desc struct)

A file of a content catalog's version: what og.load.data reads. A catalog is this game's content, published by its host in immutable versions; its content base is the host's (og run --content, the page's), never game code's.

See `loading §14`.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `catalog` | `string` | `""` | — | — | — | The catalog, as the game names it (`world`): 1 to 64 of letters, digits, `.`, `_` and `-`. |
| `version` | `string` | `""` | — | — | — | The version, as the host named it: 1 to 128 of letters, digits, `.`, `_` and `-`. |
| `path` | `string` | `""` | — | — | — | The file within the version, as its manifest lists it (`index/small/3/-2.json`): `/`-separated, no empty, `.` or `..` part. |

```ogt
struct CatalogEntry {
  doc "A file of a content catalog's version: what og.load.data reads. A catalog is this game's content, published by its host in immutable versions; its content base is the host's (og run --content, the page's), never game code's."
  guide "loading §14"
  desc true
  field catalog String "The catalog, as the game names it (`world`): 1 to 64 of letters, digits, `.`, `_` and `-`."
  field version String "The version, as the host named it: 1 to 128 of letters, digits, `.`, `_` and `-`."
  field path String "The file within the version, as its manifest lists it (`index/small/3/-2.json`): `/`-separated, no empty, `.` or `..` part."
}
```

<sub>Module `load` line 554</sub>
