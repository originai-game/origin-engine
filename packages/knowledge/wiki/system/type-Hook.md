<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`system`](../system.md) · [index](../index.md)

<a id="type-Hook"></a>

### `Hook` (interface)

A declared hook.

```ogt
interface Hook {
  doc "A declared hook."
  # members follow, each quoted below
```

<sub>Module `system` line 66</sub>

<a id="member-Hook-name"></a>

#### `Hook.name` (property)

The hook's unique name.

```ts
readonly name: string
```

Binding symbol `og_hook_get_name` · publish builds: kept

```ogt
prop name String "The hook's unique name."
```

<sub>Module `system` line 68</sub>
