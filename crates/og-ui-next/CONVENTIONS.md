# og-ui-next conventions

The new core's own primitives. Nothing here accepts an old convention:
sheets, components and packs are rewritten to these. Old forms and their
replacements are listed in the last section.

## Layout: typed element fields

The compiler reads these attributes of a native element into typed fields;
they are not CSS-visible, and no selector matches them.

| Attribute | Values | Lays out as |
|---|---|---|
| `layout` | `row` | flex row, children centred on the cross axis |
| | `stack` | flex column |
| | `wrap` | flex row that wraps, children centred on the cross axis |
| | `overlay` | flex column; its `anchor` children lie over it |
| | `scroll` | flex column that scrolls and may shrink to 0 |
| | `center` | flex column, children centred, at most `--layout-measure` wide, centred in its parent |
| `gap` | `none` `hair` `tight` `base` (default) `loose` `wide` | the gap between children: `0`, `--layout-gap-hair`, `-tight`, `--layout-gap`, `-loose`, `-wide` |
| `anchor` | `fill` `top-start` `top-end` `bottom-start` `bottom-end` `top-edge` `bottom-edge` | out of flow, at that corner or edge, `--layout-anchor-inset` in (`fill` and the edges: 0) |
| `fill` | (present) | takes the room its parent has left: `flex: 1 1 0`, min size 0 |
| `reverse` | (present) | lays its children out from the end (`row-reverse`, `column-reverse`, `wrap-reverse`) |

The field is the element's lowest style: its sheet's rules override it.
Anything else (alignment, a fixed size, a grid) is an ordinary declaration
on a class.

## Surfaces: typed per-part records

`part` names the element's surface: `frame`, `band`, `title`, `row`,
`cell`, `card`, `badge`, `well`, `figure`, `tab`, `dot`, `fade` and so on,
unqualified (`part="frame"`, not `part="AbilitySlot.frame"`). No `parts`
meta line. Two attributes pick the surface's variant:

- `variant`: `hud`, `bare`, `lead`, `small`, … (was a class on the part).
- `tier`: `common`, `uncommon`, `rare`, `epic`, `legendary`, `mythic` (was
  `data-rarity`).

A surface's record comes from the style's surfaces sheet, written as
compound class selectors over the part, variant and tier names:

```css
.frame { padding: var(--frame-pad); background: var(--frame-fill); }
.frame.hud { background: var(--plate-fill); }
.frame.bare { padding: 0; background: none; }
.cell:hover { background: var(--cell-hover-fill); }
.cell.rare { background: var(--cell-rare-fill); }
```

A selector is one compound of `.name`s and at most one state; no
combinators, attributes, `:where`/`:is`/`:not`/`:has`. The record lies
under the element's own sheet rules and over its layout field. The default
surfaces sheet is `crates/og-ui-next/default/surfaces.css`.

## Document sheets

A `.svelte` document's `<style>` may use:

- `.class`, `#id`, `tag`, and compounds of them;
- the descendant combinator between the document's own elements;
- states: `:hover`, `:active`, `:focus` / `:focus-visible`, `:disabled`;
- dynamic classes through `class:name={expr}`, and `style:--x={expr}` for a
  value a binding writes.

Not taken: attribute selectors (`[data-…]`, `[aria-…]`), `:where`, `:is`,
`:not`, `:has`, `:host`, `::part`, `:global`, `@media`, `@container`,
sibling combinators and structural pseudo-classes. Every `var()` resolves
at cook time; only the tokens a `style:--x` binding reads ship to runtime.

## States

A state a binding decides is a CSS custom state: the element writes
`state:selected={…}`, and a sheet matches `:state(selected)`, in a document
sheet and in a surfaces sheet alike (`.row:state(selected)`,
`.cell:state(empty)`). The state holds while the binding's value is set
and not `false`. It replaces `aria-selected`, `aria-current`,
`aria-pressed`, `data-selected`, `data-done`, `data-current`, `data-locked`,
`data-empty`, `data-active` and the like. Accessibility attributes stay on
the element for what they mean, never for styling.

## SVG

On `<svg>` and its shapes, `fill` and `stroke` are paint, as attributes or
in a sheet (`fill: none`, `stroke: currentColor`, `stroke-width`). SVG
shapes carry no `part`.

## Next head (not compiled yet)

- Size classes of the UI's own box, for members and pages alike:
  `:size(narrow)` (width < 600 px), `:size(short)` (height < 500 px),
  `:size(compact)` (either). They replace every `@media` and `@container`
  rule. 844×390 is `short`, 390×844 is `narrow`. Leave the `@media` and
  `@container` rules until it lands.

## Packs

A pack is a directory with `tokens.css` (`:root { --token: value; }`, and
`:root.light { … }` for the light scheme) and `surfaces.css` (the same
syntax as the default surfaces sheet, over it). The cook compiles each
pack into its own token table and surface records; swapping a pack at
runtime switches tables. No `::part`, no global rules.

## Old forms and their replacements

| Old | New |
|---|---|
| `data-layout="row\|stack\|overlay\|scroll\|center"` | `layout="…"` |
| `data-layout="cluster"`, `"switcher"` | `layout="wrap"` |
| `data-layout="cover"` | `layout="stack" fill` plus `justify-content: center; align-items: center` on a class |
| `data-layout="shell"` | `layout="stack" fill` |
| `data-layout="ratio"`, `"grid"`, `"sidebar"`, `"strip"` | a class with `aspect-ratio` / `display: grid` declarations |
| `data-gap="…"` | `gap="…"` |
| `data-anchor="…"` | `anchor="…"` (`top`, `bottom` and `center` become a class) |
| `data-fill` | `fill` |
| `data-from="end"` | `reverse` |
| `part="Comp.frame"` + `parts` meta line | `part="frame"` |
| `class="hud"` / `"bare"` / `"lead"` / `"small"` on a part | `variant="…"` |
| `data-rarity="rare"` | `tier="rare"` |
| `[aria-selected=true]`, `[data-done=true]`, … | `state:selected={…}` and `:state(selected)` |
| `:where([part$=".frame"]…)` rules in default.css | `.frame…` in `default/surfaces.css` |
| `::part(…)` in a pack | the pack's `surfaces.css` |
