<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`system`](../system.md) · [index](../index.md)

<a id="type-HookEvent"></a>

### `HookEvent` (enum)

What a hook reacts to.

A string: one of the variants below. The zero value is `"add"`.

| Variant | Doc |
|---|---|
| `"add"` | The component was added (spawned with, or inserted into an entity without it): the hook runs after it is in place. |
| `"remove"` | The component is being removed (removed, or its entity despawned): the hook runs before it goes, and reads it. |

```ogt
enum HookEvent {
  doc "What a hook reacts to."
  variant add "The component was added (spawned with, or inserted into an entity without it): the hook runs after it is in place."
  variant remove "The component is being removed (removed, or its entity despawned): the hook runs before it goes, and reads it."
}
```

<sub>`api.ogt` line 1356</sub>
