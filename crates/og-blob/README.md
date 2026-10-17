# og-blob

`.ogb`, the one cooked blob container (`docs/design.md` §2.5). The cook
(`og-cook`) writes it; every runtime loader reads it. The crate knows nothing
about what the sections mean: it owns the layout, the validation and the
refusal of stale blobs (design §6). Its only dependency is `bytemuck`.

## API

| Item | What it does |
|---|---|
| `BlobWriter::new(type_hash, schema_version, cook_stamp)` | starts a blob |
| `.raw(name, bytes, align)` | a byte section aligned to `align` (power of two, ≤ 4096) |
| `.table::<T: Pod>(name, rows, row_type)` | rows of a plain-data struct, tagged with the row type's layout hash, aligned to `max(align_of T, 16)` |
| `.rows(name, bytes, stride, row_type)` | rows whose layout is known only at run time (scene component tables) |
| `.finish() -> Vec<u8>` | the bytes; the same calls give the same bytes |
| `Blob::parse(bytes, path)` | validates the structure, borrows the bytes |
| `Blob::open(bytes, path, &Expect { type_hash, schema_version, stamp })` | `parse` plus the loader's expectations; refuses with the path and both values |
| `blob.raw(name)`, `blob.table::<T>(name, row_type)`, `blob.table_view`, `blob.rows` | sections: in place, or copied when the bytes are not aligned for `T` |
| `AlignedBytes::from_slice` | a 16-aligned copy of a file, so in-place views succeed |
| `BlobError` | `Malformed`, `ContainerVersion`, `TypeMismatch`, `SchemaVersion`, `StampMismatch { path, expected, found }`, `MissingSection`, `SectionType` |

## Layout, byte by byte

All integers are little-endian. Offsets are from the start of the file.

### Header (64 bytes)

| Offset | Size | Field | Value |
|---:|---:|---|---|
| 0 | 8 | magic | `89 4F 47 42 0D 0A 1A 0A` (`\x89OGB\r\n\x1a\n`, PNG-style: catches text-mode transfers and truncation) |
| 8 | 2 | container version | `1` |
| 10 | 2 | header size | `64` |
| 12 | 4 | section count `n` | |
| 16 | 8 | type hash | what the blob holds: for `og-cook` blobs, FNV-1a over the layout hashes of the root and row schemas (`og_cook::format::type_hash`) |
| 24 | 4 | schema version | the root schema's version |
| 28 | 4 | flags | `0` (reserved; non-zero is refused) |
| 32 | 8 | cook stamp | FNV-1a of the cooker version, the options this output depends on and its inputs (`og-cook` README §Stamps) |
| 40 | 8 | file size | the whole file; a mismatch (truncation, padding) is refused |
| 48 | 4 | name pool size | bytes of section names |
| 52 | 4 | max align | the largest section alignment: place the bytes at this alignment and every in-place view succeeds |
| 56 | 8 | reserved | `0` |

### Section table (`n` × 48 bytes, from offset 64)

| Offset | Size | Field |
|---:|---:|---|
| 0 | 4 | name offset in the name pool |
| 4 | 4 | name length (UTF-8, not terminated, non-empty, unique) |
| 8 | 8 | payload offset (a multiple of `align`) |
| 16 | 8 | payload size |
| 24 | 8 | row type hash (0 for raw bytes) |
| 32 | 4 | stride: bytes per row (0 for raw bytes; `size` is a multiple of it) |
| 36 | 4 | align (power of two, ≤ 4096) |
| 40 | 8 | reserved, `0` |

### Name pool and payloads

The name pool follows the table (`64 + 48n`, `name pool size` bytes). Payloads
follow in table order, each at the next offset aligned to its `align`, padded
with zero bytes; payloads never overlap and appear in increasing offset order.
Nothing follows the last payload.

## Guarantees

- **Deterministic**: sections in insertion order, zero padding, no
  timestamps: the same writer calls give the same bytes on every host.
- **Validated before use**: a parse checks magic, version, header size, file
  size, reserved fields, every section's bounds, alignment, order, stride
  and name; a damaged file is an error value, never a panic (the tests flip
  every header and table byte).
- **Typed tables**: `table::<T>` refuses a section whose row type hash or
  stride differs from what the loader asks for.
- **Stale blobs are refused** by `open` with
  `"<path>: cook stamp mismatch: the blob has <found>, the manifest expects <expected>; re-run og cook"`.

## What the runtime side must honour

- Fetch a blob into memory aligned to `max_align` (16 for every blob `og-cook`
  writes today; `AlignedBytes` does it natively) to view tables in place;
  misaligned bytes still load, by copy.
- The expected stamp comes from the cook manifest (`cooked/.cook.ogt`), never
  from the blob itself.
- A container version bump is a breaking change for every cooked project:
  the cook re-cooks (the cooker version is in every stamp), old runtimes
  refuse.

## Tests

`cargo test -p og-blob`: determinism, round trip and alignment, in-place vs
copied tables, refusal of stale/wrong-type/wrong-version blobs with path and
stamps, truncation and corruption.
