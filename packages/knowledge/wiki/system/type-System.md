<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`system`](../system.md) · [index](../index.md)

<a id="type-System"></a>

### `System` (interface)

A registered system.

```ogt
interface System {
  doc "A registered system."
  # members follow, each quoted below
```

<sub>`api.ogt` line 1394</sub>

<a id="member-System-name"></a>

#### `System.name` (property)

The system's unique name.

```ts
readonly name: string
```

Binding symbol `og_system_get_name` · publish builds: kept

```ogt
prop name String "The system's unique name."
```

<sub>`api.ogt` line 1396</sub>

<a id="member-System-reads"></a>

#### `System.reads` (property)

What it reads.

```ts
readonly reads: readonly AnyReflected[]
```

Binding symbol `og_system_get_reads` · publish builds: kept

```ogt
prop reads List.AnyReflected "What it reads."
```

<sub>`api.ogt` line 1397</sub>

<a id="member-System-writes"></a>

#### `System.writes` (property)

What it writes.

```ts
readonly writes: readonly AnyReflected[]
```

Binding symbol `og_system_get_writes` · publish builds: kept

```ogt
prop writes List.AnyReflected "What it writes."
```

<sub>`api.ogt` line 1398</sub>
