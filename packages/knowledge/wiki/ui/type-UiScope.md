<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-UiScope"></a>

### `UiScope` (interface)

Named presentation stores for one world, following the svelte/store contract. Svelte template documents own structure; this scope supplies data. Re-register the same IDs on reload to preserve source values.

```ogt
interface UiScope {
  doc "Named presentation stores for one world, following the svelte/store contract. Svelte template documents own structure; this scope supplies data. Re-register the same IDs on reload to preserve source values."
  # members follow, each quoted below
```

<sub>Module `ui` line 1709</sub>

<a id="member-UiScope-id"></a>

#### `UiScope.id` (property)

The stable scope ID.

```ts
readonly id: string
```

```ogt
prop id String "The stable scope ID." {
  host false
}
```

<sub>Module `ui` line 1711</sub>

<a id="member-UiScope-writable"></a>

#### `UiScope.writable`

Create a named writable store with subscribe, set and update. Re-registration preserves its checkpointed value. Failures are reported by takeErrors; setters also return Result.

```ts
UiScope.writable<T extends unknown>(id: string, initial: T): UiWritable<T>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | An explicit, nonempty stable store ID. |
| `initial` | `T` | no | The initial serializable value, used only for a new store. |

**Returns** `UiWritable<T>`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn writable {
  host false
  generic T Json
  doc "Create a named writable store with subscribe, set and update. Re-registration preserves its checkpointed value. Failures are reported by takeErrors; setters also return Result."
  param id String "An explicit, nonempty stable store ID."
  param initial T "The initial serializable value, used only for a new store."
  returns UiWritable.T
}
```

<sub>Module `ui` line 1714</sub>

<a id="member-UiScope-readable"></a>

#### `UiScope.readable`

Create a named readable store. start runs when its first subscriber arrives and its cleanup runs when the last leaves. No promises, timers or wall-clock work in a tick.

```ts
UiScope.readable<T extends unknown>(id: string, initial: T, start?: UiStart<T>): UiReadable<T>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | The stable store ID. |
| `initial` | `T` | no | The initial serializable value. |
| `start` | `UiStart<T>` | yes | The synchronous start/stop notifier. |

**Returns** `UiReadable<T>`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn readable {
  host false
  generic T Json
  doc "Create a named readable store. start runs when its first subscriber arrives and its cleanup runs when the last leaves. No promises, timers or wall-clock work in a tick."
  param id String "The stable store ID."
  param initial T "The initial serializable value."
  param start UiStart.T "The synchronous start/stop notifier." {
    optional true
  }
  returns UiReadable.T
}
```

<sub>Module `ui` line 1722</sub>

<a id="member-UiScope-derived"></a>

#### `UiScope.derived`

Derive a store synchronously from one store or a tuple of stores. Inputs are explicit and settle before subscribers run; undeclared reads and reentrant writes report errors in every build. Failure preserves the last good value, or the optional initial fallback; without either it reports an error and yields undefined.

```ts
UiScope.derived<S extends UiStores, T extends unknown>(id: string, stores: S, compute: UiDerive<S, T>, initial?: T): UiReadable<T>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | The stable derived store ID, also its callback reload identity. |
| `stores` | `S` | no | One named store or a readonly tuple of named stores from this world. |
| `compute` | `UiDerive<S, T>` | no | A pure synchronous function of the input values. |
| `initial` | `T` | yes | An optional fallback if the first computation fails. |

**Returns** `UiReadable<T>`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn derived {
  host false
  generic S UiStores
  generic T Json
  doc "Derive a store synchronously from one store or a tuple of stores. Inputs are explicit and settle before subscribers run; undeclared reads and reentrant writes report errors in every build. Failure preserves the last good value, or the optional initial fallback; without either it reports an error and yields undefined."
  param id String "The stable derived store ID, also its callback reload identity."
  param stores S "One named store or a readonly tuple of named stores from this world."
  param compute UiDerive.S.T "A pure synchronous function of the input values."
  param initial T "An optional fallback if the first computation fails." {
    optional true
  }
  returns UiReadable.T
}
```

<sub>Module `ui` line 1733</sub>

<a id="member-UiScope-get"></a>

#### `UiScope.get`

Read a store by subscribing and immediately unsubscribing, as svelte/store get does.

```ts
UiScope.get<T extends unknown>(store: UiReadable<T>): T
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `store` | `UiReadable<T>` | no | The named store. |

**Returns** `T`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn get {
  host false
  generic T Json
  doc "Read a store by subscribing and immediately unsubscribing, as svelte/store get does."
  param store UiReadable.T "The named store."
  returns T
}
```

<sub>Module `ui` line 1746</sub>

<a id="member-UiScope-collection"></a>

#### `UiScope.collection`

