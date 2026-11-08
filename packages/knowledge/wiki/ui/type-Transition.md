<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-Transition"></a>

### `Transition` (union)

How a value moves to a new target.

Tagged by `kind`.

| Case | Payload | Doc |
|---|---|---|
| `kind: "instant"` | [`InstantSpec`](../ui/type-InstantSpec.md#type-InstantSpec) | At once. |
| `kind: "spring"` | [`SpringSpec`](../ui/type-SpringSpec.md#type-SpringSpec) | On a spring that keeps its velocity. |
| `kind: "tween"` | [`TweenSpec`](../ui/type-TweenSpec.md#type-TweenSpec) | Over a duration on an ease. |
| `kind: "token"` | [`TokenRef`](../ui/type-TokenRef.md#type-TokenRef) | The style's token. |

```ogt
union Transition {
  doc "How a value moves to a new target."
  tag kind
  case instant InstantSpec "At once."
  case spring SpringSpec "On a spring that keeps its velocity."
  case tween TweenSpec "Over a duration on an ease."
  case token TokenRef "The style's token."
}
```

<sub>Module `ui` line 4708</sub>
