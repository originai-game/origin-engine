<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-UiBuild"></a>

### `UiBuild` (callback)

Builds a container's children: ui calls made inside it nest under the container.

Game code supplies this function.

```ts
type UiBuild = () => void
```

```ogt
callback UiBuild {
  doc "Builds a container's children: ui calls made inside it nest under the container."
}
```

<sub>`api.ogt` line 16234</sub>
