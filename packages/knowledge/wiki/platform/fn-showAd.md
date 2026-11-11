<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`platform`](../platform.md) · [index](../index.md)

<a id="fn-showAd"></a>

### `og.platform.showAd`

Show an ad. While it shows, the host pauses the game through its lifecycle (no ticks, no audio: on the web the Player's pause and resume) and the answer arrives after it resumed, at a tick's start. Grant a reward only on `completed`.

Also known as: rewarded ad, rewarded video, interstitial, advertisement, commercial, monetisation

```ts
og.platform.showAd(world: World, desc?: AdDesc): Result<number>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `desc` | `AdDesc` | yes | The kind and placement. |

**Returns** `Result<number>`: Refused for a malformed placement. The answer's data is `{ kind, placement, outcome }`: outcome `completed` or `skipped` for a rewarded ad, `shown` for an interstitial. Errors: `no_fill` (no ad now; retryable), `rate_limited` (the platform's frequency cap), `unavailable`.

Binding symbol `og_platform_show_ad` · publish builds: kept

```ogt
fn showAd {
  doc "Show an ad. While it shows, the host pauses the game through its lifecycle (no ticks, no audio: on the web the Player's pause and resume) and the answer arrives after it resumed, at a tick's start. Grant a reward only on `completed`."
  aliases "rewarded ad, rewarded video, interstitial, advertisement, commercial, monetisation"
  param world World "The world."
  param desc AdDesc "The kind and placement."
  returns Result.u32 "Refused for a malformed placement. The answer's data is `{ kind, placement, outcome }`: outcome `completed` or `skipped` for a rewarded ad, `shown` for an interstitial. Errors: `no_fill` (no ad now; retryable), `rate_limited` (the platform's frequency cap), `unavailable`."
}
```

<sub>Module `platform` line 370</sub>
