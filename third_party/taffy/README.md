# taffy 0.15.1, patched

og-ui's and og-ui-next's layout lays out CSS Flexbox, Grid and block flow
with floats through `taffy` 0.15.1 from https://github.com/DioxusLabs/taffy
(https://crates.io/crates/taffy/0.15.1), under the MIT licence. `LICENSE`
contains its licence. It is used through the workspace's
`[patch.crates-io]`; `Cargo.toml` is the published one without its
`[[example]]` tables (the examples are not vendored).

Five changes; 3, 4 and 5 are marked `Origin Engine` in place.

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

3. `src/compute/flexbox.rs`, `determine_hypothetical_cross_size`. A flex
item with an aspect ratio and no cross size took its content's cross
size, measured at its used main size, so a 7:10 card in an
`items-center` row was as tall as its text. Its used main size now
reaches the cross axis through the ratio (CSS Sizing 4's automatic size
in the ratio-dependent axis) and floors the measured size, which may
still make it taller (`min-height: auto`), as the intrinsic main-size
step already does the other way.

4. Same function, its new `transfer_main` argument. When a column
container's width is asked for alone (`RunMode::ComputeSize` along the
horizontal axis), that width is its intrinsic inline size, its items'
width contributions (CSS Flexbox §9.9.3), so change 3's transfer is
skipped there: a column of flexed square items stays as wide as their
own widths, not the widths their flexed heights give.

5. `src/compute/flexbox.rs`, `collect_flex_lines`. An item wraps only when
its line passes the available main size by more than `LAYOUT_SLACK`
(0.01 layout units, `src/lib.rs`, the slack og-ui's text breaks past
too). A fit-content wrapping row is as wide as its items' max-content
sum, and the line break summed them back in another order, so it lost a
few ulps and wrapped its last item onto a second line, while the row kept
the height of one.
