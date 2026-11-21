# taffy 0.15.1, patched

og-ui's and og-ui-next's layout lays out CSS Flexbox, Grid and block flow
with floats through `taffy` 0.15.1 from https://github.com/DioxusLabs/taffy
(https://crates.io/crates/taffy/0.15.1), under the MIT licence. `LICENSE`
contains its licence. It is used through the workspace's
`[patch.crates-io]`; `Cargo.toml` is the published one without its
`[[example]]` tables (the examples are not vendored).

Two changes.

1. `src/compute/flexbox.rs`, `determine_container_main_size`.
A container with no definite main size under definite available space
(fit-content: a grid item with `justify-self: center`, an absolutely
placed box) took the sum of its items' flex bases, so a `flex: 1 1 0`
item counted as 0 and was squeezed to its min-content. A single-line
container now takes its max-content size, which sums the items'
max-content contributions whatever their basis (CSS Flexbox §9.9.1),
capped by the available space and floored by the old sum. The
max-content computation moved, unchanged, out of the `MinContent |
MaxContent` arm into `intrinsic_main_size`, which both arms call. Upstream
main has the same code (checked 2026-11-21).

2. CSS's `order` (Flexbox §5.4, Grid §8.5), which taffy 0.15.1 lacks:
`Style::order` and `CoreStyle::order` (default 0). Flexbox lays its
items out in order-modified document order (a stable sort by `order`,
done only when an item has one); the out-of-flow pass, which walks the
items in document order, then finds each in-flow child's item by its
document index. Grid sorts its items by `order` for auto-placement
only and puts them back in document order after it. Focus and reading
order stay in document order, as CSS has them.
