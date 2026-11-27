<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`table`](../table.md) · [index](../index.md)

<a id="type-TableRow"></a>

### `TableRow` (handle)

A row of a loaded table: a hash of its table's asset id and its own id (52 bits, so exact as a number), never 0. It names the same row after a reload or in a save; a row a table no longer has answers no values (og.table.missing lists who still holds one).

A generational handle: a number the engine hands out. A dead handle is an error value, never a crash.

```ogt
handle TableRow "A row of a loaded table: a hash of its table's asset id and its own id (52 bits, so exact as a number), never 0. It names the same row after a reload or in a save; a row a table no longer has answers no values (og.table.missing lists who still holds one)."
```

<sub>Module `table` line 4</sub>
