<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-UiSource"></a>

### `UiSource` (interface)

A store's stable checkpoint identity. Stores are created in a named scope.

```ogt
interface UiSource {
  doc "A store's stable checkpoint identity. Stores are created in a named scope."
  # members follow, each quoted below
```

<sub>Module `ui` line 7765</sub>

<a id="member-UiSource-scope"></a>

#### `UiSource.scope` (property)

The owning scope.

```ts
readonly scope: string
```

```ogt
prop scope String "The owning scope." {
  host false
}
```

<sub>Module `ui` line 7767</sub>

<a id="member-UiSource-id"></a>

#### `UiSource.id` (property)

The store ID within that scope.

```ts
readonly id: string
```

```ogt
prop id String "The store ID within that scope." {
  host false
}
```

<sub>Module `ui` line 7770</sub>
