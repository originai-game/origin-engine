<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`platform`](../platform.md) · [index](../index.md)

<a id="type-PlatformInfo"></a>

### `PlatformInfo` (struct)

What answers platform calls here.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `host` | `string` | `""` | — | — | — | `player` (a Player frames the game), `local` (the native host's JSON store or the browser's own storage) or `none`. |
| `capabilities` | `readonly string[]` | `[]` | — | — | — | The methods it answers, by their wire names (`store.get`, `purchase.buy`, `share`, ...); og.platform.supports asks for one. See `saves §7`. |
| `pending` | `number` | `0` | — | — | — | Calls not answered yet. |
| `clock` | `string` | `""` | — | — | — | Whose clock og.platform.now and the answers' `time` read: `player` (the Player's answers carry its time), `browser` (the web host's own clock), `system` (the native host's), `fixed` (`og run --clock` and playtests: it starts where told and moves with the ticks) or `none`. See `saves §7`. |

```ogt
struct PlatformInfo {
  doc "What answers platform calls here."
  field host String "`player` (a Player frames the game), `local` (the native host's JSON store or the browser's own storage) or `none`."
  field capabilities List.String "The methods it answers, by their wire names (`store.get`, `purchase.buy`, `share`, ...); og.platform.supports asks for one." {
    guide "saves §7"
  }
  field pending u32 "Calls not answered yet."
  field clock String "Whose clock og.platform.now and the answers' `time` read: `player` (the Player's answers carry its time), `browser` (the web host's own clock), `system` (the native host's), `fixed` (`og run --clock` and playtests: it starts where told and moves with the ticks) or `none`." {
    guide "saves §7"
  }
}
```

<sub>Module `platform` line 107</sub>
