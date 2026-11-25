<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`shared`](../shared.md) · [index](../index.md)

<a id="type-SharedState"></a>

### `SharedState` (enum)

Where this client's session stands (og.SharedWorld.state).

A string: one of the variants below. The zero value is `"off"`.

| Variant | Doc |
|---|---|
| `"off"` | No session: og.shared.join was never called, or og.shared.leave ended it. |
| `"connecting"` | Joining: the host asked its session provider for a connection and the server has not welcomed this client yet. |
| `"joined"` | In the world: the server took this client in; remote avatars arrive. |
| `"reconnecting"` | The connection was lost (or the server moved this client): the engine asks the provider for a new connection, backing off from 250 ms to 4 s for 30 s, and the server keeps this avatar 10 s for it to resume. |
| `"refused"` | The provider or the server refused this client, or reconnecting gave up: og.SharedLeft says why. |

```ogt
enum SharedState {
  doc "Where this client's session stands (og.SharedWorld.state)."
  variant off "No session: og.shared.join was never called, or og.shared.leave ended it."
  variant connecting "Joining: the host asked its session provider for a connection and the server has not welcomed this client yet."
  variant joined "In the world: the server took this client in; remote avatars arrive."
  variant reconnecting "The connection was lost (or the server moved this client): the engine asks the provider for a new connection, backing off from 250 ms to 4 s for 30 s, and the server keeps this avatar 10 s for it to resume."
  variant refused "The provider or the server refused this client, or reconnecting gave up: og.SharedLeft says why."
}
```

<sub>Module `shared` line 8</sub>
