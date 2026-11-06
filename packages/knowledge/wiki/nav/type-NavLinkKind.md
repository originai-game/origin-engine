<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`nav`](../nav.md) · [index](../index.md)

<a id="type-NavLinkKind"></a>

### `NavLinkKind` (enum)

How an agent crosses an off-mesh link.

A string: one of the variants below. The zero value is `"jump"`.

| Variant | Doc |
|---|---|
| `"jump"` | Through the air on an arc, in its length over the agent's speed (at least 0.3 s). |
| `"ladder"` | Climbing, at half the agent's speed. |
| `"door"` | Walking through at the agent's speed, while the link is open (og.nav.setLinkOpen). |
| `"teleport"` | At once: on the far side the tick after it is reached. |

```ogt
enum NavLinkKind {
  doc "How an agent crosses an off-mesh link."
  variant jump "Through the air on an arc, in its length over the agent's speed (at least 0.3 s)."
  variant ladder "Climbing, at half the agent's speed."
  variant door "Walking through at the agent's speed, while the link is open (og.nav.setLinkOpen)."
  variant teleport "At once: on the far side the tick after it is reached."
}
```

<sub>`api.ogt` line 13783</sub>
