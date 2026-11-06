<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`system`](../system.md) · [index](../index.md)

<a id="fn-defineHook"></a>

### `defineHook`

Declare a hook on a component; list it in GameDesc.hooks.

```ts
defineHook<C extends AnyComponent>(name: string, component: C, desc: HookDesc, run: HookRun): Hook
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `name` | `string` | no | The hook's unique name. |
| `component` | `C` | no | The component it reacts to. |
| `desc` | `HookDesc` | no | Add or remove, and its access. |
| `run` | `HookRun` | no | The body. |

**Returns** `Hook`.

Binding symbol `og_system_define_hook` · publish builds: kept

```ogt
fn defineHook {
  doc "Declare a hook on a component; list it in GameDesc.hooks."
  generic C AnyComponent
  param name String "The hook's unique name."
  param component C "The component it reacts to."
  param desc HookDesc "Add or remove, and its access."
  param run HookRun "The body."
  returns Hook
}
```

<sub>Module `system` line 72</sub>
