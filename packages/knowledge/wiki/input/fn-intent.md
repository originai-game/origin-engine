<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`input`](../input.md) · [index](../index.md)

<a id="fn-intent"></a>

### `og.input.intent`

Send a reflected gameplay intent for the next tick and record its name, packed payload and layout hash. UI controls use this boundary instead of writing gameplay state. Inside a system, declare the event among its writes.

```ts
og.input.intent<E extends AnyEvent>(world: World, event: E, payload: Value<E>): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The game world. |
| `event` | `E` | no | The reflected event type supplies the stable name and schema. |
| `payload` | `Value<E>` | no | The event's fields. |

**Returns** `Result<void>`.

Binding symbol `og_input_intent` · publish builds: kept

```ogt
fn intent {
  doc "Send a reflected gameplay intent for the next tick and record its name, packed payload and layout hash. UI controls use this boundary instead of writing gameplay state. Inside a system, declare the event among its writes."
  generic E AnyEvent
  param world World "The game world."
  param event E "The reflected event type supplies the stable name and schema."
  param payload Value.E "The event's fields."
  returns Result.Void
}
```

<sub>Module `input` line 10</sub>
