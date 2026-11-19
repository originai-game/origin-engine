<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`platform`](../platform.md) · [index](../index.md)

<a id="type-PlatformNoticeKind"></a>

### `PlatformNoticeKind` (enum)

What a notice is about.

A string: one of the variants below. The zero value is `"purchase"`.

| Variant | Doc |
|---|---|
| `"purchase"` | A purchase settled: a pending one completed or failed, or one was refunded. Data `{ purchase, product, quantity, state }`, state `completed`, `failed` or `refunded`. |
| `"entitlements"` | The player's entitlements changed without a call (bought on another device, a subscription expired): data `{ entitlements }`, the entitlements answer's shape. |
| `"invite"` | An invite to this game arrived: data `{ invite, from: { player, name }, payload, time }`. |
| `"catalog"` | A content catalog may have a new version: data `{ catalog, version, manifest }`, og.platform.catalogHead's answer without `published`. Best effort: a host may send it late, twice or never, so a game polls catalogHead and treats this as a reason to poll now. |
| `"host"` | A message from the page that frames the game, outside any platform service (a site steering a showcase, a launcher handing over a choice): data `{ name, ... }`, its meaning agreed between game and page. The engine gives it none; a game ignores a name it does not know. |

```ogt
enum PlatformNoticeKind {
  doc "What a notice is about."
  variant purchase "A purchase settled: a pending one completed or failed, or one was refunded. Data `{ purchase, product, quantity, state }`, state `completed`, `failed` or `refunded`."
  variant entitlements "The player's entitlements changed without a call (bought on another device, a subscription expired): data `{ entitlements }`, the entitlements answer's shape."
  variant invite "An invite to this game arrived: data `{ invite, from: { player, name }, payload, time }`."
  variant catalog "A content catalog may have a new version: data `{ catalog, version, manifest }`, og.platform.catalogHead's answer without `published`. Best effort: a host may send it late, twice or never, so a game polls catalogHead and treats this as a reason to poll now."
  variant host "A message from the page that frames the game, outside any platform service (a site steering a showcase, a launcher handing over a choice): data `{ name, ... }`, its meaning agreed between game and page. The engine gives it none; a game ignores a name it does not know."
}
```

<sub>Module `platform` line 235</sub>
