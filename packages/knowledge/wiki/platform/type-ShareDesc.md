<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`platform`](../platform.md) · [index](../index.md)

<a id="type-ShareDesc"></a>

### `ShareDesc` (desc struct)

What to share through the device's share sheet.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `text` | `string` | `""` | — | — | — | The text, 1 to 280 characters. |
| `title` | `string` | `""` | — | — | — | A title, at most 80 characters; empty: the game's name. |
| `path` | `string` | `""` | — | — | — | A short query the game's own Play link carries back (`?share=<path>`), at most 256 of letters, digits, `.`, `_`, `-`, `/` and `=`; empty: the plain Play link. The host builds the link: a game names no URL. |
| `image` | `boolean` | `false` | — | — | — | Attach the frame drawn last (a score screen). |

```ogt
struct ShareDesc {
  doc "What to share through the device's share sheet."
  desc true
  field text String "The text, 1 to 280 characters."
  field title String "A title, at most 80 characters; empty: the game's name."
  field path String "A short query the game's own Play link carries back (`?share=<path>`), at most 256 of letters, digits, `.`, `_`, `-`, `/` and `=`; empty: the plain Play link. The host builds the link: a game names no URL."
  field image bool "Attach the frame drawn last (a score screen)."
}
```

<sub>`api.ogt` line 15205</sub>
