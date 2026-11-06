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
| `"host"` | A message from the page that frames the game, outside any platform service (a site steering a showcase, a launcher handing over a choice): data `{ name, ... }`, its meaning agreed between the game and that page. The engine gives it no meaning of its own; a game reading it handles an unknown name by ignoring it. |

```ogt
enum PlatformNoticeKind {
  doc "What a notice is about."
  variant purchase "A purchase settled: a pending one completed or failed, or one was refunded. Data `{ purchase, product, quantity, state }`, state `completed`, `failed` or `refunded`."
  variant entitlements "The player's entitlements changed without a call (bought on another device, a subscription expired): data `{ entitlements }`, the entitlements answer's shape."
  variant invite "An invite to this game arrived: data `{ invite, from: { player, name }, payload, time }`."
  variant host "A message from the page that frames the game, outside any platform service (a site steering a showcase, a launcher handing over a choice): data `{ name, ... }`, its meaning agreed between the game and that page. The engine gives it no meaning of its own; a game reading it handles an unknown name by ignoring it."
}
```

<sub>`api.ogt` line 15129</sub>