The engine-specific store: ordered rows with stable string keys, plus insert, remove, move and field patch. Duplicate keys and key-changing patches are refused. Re-registration keeps data and order.

```ts
UiScope.collection<T extends unknown>(id: string, items: readonly T[], desc?: UiCollectionDesc): UiCollection<T>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | The stable collection ID. |
| `items` | `readonly T[]` | no | Initial rows, used only for a new collection. |
| `desc` | `UiCollectionDesc` | yes | The key field. |

**Returns** `UiCollection<T>`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn collection {
  host false
  generic T Json
  doc "The engine-specific store: ordered rows with stable string keys, plus insert, remove, move and field patch. Duplicate keys and key-changing patches are refused. Re-registration keeps data and order."
  param id String "The stable collection ID."
  param items List.T "Initial rows, used only for a new collection."
  param desc UiCollectionDesc "The key field." {
    optional true
  }
  returns UiCollection.T
}
```

<sub>Module `ui` line 1753</sub>

<a id="member-UiScope-mount"></a>

#### `UiScope.mount`

Register a loaded document once with named store inputs. The native renderer updates it from store changes; no per-element TypeScript calls are made. Re-registering the same ID and inputs preserves its mounted identity. Inputs stay subscribed until unmount or scope disposal. A `type screen` document is a layer of the stack while it is mounted (docs/ui.md §2.1): its `<meta name="screen" data-layer …>` decides its kind, input, pause, Back and focus memory, Back or a `pop` trigger unmounts it, and its `@view-transition` animates it (a cross-fade by default) as it joins and leaves the stack.

```ts
UiScope.mount(id: string, document: string, inputs: Readonly<Record<string, UiReadable<unknown>>>, content?: Readonly<Record<string, string>>): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Stable mount ID within this scope. |
| `document` | `string` | no | The loaded document's declared name. |
| `inputs` | `Readonly<Record<string, UiReadable<unknown>>>` | no | Document input names mapped to named stores from this world. |
| `content` | `Readonly<Record<string, string>>` | yes | Content passed by reference (Svelte snippets as props): a name the document renders with `{@render name()}`, mapped to a loaded document's name (its root) or `document#snippet` (one of its snippets), rendered with this mount's stores. |

**Returns** `Result<void>`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn mount {
  host false
  doc "Register a loaded document once with named store inputs. The native renderer updates it from store changes; no per-element TypeScript calls are made. Re-registering the same ID and inputs preserves its mounted identity. Inputs stay subscribed until unmount or scope disposal. A `type screen` document is a layer of the stack while it is mounted (docs/ui.md §2.1): its `<meta name=\"screen\" data-layer …>` decides its kind, input, pause, Back and focus memory, Back or a `pop` trigger unmounts it, and its `@view-transition` animates it (a cross-fade by default) as it joins and leaves the stack."
  param id String "Stable mount ID within this scope."
  param document String "The loaded document's declared name."
  param inputs Map.UiReadable.Json "Document input names mapped to named stores from this world."
  param content Map.String "Content passed by reference (Svelte snippets as props): a name the document renders with `{@render name()}`, mapped to a loaded document's name (its root) or `document#snippet` (one of its snippets), rendered with this mount's stores." {
    optional true
  }
  returns Result.Void
}
```

<sub>Module `ui` line 1764</sub>

<a id="member-UiScope-unmount"></a>

#### `UiScope.unmount`

Remove a registered document and release its input subscriptions. Removing an absent mount succeeds.

```ts
UiScope.unmount(id: string): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | The mount ID in this scope. |

**Returns** `Result<void>`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn unmount {
  host false
  doc "Remove a registered document and release its input subscriptions. Removing an absent mount succeeds."
  param id String "The mount ID in this scope."
  returns Result.Void
}
```

<sub>Module `ui` line 1775</sub>

<a id="member-UiScope-takeErrors"></a>

#### `UiScope.takeErrors`

Drain operation and scheduler errors naming scope and store. Errors are reported in publish too; the scheduler never throws into the engine.

```ts
UiScope.takeErrors(): readonly string[]
```

**Returns** `readonly string[]`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn takeErrors {
  host false
  doc "Drain operation and scheduler errors naming scope and store. Errors are reported in publish too; the scheduler never throws into the engine."
  returns List.String
}
```

<sub>Module `ui` line 1781</sub>

<a id="member-UiScope-dispose"></a>

#### `UiScope.dispose`

Remove the scope's stores and subscriptions and run start/stop cleanups.

```ts
UiScope.dispose(): Result<void>
```

**Returns** `Result<void>`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn dispose {
  host false
  doc "Remove the scope's stores and subscriptions and run start/stop cleanups."
  returns Result.Void
}
```

<sub>Module `ui` line 1786</sub>
