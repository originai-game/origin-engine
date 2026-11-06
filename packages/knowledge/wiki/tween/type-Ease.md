<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`tween`](../tween.md) · [index](../index.md)

<a id="type-Ease"></a>

### `Ease` (enum)

An easing curve from the start (0) to the end (1). In starts slow, out ends slow, inOut both. back overshoots, elastic rings, bounce bounces at the end.

A string: one of the variants below. The zero value is `"quadOut"`.

| Variant | Doc |
|---|---|
| `"quadOut"` | Gentle ease out: the default for most motion. |
| `"linear"` | Constant speed: scrolling, timers, colour cycles. |
| `"quadIn"` | Gentle ease in. |
| `"quadInOut"` | Gentle in and out. |
| `"cubicIn"` | Ease in. |
| `"cubicOut"` | Ease out: UI sliding in, things settling. |
| `"cubicInOut"` | In and out: camera moves, deliberate motion. |
| `"quartIn"` | Strong ease in. |
| `"quartOut"` | Strong ease out. |
| `"quartInOut"` | Strong in and out. |
| `"expoIn"` | Exponential ease in: things falling away, warp-outs. |
| `"expoOut"` | Exponential ease out: snappy arrivals. |
| `"expoInOut"` | Exponential in and out. |
| `"backIn"` | Pulls back first, then goes: anticipation. |
| `"backOut"` | Overshoots, then settles: pops, squash recovery, buttons. |
| `"backInOut"` | Pulls back and overshoots. |
| `"elasticIn"` | Rings before leaving. |
| `"elasticOut"` | Rings after arriving: springy, cartoon impacts. |
| `"elasticInOut"` | Rings at both ends. |
| `"bounceIn"` | Bounces before leaving. |
| `"bounceOut"` | Bounces at the end: drops, landings. |
| `"bounceInOut"` | Bounces at both ends. |

```ogt
enum Ease {
  doc "An easing curve from the start (0) to the end (1). In starts slow, out ends slow, inOut both. back overshoots, elastic rings, bounce bounces at the end."
  variant quadOut "Gentle ease out: the default for most motion."
  variant linear "Constant speed: scrolling, timers, colour cycles."
  variant quadIn "Gentle ease in."
  variant quadInOut "Gentle in and out."
  variant cubicIn "Ease in."
  variant cubicOut "Ease out: UI sliding in, things settling."
  variant cubicInOut "In and out: camera moves, deliberate motion."
  variant quartIn "Strong ease in."
  variant quartOut "Strong ease out."
  variant quartInOut "Strong in and out."
  variant expoIn "Exponential ease in: things falling away, warp-outs."
  variant expoOut "Exponential ease out: snappy arrivals."
  variant expoInOut "Exponential in and out."
  variant backIn "Pulls back first, then goes: anticipation."
  variant backOut "Overshoots, then settles: pops, squash recovery, buttons."
  variant backInOut "Pulls back and overshoots."
  variant elasticIn "Rings before leaving."
  variant elasticOut "Rings after arriving: springy, cartoon impacts."
  variant elasticInOut "Rings at both ends."
  variant bounceIn "Bounces before leaving."
  variant bounceOut "Bounces at the end: drops, landings."
  variant bounceInOut "Bounces at both ends."
}
```

<sub>`api.ogt` line 12374</sub>
