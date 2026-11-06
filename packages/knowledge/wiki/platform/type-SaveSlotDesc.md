<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`platform`](../platform.md) · [index](../index.md)

<a id="type-SaveSlotDesc"></a>

### `SaveSlotDesc` (desc struct)

A save slot: a long text (a world.snapshot()) kept across as many store keys as it needs.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `slot` | `string` | `""` | — | — | — | The slot's name: 1 to 60 of letters, digits, `.`, `_` and `-` (`slot1`). Its keys are the name itself (the manifest) and `<slot>.a0`, `<slot>.a1`, ... or `<slot>.b0`, ... (the parts). |
| `text` | `string` | `""` | — | — | — | The text: at most 512 KiB of UTF-8 (eight parts of 64 KiB). |
| `meta` | `unknown` | `null` | — | — | — | What a title screen shows of the slot (`{ place, hour }`), kept in the manifest so loadSlot with metaOnly reads it without the parts: any JSON, at most 4 KiB encoded. |

```ogt
struct SaveSlotDesc {
  doc "A save slot: a long text (a world.snapshot()) kept across as many store keys as it needs."
  desc true
  field slot String "The slot's name: 1 to 60 of letters, digits, `.`, `_` and `-` (`slot1`). Its keys are the name itself (the manifest) and `<slot>.a0`, `<slot>.a1`, ... or `<slot>.b0`, ... (the parts)."
  field text String "The text: at most 512 KiB of UTF-8 (eight parts of 64 KiB)."
  field meta Json "What a title screen shows of the slot (`{ place, hour }`), kept in the manifest so loadSlot with metaOnly reads it without the parts: any JSON, at most 4 KiB encoded."
}
```

<sub>`api.ogt` line 14900</sub>
