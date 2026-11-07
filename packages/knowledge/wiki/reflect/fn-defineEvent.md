<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`reflect`](../reflect.md) · [index](../index.md)

<a id="fn-defineEvent"></a>

### `defineEvent`

Declare a reflected gameplay event. Its stable name and packed field layout identify recorded intents; changing that layout refuses old intent recordings.

```ts
defineEvent<F extends Fields>(name: string, fields: F): EventType<FieldValues<F>>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `name` | `string` | no | The event's unique name, PascalCase. |
| `fields` | `F` | no | The payload layout, as for defineComponent. |

**Returns** `EventType<FieldValues<F>>`: The token world.events, world.emit and input.intent take.

Binding symbol `og_reflect_define_event` · publish builds: kept

```ogt
fn defineEvent {
  doc "Declare a reflected gameplay event. Its stable name and packed field layout identify recorded intents; changing that layout refuses old intent recordings."
  generic F Fields
  param name String "The event's unique name, PascalCase."
  param fields F "The payload layout, as for defineComponent."
  returns EventType.F "The token world.events, world.emit and input.intent take."
}
```

<sub>Module `reflect` line 122</sub>
