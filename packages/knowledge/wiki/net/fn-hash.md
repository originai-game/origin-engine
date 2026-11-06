<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`net`](../net.md) · [index](../index.md)

<a id="fn-hash"></a>

### `og.net.hash`

The world's shared hash now, hex: the hash lockstep compares, which leaves this peer's session out (og.Input, og.Lockstep, input and room events, the UI's focus; module docs). Two peers at the same tick agree on it exactly when their simulations agree; the world dump's `hash` covers everything and differs between peers.

```ts
og.net.hash(world: World): string
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |

**Returns** `string`.

Binding symbol `og_net_hash` · publish builds: kept

```ogt
fn hash {
  doc "The world's shared hash now, hex: the hash lockstep compares, which leaves this peer's session out (og.Input, og.Lockstep, input and room events, the UI's focus; module docs). Two peers at the same tick agree on it exactly when their simulations agree; the world dump's `hash` covers everything and differs between peers."
  param world World "The world."
  returns String
}
```

<sub>Module `net` line 252</sub>
