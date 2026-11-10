<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `components` guide: The UI component catalogue

Core elements and their reflected properties for scriptless Svelte documents. Unknown elements, properties, and values fail cooking. The `screens` guide describes contracts, bindings, styles, and content composition.

Write `<button id="Buy" onclick={buy}>Buy</button>`. Put CSS declarations such as `gap: 8px` in a scoped `<style>` block. Bind attributes with `value={$amount}` and writable controls with `bind:value={$amount}`. Use `<text>` for the reflected label kind and `<img>` for image. Named content uses snippets and `{@render name()}`, not legacy slot elements.

<a id="guide-components-1"></a>
### `components §1` Every element

What every kind takes: its box, layout, paint, motion and input.

| Write | Value | What |
|---|---|---|
| `width:` | `auto \| fit-content \| fit-content(<n>px) \| min-content \| max-content \| stretch \| <n>px \| <n>% \| calc(…) \| min(…) \| max(…) \| clamp(…) \| round(…) …` | Its width (fit by default). `anchorWidth`: CSS `width: anchor-size(…)` as written. |
| `height:` | `auto \| fit-content \| fit-content(<n>px) \| min-content \| max-content \| stretch \| <n>px \| <n>% \| calc(…) \| min(…) \| max(…) \| clamp(…) \| round(…) …` | Its height (fit by default). `anchorHeight`: CSS `height: anchor-size(…)` as written. |
| `padding:` | `1 to 4 lengths (top right bottom left)` | Space inside its edges, CSS px. |
| `padding-top:` | `<n>px` | Space inside its top edge, CSS px (over `padding`). |
| `padding-right:` | `<n>px` | Space inside its right edge, CSS px (over `padding`). |
| `padding-bottom:` | `<n>px` | Space inside its bottom edge, CSS px (over `padding`). |
| `padding-left:` | `<n>px` | Space inside its left edge, CSS px (over `padding`). |
| `min-width:` | `auto \| fit-content \| fit-content(<n>px) \| min-content \| max-content \| stretch \| <n>px \| <n>% \| calc(…) \| min(…) \| max(…) \| clamp(…) \| round(…) …` | Its least width: CSS px, a percentage or calc() of the width its parent offers, or minContent or maxContent. |
| `max-width:` | `auto \| fit-content \| fit-content(<n>px) \| min-content \| max-content \| stretch \| <n>px \| <n>% \| calc(…) \| min(…) \| max(…) \| clamp(…) \| round(…) …` | Its greatest width: CSS px, a percentage or calc() of the width its parent offers, or minContent or maxContent. |
| `min-height:` | `auto \| fit-content \| fit-content(<n>px) \| min-content \| max-content \| stretch \| <n>px \| <n>% \| calc(…) \| min(…) \| max(…) \| clamp(…) \| round(…) …` | Its least height: CSS px, a percentage or calc() of the height its parent offers, or minContent or maxContent. |
| `max-height:` | `auto \| fit-content \| fit-content(<n>px) \| min-content \| max-content \| stretch \| <n>px \| <n>% \| calc(…) \| min(…) \| max(…) \| clamp(…) \| round(…) …` | Its greatest height: CSS px, a percentage or calc() of the height its parent offers, or minContent or maxContent. |
| `box-sizing:` | `border-box \| content-box` | Whether its width and height hold its padding and border (borderBox, every element's default) or its content alone (contentBox). |
| `margin-top:` | `<n>px \| <n>em \| <n>% \| auto \| calc(…)` | Space outside its top edge: CSS px, em of its type step, a percentage of its parent's content width, or auto (the free space). Adjacent block margins collapse. |
| `margin-right:` | `<n>px \| <n>em \| <n>% \| auto \| calc(…)` | Space outside its right edge: CSS px, em of its type step, a percentage of its parent's content width, or auto (the free space). |
| `margin-bottom:` | `<n>px \| <n>em \| <n>% \| auto \| calc(…)` | Space outside its bottom edge: CSS px, em of its type step, a percentage of its parent's content width, or auto (the free space). Adjacent block margins collapse. |
| `margin-left:` | `<n>px \| <n>em \| <n>% \| auto \| calc(…)` | Space outside its left edge: CSS px, em of its type step, a percentage of its parent's content width, or auto (the free space). |
| `gap:` | `<n>px` | Space between its children, CSS px. |
| `justify-content:` | `start \| center \| end \| space-between \| space-around \| space-evenly \| stretch \| flex-start \| flex-end \| normal` | Where children sit along the main axis (`start` and `end` are the writing mode's sides, `flexStart` and `flexEnd` the line's; `normal`: a line's start, a grid's `auto` tracks stretched). |
| `align-items:` | `start \| center \| end \| stretch \| baseline \| last-baseline` | Where children sit across it. |
| `flex-wrap:` | `wrap \| nowrap` | A row (or a column) breaks its children into lines. |
| `wrap-reverse` | `bool` | Its lines stack from the far side (CSS `wrap-reverse`). |
| `row-gap:` | `<n>px` | Space between its rows (a column's children, a row's lines), CSS px, over `gap`. |
| `column-gap:` | `<n>px` | Space between its columns (a row's children, a column's lines), CSS px, over `gap`. |
| `align-content:` | `start \| center \| end \| space-between \| space-around \| space-evenly \| stretch` | Where a wrapping container's lines sit across it, and how a grid's tracks sit down it (CSS `align-content`). |
| `justify-items:` | `start \| center \| end \| stretch \| baseline \| last-baseline` | Where a grid's items sit across their areas (CSS `justify-items`). |
| `safe-justify` | `bool` | Its main-axis alignment keeps an overflow at the start (CSS `safe`). |
| `safe-align` | `bool` | Its cross-axis alignment keeps an overflow at the start (CSS `safe`). |
| `flex-grow:` | `<number>` | Its share of a flex line's free space (CSS `flex-grow`). |
| `flex-shrink:` | `<number>` | How much it gives up when a flex line overflows (CSS `flex-shrink`). |
| `flex-basis:` | `auto \| fit-content \| fit-content(<n>px) \| min-content \| max-content \| stretch \| <n>px \| <n>% \| calc(…) \| min(…) \| max(…) \| clamp(…) \| round(…) …` | Its size in a flex line before the free space is shared (CSS `flex-basis`). |
| `order:` | `<number>` | Its place among its flex or grid siblings (CSS `order`). |
| `align-self:` | `start \| center \| end \| stretch \| baseline \| last-baseline` | Where it sits across its flex line or down its grid area, over its parent's (CSS `align-self`). |
| `justify-self:` | `start \| center \| end \| stretch \| baseline \| last-baseline` | Where it sits across its grid area or its absolute box, over its parent's (CSS `justify-self`). |
| `background:` | `var(--<role>) \| #rrggbb[aa]` | Its fill. |
| `border-radius:` | `<n>px` | Its corner radius, CSS px. |
| `border-top-left-radius:` | `<h> [<v>] (<n>px \| <n>%)` | Its top left corner's radii: `<h> [<v>]`, px or %. |
| `border-top-right-radius:` | `<h> [<v>] (<n>px \| <n>%)` | Its top right corner's radii: `<h> [<v>]`, px or %. |
| `border-bottom-right-radius:` | `<h> [<v>] (<n>px \| <n>%)` | Its bottom right corner's radii: `<h> [<v>]`, px or %. |
| `border-bottom-left-radius:` | `<h> [<v>] (<n>px \| <n>%)` | Its bottom left corner's radii: `<h> [<v>]`, px or %. |
| `corner-top-left-shape:` | `round \| squircle \| bevel \| scoop \| notch \| square \| superellipse(<K>)` | Its top left corner's shape: `superellipse(<K>)`. |
| `corner-top-right-shape:` | `round \| squircle \| bevel \| scoop \| notch \| square \| superellipse(<K>)` | Its top right corner's shape: `superellipse(<K>)`. |
| `corner-bottom-right-shape:` | `round \| squircle \| bevel \| scoop \| notch \| square \| superellipse(<K>)` | Its bottom right corner's shape: `superellipse(<K>)`. |
| `corner-bottom-left-shape:` | `round \| squircle \| bevel \| scoop \| notch \| square \| superellipse(<K>)` | Its bottom left corner's shape: `superellipse(<K>)`. |
| `border-top-width:` | `<n>px` | Its top border's width, CSS px: room inside its box, as padding is. |
| `border-top-style:` | `none \| hidden \| solid \| dashed \| dotted \| double \| groove \| ridge \| inset \| outset` | Its top border's style (none draws nothing and takes no room). |
| `border-top-color:` | `var(--<role>) \| #rrggbb[aa]` | Its top border's colour (unset: its text colour). |
| `border-right-width:` | `<n>px` | Its right border's width, CSS px: room inside its box, as padding is. |
| `border-right-style:` | `none \| hidden \| solid \| dashed \| dotted \| double \| groove \| ridge \| inset \| outset` | Its right border's style (none draws nothing and takes no room). |
| `border-right-color:` | `var(--<role>) \| #rrggbb[aa]` | Its right border's colour (unset: its text colour). |
| `border-bottom-width:` | `<n>px` | Its bottom border's width, CSS px: room inside its box, as padding is. |
| `border-bottom-style:` | `none \| hidden \| solid \| dashed \| dotted \| double \| groove \| ridge \| inset \| outset` | Its bottom border's style (none draws nothing and takes no room). |
| `border-bottom-color:` | `var(--<role>) \| #rrggbb[aa]` | Its bottom border's colour (unset: its text colour). |
| `border-left-width:` | `<n>px` | Its left border's width, CSS px: room inside its box, as padding is. |
| `border-left-style:` | `none \| hidden \| solid \| dashed \| dotted \| double \| groove \| ridge \| inset \| outset` | Its left border's style (none draws nothing and takes no room). |
| `border-left-color:` | `var(--<role>) \| #rrggbb[aa]` | Its left border's colour (unset: its text colour). |
| `outline-width:` | `<n>px` | Its outline's width, CSS px, drawn outside its box and taking no room. |
| `outline-style:` | `none \| auto \| solid \| dashed \| dotted \| double \| groove \| ridge \| inset \| outset` | Its outline's style; auto: the focus ring. |
| `outline-color:` | `var(--<role>) \| #rrggbb[aa]` | Its outline's colour (unset: its text colour). |
| `outline-offset:` | `<n>px` | How far, CSS px, its outline stands off its box. |
| `box-shadow:` | `[inset] <x> <y> [<blur> [<spread>]] [<colour>], … \| none` | Its shadows, as CSS box-shadow: `[inset] <x> <y> [<blur> [<spread>]] [<colour>]`, comma-separated, the first on top. |
| `overflow-x:` | `visible \| hidden \| clip \| scroll \| auto` | What its content past its left and right edges does: visible, hidden or clip (cut), scroll or auto (scrolls). |
| `overflow-y:` | `visible \| hidden \| clip \| scroll \| auto` | What its content past its top and bottom edges does: visible, hidden or clip (cut), scroll or auto (scrolls). |
| `scrollbar-width:` | `auto \| thin \| none` | CSS `scrollbar-width`: thin draws its scrollbars at half the width (a half-width grab strip); none hides them (it still scrolls). |
| `aspect-ratio:` | `<number>` | Width over height (0: none). |
| `interactive` | `bool` | It reports clicks (`on click`). |
| `focusable` | `bool` | With interactive: keys and pads reach it and press it. |
| `adjustable` | `bool` | With focusable: Left and Right step it while it has the focus instead of moving the focus. |
| `visible` | `bool` | False leaves it (and its subtree) out of the tree. |
| `aria-label` | `text` | What a screen reader calls it. |
| `player` | `number` | A split screen's region: player 0 to 3's view. That player's pad focus moves only inside it, and what is pressed inside it says the player. |
| `role` | `button \| link \| text \| heading \| image \| checkbox \| radio \| switch \| slider \| spinbutton \| textbox \| combobox \| listbox \| option \| tab \| tablist \| tabpanel \| radiogroup \| progressbar \| meter \| timer \| status \| alert \| log \| dialog \| alertdialog \| tooltip \| separator \| group \| region \| navigation \| toolbar \| menu \| menubar \| menuitem \| menuitemcheckbox \| menuitemradio \| list \| listitem \| tree \| treeitem \| table \| grid \| row \| rowheader \| columnheader \| cell \| gridcell \| figure \| scrollbar` | Its ARIA role, over its tag's (`status`, `alert`, `log` are live regions). `role`: Its accessibility role over its kind's (ARIA `role`; one of og.ui's roles). |
| `aria-live` | `off \| polite \| assertive` | `aria-live`: a screen reader announces what changes in it, after what it is reading (`polite`) or at once (`assertive`); over its role's. |
| `aria-atomic` | `bool` | `aria-atomic`: a live region announces its whole text when any of it changes, not only what changed; over its role's. |
| `part` | `name` | Its part name a style dresses (`ShopCard.frame`); the document's `parts` lists them. |
| `translate:` | `none \| <x> [<y>]: px, or % of its own box` | `x`: Moved right after layout, CSS px. `y`: Moved down after layout, CSS px. `xShift`: Moved right after layout, a fraction of its own width (`translate`'s %). `yShift`: Moved down after layout, a fraction of its own height (`translate`'s %). |
| `scale:` | `<both> \| <x> <y>` | `scale`: Both scales. `scaleX`: Its horizontal scale. `scaleY`: Its vertical scale. |
| `rotate:` | `<n>deg \| <n>rad \| <n>turn` | A turn, radians, clockwise. |
| `transform-origin:` | `<x>% <y>%` | The transform's origin, fractions of its box. |
| `opacity:` | `<number>` | 0 to 1, multiplied down the tree. |
| `z-index:` | `<number>` | Drawing order among its siblings. |
| `position:` | `static \| absolute \| fixed \| relative \| sticky` | In the flow, placed by its insets in its parent, by its insets on the whole screen (full bleed, under the safe area), or against another element. |
| `inset-inline-start:` | `<n>px` | The inset from its parent's start edge (absolute). |
| `top:` | `<n>px` | The inset from the top. `insetTop`: CSS `top` / `inset-block-start` as written: px, `%` of the containing block, `anchor()` or `calc()` of one plus px. |
| `inset-inline-end:` | `<n>px` | The inset from the end edge. |
| `bottom:` | `<n>px` | The inset from the bottom. `insetBottom`: CSS `bottom` / `inset-block-end` as written (see insetTop). |
| `data-hit-slop` | `number` | Extra pointer reach in CSS px (`data-hit-slop`), without changing layout or paint. Overlapping targets keep their existing hit priority. |
| `grid-template-columns:` | `[name] 120px 1fr auto 25% minmax(min-content, 1fr) repeat(auto-fill, 200px) \| subgrid \| masonry` | A grid's column tracks (CSS `grid-template-columns`: sizes, `[line names]`, `repeat()`, `subgrid` or `masonry`). |
| `grid-template-rows:` | `[name] 120px 1fr auto 25% minmax(min-content, 1fr) repeat(auto-fill, 200px) \| subgrid \| masonry` | A grid's row tracks (CSS `grid-template-rows`). |
| `grid-template-areas:` | `"<a>" "<b>" …` | A grid's named areas, one string per row. |
| `grid-auto-flow:` | `row \| column [dense]` | `flow`: How a grid auto-places. `dense`: A grid back-fills holes. |
| `grid-area:` | `<name>` | The named grid area it takes. |
| `grid-column:`, `grid-row:` | `<n> \| <n> / span <n> \| span <n>` | `cell`: Its grid column and row, 1-based. `span`: Columns and rows it spans. |
| `grid-auto-columns:` | `[name] 120px 1fr auto 25% minmax(min-content, 1fr) repeat(auto-fill, 200px) \| subgrid \| masonry` | A grid's implicit column tracks, repeating (CSS `grid-auto-columns`). |
| `grid-auto-rows:` | `[name] 120px 1fr auto 25% minmax(min-content, 1fr) repeat(auto-fill, 200px) \| subgrid \| masonry` | A grid's implicit row tracks, repeating (CSS `grid-auto-rows`). |
| `grid-column-start:` | `auto \| <n> [<name>] \| <name> \| span [<n>] [<name>]` | Its first grid column line: `<n> [<name>]`, `<name>`, `span [<n>] [<name>]` or auto (CSS `grid-column-start`). |
| `grid-column-end:` | `auto \| <n> [<name>] \| <name> \| span [<n>] [<name>]` | The grid column line it ends at (CSS `grid-column-end`). |
| `grid-row-start:` | `auto \| <n> [<name>] \| <name> \| span [<n>] [<name>]` | Its first grid row line (CSS `grid-row-start`). |
| `grid-row-end:` | `auto \| <n> [<name>] \| <name> \| span [<n>] [<name>]` | The grid row line it ends at (CSS `grid-row-end`). |
| `-og-overlaps:` | `"<a>" "<b>" …` | Paths it overlaps on purpose. |
| `match` | `name` | A shared-element key. |
| `transition:` | `[<property>] <duration> [<timing-function>] [<delay>], …` | How its animatable values move when they change. |
| `-og-layout-transition:` | `none \| <ms>ms [ease] \| -og-spring(<ms>ms, <bounce>) \| var(--motion-<name>)` | Moves from its old rectangle to its new one. |
| `-og-enter:` | `<kind> [<distance>px] [<transition>]` | How it arrives. |
| `-og-exit:` | `<kind> [<distance>px] [<transition>]` | How it leaves. |
| `inert` | `bool` | It and its subtree take no input. |
| `nav-up` | `name` | Where Up moves the focus from it (a path). |
| `nav-down` | `name` | Where Down moves the focus from it. |
| `nav-left` | `name` | Where Left moves the focus from it. |
| `nav-right` | `name` | Where Right moves the focus from it. |
| `nav-next` | `name` | Where Tab moves the focus from it. |
| `nav-prev` | `name` | Where Shift+Tab moves the focus from it. |
| `focus-group` | `stop \| wrap` | Its focusable controls are a focus group that stops (leaves) or wraps at its edge. |
| `focus-memory` | `bool` | With focusGroup: focus coming back lands on its last member. |
| `tap` | `bool` | It recognises a tap (`on tap`). |
| `double-tap` | `bool` | It recognises a double tap (`on doubleTap`); its tap waits for it. |
| `long-press` | `bool` | It recognises a long press (`on longPress`). |
| `long-press-ms` | `number` | How long its long press is held, ms (0: 500). |
| `gesture-priority` | `number` | Among nested recognisers of one press: higher wins. |
| `cursor:` | `[url(<icon or texture>) [<x> <y>],]* auto \| default \| none \| context-menu \| help \| pointer \| progress \| wait \| cell \| crosshair \| text \| vertical-text \| alias \| copy \| move \| no-drop \| not-allowed \| grab \| grabbing \| all-scroll \| col-resize \| row-resize \| n-resize \| e-resize \| s-resize \| w-resize \| ne-resize \| nw-resize \| se-resize \| sw-resize \| ew-resize \| ns-resize \| nesw-resize \| nwse-resize \| zoom-in \| zoom-out` | CSS `cursor`: the pointer's shape over it and its subtree, any CSS keyword. `url(<icon or texture>) [<x> <y>]` images before it are tried in order and the first that loads is shown at its hotspot (at most 128x128), the keyword where none does. |
| `pointer-events:` | `auto \| none \| visiblePainted \| visibleFill \| visibleStroke \| visible \| painted \| fill \| stroke \| bounding-box \| all` | CSS `pointer-events`, inherited: none lets the pointer through to what is under it (keys and pads still reach it; a child saying auto still takes it). On SVG shapes the SVG values pick the fill, the stroke or the bounding box. |
| `touch-action:` | `auto \| none \| [pan-x \| pan-left \| pan-right] \|\| [pan-y \| pan-up \| pan-down] \|\| pinch-zoom \| manipulation` | CSS `touch-action`: the pans (pan-x, pan-left…, pan-y…) and pinch-zoom a touch starting on it may make; none keeps the touch for its own handlers. The used value intersects its ancestors'. |
| `image-rendering:` | `auto \| smooth \| high-quality \| pixelated \| crisp-edges` | CSS `image-rendering`, inherited: pixelated or crisp-edges samples every image it draws (its image, art, background and border image) at the nearest texel, so pixel art scaled up stays sharp; auto, smooth and high-quality keep the texture's own sampling. |
| `visibility:` | `visible \| hidden \| collapse` | CSS `visibility`, inherited: hidden keeps its place but draws nothing and takes no pointer or focus, while a child saying visible is drawn; collapse also gives up a flex item's main size. |
| `-og-text-path:` | `none \| path("<svg path data>") \| circle([<n>px])` | Its text set along a path (`-og-text-path`): `none`, `path("<svg path data>")` in px from its content box, or `circle([<n>px])` centred in it. |
| `-og-text-path-offset:` | `<n>px \| <n>%` | Where its text's anchor (text-align) sits along its path: `<n>px` or `<n>%` of the path's length. |
| `-og-text-path-side:` | `left \| right` | Which side of its path the text stands on: right runs the path backwards (inside a circle). |
| `user-select:` | `auto \| none \| text \| all` | Whether its text and its subtree's can be selected and copied; auto takes its parent's (none at the top and under a control). |
| `customize` | `bool` | The player can move, scale and fade it in the HUD edit mode. |
| `customize-key` | `name` | With customize: its name in the saved HUD layout (default: its path). |
| `title` | `text` | Its tooltip, shown after the pointer rests on it (HTML `title`); also its description for a screen reader. |
| `popover` | `auto \| manual \| hint` | Shown over the page against the control that opened it, not in its place (HTML `popover`): auto closes on a press outside or Back and closes other auto ones; manual only when a control closes it; hint is a rich tooltip an `interestfor` control shows. |
| `popovertarget` | `name` | The id of the popover a press on it opens, closes or toggles (HTML `popovertarget`). |
| `popovertargetaction` | `toggle \| show \| hide` | What its press does to its popovertarget (default toggle). |
| `interestfor` | `name` | The id of the popover shown while the pointer rests on it or it holds the keyboard focus (HTML `interestfor`). |
| `contextmenu` | `name` | The id of the popover a secondary press or a long press on it opens at the pointer, its anchor that point (HTML's former `contextmenu`): a context menu. |
| `aria-description` | `text` | Read after its name (`aria-description`, or the text `aria-describedby` names). |
| `aria-valuemin` | `number` | Its range's low end for a screen reader (`aria-valuemin`). |
| `aria-valuemax` | `number` | Its range's high end for a screen reader (`aria-valuemax`). |
| `aria-valuenow` | `number` | Its value as a number for a screen reader (`aria-valuenow`). |
| `aria-valuetext` | `text` | Its value as a screen reader says it (`aria-valuetext`). |
| `aria-checked` | `bool` | Checked, for a screen reader (`aria-checked`). |
| `aria-selected` | `bool` | Selected, for a screen reader (`aria-selected`): a tab, an option, a row. |
| `aria-expanded` | `bool` | Expanded or collapsed (`aria-expanded`): a disclosure, a tree item, a menu button. |
| `aria-pressed` | `bool` | A toggle button's pressed state (`aria-pressed`): a chip, a filter. |
| `aria-busy` | `bool` | Busy (`aria-busy`): loading or working. |
| `aria-disabled` | `bool` | Disabled for a screen reader and inert (`aria-disabled`). |
| `aria-level` | `number` | Its level, 1 and up (`aria-level`): a heading's, a tree item's. |
| `aria-current` | `page \| step \| location \| date \| time \| true \| false` | The current item in a set (`aria-current`): a pagination's page, a breadcrumb's last, a wizard's step. |
| `data-og-instant` | `bool` | Its first entrance is settled (`data-og-instant`): the screen a game opens on, which the web draws behind its play gate. Later entrances play. |
| `-og-reduced-motion:` | `fade \| none \| keep` | Its reduced-motion fallback. |
| `data-hold` | `number` | How long a press is held before `onhold` fires, in ms (`data-hold`; default 500). While it is held a fill grows across the element from its start edge, drawn over its background and under its content and coloured by `::-og-hold { background }` (else the style's primary at a third); with `data-action`, holding that action holds it too. |
| `display:` | `block \| inline \| inline-block \| flow-root \| list-item \| flex \| inline-flex \| grid \| inline-grid \| table \| inline-table \| contents \| table-row-group \| table-header-group \| table-footer-group \| table-row \| table-cell \| table-caption \| table-column \| table-column-group` | Its CSS `display`. On a box its layout: `block` stacks its children top to bottom (the default), `flex` lays them out as a row (or `flex-direction`), `grid` in tracks, `table` (and its parts) as a table, `flowRoot` a block whose children's margins stay inside, `contents` no box at all (its children are its parent's). On any element `inline…` puts it in lines beside its inline-level siblings. |
| `vertical-align:` | `baseline \| top \| middle \| bottom \| text-top \| text-bottom \| sub \| super` | CSS `vertical-align`: where an inline-level element sits in its line, and a table cell's content in its row (`middle` by default). |
| `caption-side:` | `top \| bottom` | CSS `caption-side`: a table caption above (default) or below its table. |
| `list-style-image:` | `none \| url(<image>)` | CSS `list-style-image`: an inherited image URL replacing a list item's marker, or none. Only list items draw the image. |
| `-og-emitter:` | `[<preset>] [<rate>] [burst <n>] [for <time>] [life <time>] [<n>px/s …] … \| none` | `-og-emitter: [<preset>] [<rate>] [burst <n>] [for <time>] [life <time>] [<speed>px/s [<speed>px/s]] [up\|down\|<angle>deg] [square\|circle\|diamond] [<size>px] [<colour>…]`: particles from the element while it stands, drawn over it; `none` stops. A preset (sparkles, confetti, embers, snow, bubbles, dust, sparks, fireworks) gives the look, rate and the style's colours; the later words override it (`.rare { -og-emitter: sparkles; }`, `.win { -og-emitter: confetti 70 burst 110 for 1.4s; }`, `#Gem { -og-emitter: 12 diamond 6px #ffd166 #fff; }`). `burst` throws that many at once when the element first stands, `for` stops the rate that long after, `life` is each particle's life and a speed (one, or a range) its travel a second. Silent under reduced motion. |
| `-og-meta:` | `<n>px \| <n>%` | `-og-meta: vignette [<colour>] \| tint <colour> \| material <material>`: a full-screen overlay drawn with the element, never taking input, its strength the element's `opacity` (`#Hurt { -og-meta: vignette #a00; opacity: var(--hit); }`); `material` composites the frame under it through a material written as `-og-material` writes one (`-og-meta: material ripple(6 48 60)` for sanity, `-og-meta: material "fx/static.ogt" var(--k)`). A pulse is a CSS animation of its opacity. |
| `draggable` | `bool` | HTML `draggable`: a drag that starts on it moves it with the pointer and returns it home on release, firing `ondragstart`, `ondragend`, and `ondragover`/`ondrop` on the element it is over (any element with one of those handlers takes drops). |
| `data-drag-type` | `text` | What a draggable element is to drop targets (`data-drag-type="weapon"`), as a drag's data type is in HTML. |
| `data-accept` | `text` | The drag types a drop target takes, space-separated (`data-accept="weapon shield"`; default any). |
| `background-image:` | `<image> [<position> [/ <size>]] [<repeat>], … [<colour>], an <image> url(<icon or texture>) or a gradient` | CSS `background` or `background-image` layers over the background colour, the first on top (CSS Backgrounds 3): images `url(<name>)` (an icon of the UI's atlas, as `<image href>` names one) or `url(<handle>)` (a texture, `og.load.texture`), each `<image> [<position> [/ <size>]] [<repeat>]` (`background: url(coin) right 8px bottom 8px / 24px no-repeat, url(sky) center / cover, #123`), and gradients: `linear-gradient()`, `radial-gradient()`, `conic-gradient()` and their `repeating-` forms, which fill the box (up to 8 stops each, mixed in OKLab). The element's `border-radius` and `clip-path` cut every layer. |
| `background-size:` | `cover \| contain \| [<n>px \| <n>% \| auto]{1,2}, …` | CSS `background-size` of each image layer, a comma list repeated over the layers: `cover`, `contain`, or a width and height (`<n>px`, `<n>%` of the box, `auto` from the image's ratio). |
| `background-position:` | `[left \| center \| right \| top \| bottom \| <n>px \| <n>%]{1,4}, …` | CSS `background-position` of each image layer, a comma list: one to four of `left`, `center`, `right`, `top`, `bottom`, `<n>px` and `<n>%` (`right 10px bottom 20%` from those edges). |
| `background-repeat:` | `repeat-x \| repeat-y \| [repeat \| no-repeat \| space \| round]{1,2}, …` | CSS `background-repeat` of each image layer, a comma list: `repeat-x`, `repeat-y`, or one or two of `repeat`, `no-repeat`, `space` (whole images edge to edge) and `round` (rescaled to a whole number). |
| `mask-image:` | `linear-gradient(…) \| radial-gradient(…) \| conic-gradient(…) (repeating- too)` | CSS `mask-image` or the `mask` shorthand: layers of `url(…)` and gradients whose alpha (or luminance) cuts the element and its subtree, each sized, placed and combined as the `mask-*` longhands say (`mask-image: linear-gradient(#000 70%, transparent)`). |
| `background-origin:` | `<n>px \| <n>%` | CSS `background-origin` of each image layer, a comma list: the box it is sized and placed in, `border-box`, `padding-box` (default) or `content-box`. |
| `background-clip:` | `<n>px \| <n>%` | CSS `background-clip` of each layer, a comma list: where it is painted, `border-box` (default), `padding-box`, `content-box` or `text` (the element's glyphs); the last layer's cuts the colour. |
| `background-attachment:` | `<n>px \| <n>%` | CSS `background-attachment` of each layer: `scroll` (default), `fixed` (placed in the viewport) or `local` (scrolls with a scroller's content). |
| `background-blend-mode:` | `<n>px \| <n>%` | CSS `background-blend-mode` of each layer, a comma list of `mix-blend-mode`'s modes: how it blends with the layers and colour under it. |
| `mask-size:` | `<n>px \| <n>%` | CSS `mask-size` of each mask layer, as `background-size`. |
| `mask-position:` | `<n>px \| <n>%` | CSS `mask-position` of each mask layer, as `background-position`. |
| `mask-repeat:` | `<n>px \| <n>%` | CSS `mask-repeat` of each mask layer, as `background-repeat`. |
| `mask-origin:` | `<n>px \| <n>%` | CSS `mask-origin` of each mask layer: `border-box` (default), `padding-box` or `content-box`. |
| `mask-clip:` | `<n>px \| <n>%` | CSS `mask-clip` of each mask layer: `border-box` (default), `padding-box`, `content-box` or `no-clip`. |
| `mask-mode:` | `<n>px \| <n>%` | CSS `mask-mode` of each mask layer: `match-source` or `alpha` (its alpha), or `luminance`. |
| `mask-composite:` | `<n>px \| <n>%` | CSS `mask-composite` of each mask layer with those under it: `add`, `subtract`, `intersect` or `exclude`. |
| `border-image-source:` | `<n>px \| <n>%` | CSS `border-image-source`: `url(…)`, a gradient or `none`. |
| `border-image-slice:` | `<n>px \| <n>%` | CSS `border-image-slice`: one to four numbers (image px) or percentages, and `fill`. |
| `border-image-width:` | `<n>px` | CSS `border-image-width`: one to four of `<n>px`, `<n>%`, a number (times the border width) or `auto`. |
| `border-image-outset:` | `<n>px \| <n>%` | CSS `border-image-outset`: one to four of `<n>px` or a number (times the border width). |
| `border-image-repeat:` | `<n>px \| <n>%` | CSS `border-image-repeat`: one or two of `stretch`, `repeat`, `round`, `space`. |
| `mask-border-source:` | `<n>px \| <n>%` | CSS `mask-border-source`: `url(…)`, a gradient or `none`. |
| `mask-border-slice:` | `<n>px \| <n>%` | CSS `mask-border-slice`, as `border-image-slice`. |
| `mask-border-width:` | `<n>px \| <n>%` | CSS `mask-border-width`, as `border-image-width`. |
| `mask-border-outset:` | `<n>px \| <n>%` | CSS `mask-border-outset`, as `border-image-outset`. |
| `mask-border-repeat:` | `<n>px \| <n>%` | CSS `mask-border-repeat`, as `border-image-repeat`. |
| `mask-border-mode:` | `<n>px \| <n>%` | CSS `mask-border-mode`: `alpha` (default) or `luminance`. |
| `backdrop-filter:` | `blur(<n>px) brightness(<n>) contrast(<n>) drop-shadow([<colour>] <x> <y> [<blur>]) grayscale(<n>) hue-rotate(<angle>) invert(<n>) opacity(<n>) saturate(<n>) sepia(<n>) … \| none` | CSS `backdrop-filter`: what is really under the element (the screen, or the group it is drawn in) through `filter`'s functions and `url(#id)` in order, laid in its border-box shape under its background. |
| `filter:` | `blur(<n>px) brightness(<n>) contrast(<n>) drop-shadow([<colour>] <x> <y> [<blur>]) grayscale(<n>) hue-rotate(<angle>) invert(<n>) opacity(<n>) saturate(<n>) sepia(<n>) … \| none` | CSS `filter`: `blur()`, `brightness()`, `contrast()`, `drop-shadow()` (of the alpha, not the box), `grayscale()`, `hue-rotate()`, `invert()`, `opacity()`, `saturate()` and `sepia()` on the element and its subtree, applied in the order written (`filter: drop-shadow(0 4px 6px #0008) saturate(1.4)`). |
| `mix-blend-mode:` | `normal \| multiply \| screen \| plus-lighter \| overlay \| soft-light \| color-dodge` | CSS `mix-blend-mode`: any of CSS Compositing's modes (normal, multiply, screen, overlay, darken, lighten, color-dodge, color-burn, hard-light, soft-light, difference, exclusion, hue, saturation, color, luminosity) or plus-lighter. The element and its subtree composite as one group over what is under it. |
| `isolation:` | `auto \| isolate` | CSS `isolation`, `auto` or `isolate`: `isolate` makes the element a group its descendants' blends blend within, over a transparent backdrop. |
| `text-shadow:` | `<x> <y> [<blur>] [<colour>], … \| none` | CSS `text-shadow: <x> <y> [<blur>] [<colour>], …` on the element's text. |
| `clip-path:` | `[inset(…) \| rect(…) \| xywh(…) \| circle(…) \| ellipse(…) \| polygon(…) \| path("…") \| shape(…)] [<geometry-box>] \| <geometry-box> \| url(#id) \| none` | CSS `clip-path`: a basic shape (`inset()`, `rect()`, `xywh()`, `circle()`, `ellipse()`, `polygon()`, `path()` or `shape()`) over a geometry box (`border-box` by default, `padding-box`, `content-box`, `margin-box`; `fill-box` is the content box, `stroke-box` and `view-box` the border box), a geometry box alone (that box with its corners), or `url(#id)` of a `<clipPath>` in the document. It cuts everything the element draws (shadows, background, borders) and its children, by the shape's coverage with either fill rule. |
| `-og-material:` | `glass <regular\|thick> <lift\|float> \| <material>(<n> …[, <colour>]) \| "<ui-material asset>" <n> … \| none` | `-og-material: glass <regular\|thick> <lift\|float>` supplies glass tint, edge, elevation and reduced-transparency fallback; standard `backdrop-filter` supplies its blur and colour filters. Alternatively `<name>(<n> …[, <colour>])` composites through a built-in shader (dissolve, shine, hologram, glitch, ripple, pixelate, color-adjust, outline, blur), or `"<asset>" [<n> …]` loads a `type ui-material` asset with four numbers per parameter. `none` removes the material. Values may substitute author variables, but the engine reads no named custom property. Extension reason: standard CSS has no shader-material or native glass-weight/elevation primitive. |
| `-og-sound:` | `none \| [ <event> [ <url>+ [volume(<number> <number>?)]? [pitch(<number> <number>?)]? \| none ] ]#` | `-og-sound: none \| [ <event> [ <url>+ [volume(<number> <number>?)]? [pitch(<number> <number>?)]? \| none ] ]#`. Each event takes up to four URLs, selected without immediate repeats from the checkpointed engine UI random stream. Volume is an ordered 0..1 range, pitch an ordered positive range. URLs resolve beside the CSS source. `hover none` silences one event. Extension reason: standard CSS has no interaction audio playback. Haptics remain in the og.* API. |
| `scroll-snap-type:` | `none \| x \| y \| block \| inline \| both [mandatory \| proximity]` | CSS `scroll-snap-type: x \| y \| both [mandatory \| proximity]` on a scroll container: a drag, fling or wheel settles on the snap position of the area nearest where it would stop (`proximity`: only within a third of the view). |
| `scroll-snap-align:` | `none \| start \| center \| end [<inline>]` | CSS `scroll-snap-align: start \| center \| end` (block, then inline) of an area inside a snapping scroll container: the edge or centre that lines up with its view's. |
| `overscroll-behavior-x:` | `auto \| contain \| none` | CSS `overscroll-behavior-x` (and `overscroll-behavior`'s first, `-inline`) of a scroll container: `contain` or `none` keeps a wheel, drag or fling that reaches its end from scrolling the container around it; `auto` hands the rest on. |
| `overscroll-behavior-y:` | `auto \| contain \| none` | CSS `overscroll-behavior-y` (and `overscroll-behavior`'s second, `-block`): as `overscrollBehaviorX`, vertically. |
| `scroll-behavior:` | `auto \| smooth` | CSS `scroll-behavior: auto \| smooth` of a scroll container: smooth glides a scroll into view (a focus move, a pad's step, og.ui.scrollIntoView) there along the momentum curve; the player's reduced motion jumps. |
| `scroll-margin-top:` | `<length>` | CSS `scroll-margin-top` (`scroll-margin`, `-block`, `-block-start`): the element's box grown by it where a scroll into view or a snap position lines it up. |
| `scroll-margin-right:` | `<length>` | CSS `scroll-margin-right` (`scroll-margin`, `-inline`, `-inline-end`). |
| `scroll-margin-bottom:` | `<length>` | CSS `scroll-margin-bottom` (`scroll-margin`, `-block`, `-block-end`). |
| `scroll-margin-left:` | `<length>` | CSS `scroll-margin-left` (`scroll-margin`, `-inline`, `-inline-start`). |
| `scroll-padding-top:` | `auto \| <length> \| <n>%` | CSS `scroll-padding-top` (`scroll-padding`, `-block`, `-block-start`) of a scroll container: px or % of its view inset from its top where a scroll into view or a snap position lines things up (under a sticky header). |
| `scroll-padding-right:` | `auto \| <length> \| <n>%` | CSS `scroll-padding-right` (`scroll-padding`, `-inline`, `-inline-end`). |
| `scroll-padding-bottom:` | `auto \| <length> \| <n>%` | CSS `scroll-padding-bottom` (`scroll-padding`, `-block`, `-block-end`). |
| `scroll-padding-left:` | `auto \| <length> \| <n>%` | CSS `scroll-padding-left` (`scroll-padding`, `-inline`, `-inline-start`). |
| `scroll-snap-stop:` | `normal \| always` | CSS `scroll-snap-stop: normal \| always` of a snap area: always stops a fling or a settling wheel at it rather than passing it for a farther one. |
| `scrollbar-color:` | `auto \| <thumb colour> <track colour>` | CSS `scrollbar-color: auto \| <thumb> <track>` of a scroll container: the scrollbars' thumb colour and a track drawn under it; inherited. |
| `scrollbar-gutter:` | `auto \| stable [both-edges]` | CSS `scrollbar-gutter: auto \| stable [both-edges]` of a vertical scroll container: stable keeps the scrollbar's room on its end side whether or not it overflows (both-edges on its start side too), so its content does not shift when it starts to. |
| `right:` | `<n>px \| <n>%` | CSS `right` / `inset-inline-end` as written (see insetTop). |
| `left:` | `<n>px \| <n>%` | CSS `left` / `inset-inline-start` as written (see insetTop). |
| `view-transition-name:` | `none \| auto \| match-element \| <name>` | CSS `view-transition-name`: the shared element's name in a view transition (empty: none; `auto` and `match-element` name it by its identity). Its old and new selves morph as one `::view-transition-group(name)`. |
| `view-transition-class:` | `none \| <name>+` | CSS `view-transition-class`: the classes its view-transition group is styled by (`::view-transition-group(*.card)`), space-separated. |
| `view-transition-group:` | `normal \| contain \| nearest \| <name>` | CSS `view-transition-group`: where its view-transition group nests (empty: `normal`, in the nearest shared ancestor's that is not normal; `contain`: as normal, and its own nest in it; `nearest`: in the nearest shared ancestor's; a name: in that ancestor's). A nested group moves with its parent, inside its `::view-transition-group-children`. |
| `anchor-name:` | `<n>px \| <n>%` | CSS `anchor-name`: the dashed names (`--menu`) other elements anchor to it by. |
| `position-anchor:` | `<n>px \| <n>%` | CSS `position-anchor`: its default anchor's dashed name (empty: its implicit anchor, a popover's invoker). |
| `position-area:` | `<n>px \| <n>%` | CSS `position-area`: the cell or span of the 3×3 grid its anchor cuts its containing block into (`bottom center`, `span-right top`, `block-end`). |
| `position-try-fallbacks:` | `<n>px \| <n>%` | CSS `position-try-fallbacks`: flips (`flip-block`, `flip-inline`, `flip-start`), `@position-try` names and position-areas tried in order while it overflows. |
| `position-try-order:` | `<n>px \| <n>%` | CSS `position-try-order`: normal, most-width or most-height. |
| `position-visibility:` | `<n>px \| <n>%` | CSS `position-visibility`: always, anchors-visible (the default) or no-overflow. |
| `column-count:` | `<n>px \| <n>%` | CSS `column-count`: `auto` or how many columns its flow lays out in (a multi-column container). |
| `column-width:` | `<n>px \| <n>%` | CSS `column-width`: `auto` or the least width of a column; with `column-count`, as many columns as fit, at most that count. |
| `float:` | `<n>px \| <n>%` | CSS `float` (`none`, `left`, `right`, `inline-start`, `inline-end`): in a block's flow it leaves the flow for that side of its containing block, shrinking to fit; the lines beside it shorten and boxes that start a formatting context of their own keep clear of it. |
| `clear:` | `<n>px \| <n>%` | CSS `clear` (`none`, `left`, `right`, `both`, `inline-start`, `inline-end`): it is placed below the earlier floats on those sides. |
| `column-span:` | `<n>px \| <n>%` | CSS `column-span: none \| all`: in a multi-column container, `all` spans every column and starts a new set of columns below it. |
| `column-fill:` | `<n>px \| <n>%` | CSS `column-fill`: `balance` (and `balance-all`) evens the columns' heights; `auto` fills them in turn to the container's definite height. |
| `column-rule-width:` | `<n>px \| <n>%` | CSS `column-rule-width`: a px length, `thin`, `medium` or `thick`. |
| `column-rule-style:` | `<n>px \| <n>%` | CSS `column-rule-style`: a line style (`none`, `solid`, `dashed`, `dotted`, `double`…). |
| `column-rule-color:` | `<n>px \| <n>%` | CSS `column-rule-color` (`currentcolor`: its text colour). |
| `break-before:` | `<n>px \| <n>%` | CSS `break-before` (and `page-break-before`): `column`, `page` or `always` starts a new column in a multi-column flow. |
| `break-after:` | `<n>px \| <n>%` | CSS `break-after` (and `page-break-after`): `column`, `page` or `always` puts what follows in a new column. |
| `break-inside:` | `<n>px \| <n>%` | CSS `break-inside` (and `page-break-inside`): `avoid` keeps a text's lines in one column. |
| `contain:` | `none \| strict \| content \| [size \| inline-size] \|\| layout \|\| style \|\| paint` | CSS `contain`: `size` (or `inline-size`) sizes it as if empty (or as `contain-intrinsic-size` says), `layout` and `paint` make it a formatting context and clip what it draws to its box, `style` keeps its counters and quotes inside it; `strict` is all four, `content` all but size. |
| `content-visibility:` | `visible \| auto \| hidden` | CSS `content-visibility`: `hidden` skips its contents (not laid out, drawn, hit or in the accessibility tree); `auto` skips them while it is far from the screen and holds no focus, sized by `contain-intrinsic-size`; both contain its layout, style and paint. |
| `contain-intrinsic-width:` | `none \| <n>px \| auto <n>px \| auto none` | CSS `contain-intrinsic-width` (and `contain-intrinsic-size`): the content width size containment gives it, `none` (0) or a px length; `auto` uses its last laid-out width while its contents are skipped. |
| `contain-intrinsic-height:` | `none \| <n>px \| auto <n>px \| auto none` | CSS `contain-intrinsic-height` (and `contain-intrinsic-size`): as `containIntrinsicWidth`, down. |
| `resize:` | `none \| both \| horizontal \| vertical \| block \| inline` | CSS `resize` on an element whose overflow is not visible: `both`, `horizontal` (`inline`) or `vertical` (`block`) puts a grab handle on its bottom end corner whose drag sets its width or height. |
| `will-change:` | `auto \| <property>, …` | CSS `will-change`: properties it is about to change; one that makes a group (transform, opacity, filter…) draws it on a layer of its own. |

Style interaction with `#Buy:hover { scale: 1.04 }`. The screens guide lists state selectors and their scoped cascade. Bind game state with `class:active={$selected}` and style `.active`. Custom properties inherit computed values from the mounted parent; `var(--name, fallback)` resolves after specificity and `!important`. CSS motion uses shared timing values; markup contains no state blocks or timelines. `transition` and its longhands (`transition-property`, `-duration`, `-timing-function`, `-delay`) and `animation` and its longhands (`animation-name`, `-duration`, `-timing-function`, `-delay`, `-iteration-count`, `-direction`, `-fill-mode`, `-play-state`) follow CSS Transitions and Animations Level 1 over the animatable values: opacity, translate, scale, rotate, border-radius, background, color and z-index (keyframes animate the scalar ones). Other animatable properties, including transform with skew functions, interpolate their computed values.

### Exposed widget parts

These declarations also construct widget part identities and are exported by documentSchema and dump_ui. Classes (such as primary), ARIA attributes (such as aria-selected), and author-declared custom parts are separate from state pseudo-classes. Native API styling hosts and their markup counterparts are listed separately where their names differ.

| Host | Part | State pseudo-classes |
|---|---|---|
| `button` | `base` | `:enabled`, `:disabled`, `:hover`, `:active`, `:focus-visible` |
| `button` | `focus` | `:enabled`, `:disabled`, `:hover`, `:active`, `:focus-visible` |
| `input[type=checkbox][switch]` | `row` | `:enabled`, `:disabled`, `:hover`, `:active`, `:focus-visible`, `:checked` |
| `input[type=checkbox][switch]` | `track` | `:enabled`, `:disabled`, `:hover`, `:active`, `:focus-visible`, `:checked` |
| `input[type=checkbox][switch]` | `knob` | `:enabled`, `:checked` |
| `input[type=checkbox]` | `row` | `:enabled`, `:disabled`, `:hover`, `:active`, `:focus-visible`, `:checked`, `:invalid` |
| `input[type=checkbox]` | `box` | `:enabled`, `:disabled`, `:hover`, `:active`, `:focus-visible`, `:checked`, `:invalid` |
| `input[type=checkbox]` | `mark` | `:enabled` |
| `input[type=radio]` | `option` | `:enabled`, `:disabled`, `:hover`, `:active`, `:focus-visible`, `:checked` |
| `input[type=radio]` | `card` | `:enabled`, `:disabled`, `:hover`, `:active`, `:focus-visible`, `:checked` |
| `input[type=radio]` | `ring` | `:enabled`, `:disabled`, `:hover`, `:active`, `:focus-visible`, `:checked` |
| `input[type=radio]` | `dot` | `:enabled` |
| `input[type=radio]` | `chosen` | `:enabled` |
| `tabs` | `track` | `:enabled`, `:disabled` |
| `tabs` | `indicator` | `:enabled` |
| `tabs` | `tab` | `:enabled`, `:disabled`, `:hover`, `:active`, `:focus-visible`, `:checked` |
| `tabs` | `focus` | `:enabled`, `:disabled`, `:hover`, `:active`, `:focus-visible`, `:checked` |
| `input[type=range]` | `track` | `:enabled` |
| `input[type=range]` | `fill` | `:enabled` |
| `input[type=range]` | `tick` | `:enabled` |
| `input[type=range]` | `core` | `:enabled` |
| `input[type=range]` | `bubble` | `:enabled` |
| `input[type=range]` | `reach` | `:enabled`, `:disabled`, `:hover`, `:active`, `:focus-visible` |
| `input[type=range]` | `knob` | `:enabled`, `:disabled`, `:hover`, `:active`, `:focus-visible` |
| `select` | `trigger` | `:enabled`, `:disabled`, `:hover`, `:active`, `:focus-visible`, `:open`, `:invalid` |
| `select` | `list` | `:enabled` |
| `select` | `option` | `:enabled`, `:disabled`, `:hover`, `:active`, `:focus-visible` |
| `select` | `group` | `:enabled` |
| `details` | `root` | `:enabled` |
| `details` | `summary` | `:enabled`, `:disabled`, `:hover`, `:active`, `:focus-visible`, `:open` |
| `details` | `content` | `:enabled` |
| `stepper` | `frame` | `:enabled` |
| `stepper` | `value` | `:enabled`, `:disabled`, `:hover`, `:active`, `:focus-visible` |
| `stepper` | `button` | `:enabled`, `:disabled`, `:hover`, `:active`, `:focus-visible` |
| `keyboard` | `panel` | `:enabled` |
| `keyboard` | `preview` | `:enabled` |
| `keyboard` | `candidates` | `:enabled` |
| `keyboard` | `candidate` | `:enabled`, `:disabled`, `:hover`, `:active`, `:focus-visible` |
| `keyboard` | `key` | `:enabled`, `:disabled`, `:hover`, `:active`, `:focus-visible` |
| `input` | `frame` | `:enabled`, `:disabled`, `:focus-visible`, `:invalid` |
| `input` | `text` | `:enabled`, `:disabled`, `:focus-visible`, `:invalid` |
| `input` | `placeholder` | `:enabled`, `:disabled`, `:focus-visible`, `:invalid` |
| `input` | `reveal` | `:enabled`, `:disabled`, `:hover`, `:active`, `:focus-visible`, `:invalid` |
| `input` | `reveal-icon` | `:enabled`, `:disabled`, `:focus-visible`, `:invalid` |
| `input` | `clear` | `:enabled`, `:disabled`, `:hover`, `:active`, `:focus-visible`, `:invalid` |
| `input` | `clear-icon` | `:enabled`, `:disabled`, `:focus-visible`, `:invalid` |
| `textarea` | `frame` | `:enabled`, `:disabled`, `:focus-visible`, `:invalid` |
| `textarea` | `text` | `:enabled`, `:disabled`, `:focus-visible`, `:invalid` |
| `textarea` | `placeholder` | `:enabled`, `:disabled`, `:focus-visible`, `:invalid` |
| `textarea` | `selection` | `:enabled` |
| `panel` | `frame` | `:enabled` |
| `slot` | `frame` | `:enabled`, `:hover`, `:active`, `:focus-visible` |
| `tooltip` | `frame` | `:enabled` |
| `hud` | `frame` | `:enabled` |
| `progress` | `track` | `:enabled` |
| `progress` | `fill` | `:enabled` |
| `meter` | `track` | `:enabled` |
| `meter` | `fill` | `:enabled` |
| `meter` | `trail` | `:enabled` |
| `meter` | `separator` | `:enabled` |
| `meter` | `label` | `:enabled` |
| `chip` | `base` | `:enabled`, `:disabled`, `:hover`, `:active`, `:focus-visible` |
| `badge` | `base` | `:enabled` |
| `skeleton` | `frame` | `:enabled` |
| `skeleton` | `sheen` | `:enabled` |
| `skeleton` | `line` | `:enabled` |
| `spinner` | `base` | `:enabled` |
| `spinner` | `dot` | `:enabled` |
| `og-icon` | `image` | `:enabled` |
| `og-icon` | `secondary` | `:enabled` |
| `artslot` | `icon` | `:enabled` |
| `artslot` | `placeholder` | `:enabled` |
| `artslot` | `label` | `:enabled` |
| `dialog` | `backdrop` | `:enabled` |
| `dialog` | `panel` | `:enabled` |
| `popover` | `arrow` | `:enabled` |
| `toast` | `indicator` | `:enabled` |
| `og-spotlight` | `edge` | `:enabled` |
| `cooldown` | `track` | `:enabled` |
| `cooldown` | `sweep` | `:enabled` |
| `cooldown` | `mask` | `:enabled` |
| `cooldown` | `core` | `:enabled`, `:active` |
| `cooldown` | `icon` | `:enabled` |
| `cooldown` | `label` | `:enabled` |
| `minimap` | `frame` | `:enabled` |
| `minimap` | `image` | `:enabled` |
| `minimap` | `markerOutline` | `:enabled` |
| `minimap` | `marker` | `:enabled` |
| `minimap` | `north` | `:enabled` |
| `dialogue` | `portraitIcon` | `:enabled` |
| `dialogue` | `initial` | `:enabled` |
| `dialogue` | `continue` | `:enabled` |
| `joystick` | `base` | `:enabled`, `:disabled` |
| `joystick` | `rim` | `:enabled`, `:disabled`, `:active` |
| `joystick` | `knob` | `:enabled`, `:disabled`, `:active` |
| `joystick` | `direction` | `:enabled`, `:disabled`, `:active` |
| `dpad` | `base` | `:enabled`, `:disabled` |
| `dpad` | `indicator` | `:enabled`, `:active` |
| `dpad` | `chevron` | `:enabled`, `:active` |
| `actionbutton` | `face` | `:enabled`, `:disabled`, `:active` |
| `actionbutton` | `icon` | `:enabled`, `:disabled`, `:active` |
| `actionbutton` | `track` | `:enabled` |
| `actionbutton` | `sweep` | `:enabled` |
| `actionbutton` | `mask` | `:enabled` |
| `actionbutton` | `core` | `:enabled`, `:disabled`, `:active` |
| `actionbutton` | `label` | `:enabled`, `:disabled`, `:active` |
| `heading` | `flourish` | `:enabled` |
| `scroll` | `bar` | `:enabled` |
| `textarea` | `caret` | `:enabled` |
| `textarea` | `composition` | `:enabled` |
| `input` | `selection` | `:enabled` |
| `input` | `caret` | `:enabled` |
| `input` | `composition` | `:enabled` |
| `input` | `focus` | `:enabled`, `:disabled`, `:hover`, `:active`, `:focus-visible`, `:invalid` |
| `input[type=checkbox]` | `focus` | `:enabled`, `:disabled`, `:hover`, `:active`, `:focus-visible`, `:checked`, `:invalid` |
| `input[type=checkbox][switch]` | `focus` | `:enabled`, `:disabled`, `:hover`, `:active`, `:focus-visible`, `:checked` |
| `input[type=radio]` | `focus` | `:enabled`, `:disabled`, `:hover`, `:active`, `:focus-visible`, `:checked` |
| `input[type=range]` | `focus` | `:enabled`, `:disabled`, `:hover`, `:active`, `:focus-visible` |
| `textarea` | `focus` | `:enabled`, `:disabled`, `:focus-visible`, `:invalid` |
| `select` | `focus` | `:enabled`, `:disabled`, `:hover`, `:active`, `:focus-visible`, `:open`, `:invalid` |
| `details` | `focus` | `:enabled`, `:disabled`, `:hover`, `:active`, `:focus-visible`, `:open` |
| `stepper` | `focus` | `:enabled`, `:disabled`, `:hover`, `:active`, `:focus-visible` |
| `keyboard` | `focus` | `:enabled`, `:disabled`, `:hover`, `:active`, `:focus-visible` |
| `slot` | `focus` | `:enabled`, `:hover`, `:active`, `:focus-visible` |
| `art` | `icon` | `:enabled` |
| `art` | `placeholder` | `:enabled` |
| `art` | `label` | `:enabled` |
| `h1` | `flourish` | `:enabled` |
| `h2` | `flourish` | `:enabled` |
| `h3` | `flourish` | `:enabled` |
| `h4` | `flourish` | `:enabled` |
| `h5` | `flourish` | `:enabled` |
| `h6` | `flourish` | `:enabled` |
| `input[type=number]` | `frame` | `:enabled` |
| `input[type=number]` | `value` | `:enabled`, `:disabled`, `:hover`, `:active`, `:focus-visible` |
| `input[type=number]` | `button` | `:enabled`, `:disabled`, `:hover`, `:active`, `:focus-visible` |

### CSS value functions

| Function | Grammar | Reason and behavior |
|---|---|---|
| `-og-spring` | `-og-spring(<time [0,∞]> [, <number>])` | Transition timing with a perceptual duration (ms or s) and optional finite bounce (default 0). Bounce is clamped to -0.95..1 by the spring solver. Extension reason: CSS linear() cannot preserve velocity when a transition is interrupted. Springs time transitions, not keyframe animations. |

<a id="guide-components-2"></a>
### `components §2` `<column>` (`column`): Children top to bottom

Children top to bottom. A container with nested children: `<column id="Name">…</column>`.

Its own props, beside every element's:

| Write | Value | What |
|---|---|---|
| `flex-direction:` | `row \| column \| row-reverse \| column-reverse` | Lays a row's or a column's children out the other way (an `at` block's: a phone in landscape puts the actions beside the content). |
| `-og-frame:` | `rive("<file>.riv#<Artboard>")` | The Rive artboard it plays in its box, `<file>.riv#<Artboard>` relative to assets/ (`-og-frame: rive("ui/chest.riv#Chest")`, docs/ui.md §3.3): loaded with its document, advanced on the tick, drawn over its background and saved with the UI's state; its drawn objects are elements for dump_ui, describe and pick. |

<a id="guide-components-3"></a>
### `components §3` `<row>` (`row`): Children left to right (mirrored right to left)

Children left to right (mirrored right to left). A container with nested children: `<row id="Name">…</row>`.

Its own props, beside every element's:

| Write | Value | What |
|---|---|---|
| `flex-direction:` | `row \| column \| row-reverse \| column-reverse` | Lays a row's or a column's children out the other way (an `at` block's: a phone in landscape puts the actions beside the content). |
| `-og-frame:` | `rive("<file>.riv#<Artboard>")` | The Rive artboard it plays in its box, `<file>.riv#<Artboard>` relative to assets/ (`-og-frame: rive("ui/chest.riv#Chest")`, docs/ui.md §3.3): loaded with its document, advanced on the tick, drawn over its background and saved with the UI's state; its drawn objects are elements for dump_ui, describe and pick. |

<a id="guide-components-4"></a>
### `components §4` `<stack>` (`stack`): Children overlaid in one rectangle, each placed by its insets

Children overlaid in one rectangle, each placed by its insets. A container with nested children: `<stack id="Name">…</stack>`.

Its own props, beside every element's:

| Write | Value | What |
|---|---|---|
| `-og-frame:` | `rive("<file>.riv#<Artboard>")` | The Rive artboard it plays in its box, `<file>.riv#<Artboard>` relative to assets/ (`-og-frame: rive("ui/chest.riv#Chest")`, docs/ui.md §3.3): loaded with its document, advanced on the tick, drawn over its background and saved with the UI's state; its drawn objects are elements for dump_ui, describe and pick. |

<a id="guide-components-5"></a>
### `components §5` `<grid>` (`grid`): Children in tracks and cells (columns, rows, areas)

Children in tracks and cells (columns, rows, areas). A container with nested children: `<grid id="Name">…</grid>`.

Its own props, beside every element's:

| Write | Value | What |
|---|---|---|
| `-og-frame:` | `rive("<file>.riv#<Artboard>")` | The Rive artboard it plays in its box, `<file>.riv#<Artboard>` relative to assets/ (`-og-frame: rive("ui/chest.riv#Chest")`, docs/ui.md §3.3): loaded with its document, advanced on the tick, drawn over its background and saved with the UI's state; its drawn objects are elements for dump_ui, describe and pick. |

<a id="guide-components-6"></a>
### `components §6` `<panel>` (`panel`): A column on the style's panel surface (its card frame)

A column on the style's panel surface (its card frame). A container with nested children: `<panel id="Name">…</panel>`.

Its own props, beside every element's:

| Write | Value | What |
|---|---|---|
| `-og-frame:` | `rive("<file>.riv#<Artboard>")` | The Rive artboard it plays in its box, `<file>.riv#<Artboard>` relative to assets/ (`-og-frame: rive("ui/chest.riv#Chest")`, docs/ui.md §3.3): loaded with its document, advanced on the tick, drawn over its background and saved with the UI's state; its drawn objects are elements for dump_ui, describe and pick. |
| `world` | `bool` | `<panel world src={$target} entity={$e} size="1.2 0.8" rotation="0 30 0">`: a world panel, its subtree drawn into the render target `src` at its CSS width and height, shown on a quad `size` world units across at the entity (or `at="x y z"`) turned by `rotation` (degrees about x, y, z); the pointer's ray reaches its controls. |
| `world-size` | `text` | A world panel's quad, `width height` world units (default 1 1). |
| `world-rotation` | `text` | A world panel's turn, degrees about x, y and z (`0 30 0`); facing +z unturned. |
| `entity` | `number` | The entity it follows, by its bits (`entity={u.entity}`): its world pose as the host hands it in. |
| `world-at` | `text` | `at="x y z"`: a world point it follows, without an entity. |
| `viewport-src` | `number` | The render target its camera draws into (`src={$target}`, the target's colour texture handle). |

<a id="guide-components-7"></a>
### `components §7` `<scroll>` (`scroll`): A column that scrolls its content

A column that scrolls its content. A container with nested children: `<scroll id="Name">…</scroll>`.

Its own props, beside every element's:

| Write | Value | What |
|---|---|---|
| `-og-frame:` | `rive("<file>.riv#<Artboard>")` | The Rive artboard it plays in its box, `<file>.riv#<Artboard>` relative to assets/ (`-og-frame: rive("ui/chest.riv#Chest")`, docs/ui.md §3.3): loaded with its document, advanced on the tick, drawn over its background and saved with the UI's state; its drawn objects are elements for dump_ui, describe and pick. |
| `-og-scroll-fade:` | `<n>px` | How far, CSS px, its edges fade where its content runs on past them (a cut row reads as more, not as a clip); 0: none. |
| `-og-scroll-fade-into:` | `var(--<role>) \| #rrggbb[aa]` | The colour its fade runs into: the ground it sits on (default panel). |

<a id="guide-components-8"></a>
### `components §8` `<list>` (`list`): A virtualised list: its `for` builds only the rows in view, each a selectable row

A virtualised list: its `for` builds only the rows in view, each a selectable row. A container with nested children: `<list id="Name">…</list>`.

Its own props, beside every element's:

| Write | Value | What |
|---|---|---|
| `row-height` | `number` | Every row's height, CSS px. |

<a id="guide-components-9"></a>
### `components §9` `<cells>` (`cells`): A virtualised grid of equal cells: its `for` builds only the cells in view

A virtualised grid of equal cells: its `for` builds only the cells in view. A container with nested children: `<cells id="Name">…</cells>`.

Its own props, beside every element's:

| Write | Value | What |
|---|---|---|
| `cell-width` | `number` | A cell's width, CSS px. |
| `cell-height` | `number` | A cell's height, CSS px. |
| `cell-columns` | `number` | Cells across (0: as many as fit). |

<a id="guide-components-10"></a>
### `components §10` `<text>`, `<p>`, `<span>`, `<h1>`, `<a>`, `<og-counter>`, `<og-typewriter>` (`label`): A text

A text. A leaf; text-bearing controls accept text content: `<text id="Name">…</text>` or `<p id="Name">Read the <a href="rules">rules</a>.</p>` or `<span id="Name">…</span>` or `<h1 id="Name">…</h1> (to <h6>)` or `<a id="Name" href="shop">…</a>` or `<og-counter id="Name">…</og-counter>` or `<og-typewriter id="Name">…</og-typewriter>`.

Its own props, beside every element's:

| Write | Value | What |
|---|---|---|
| `text` | `text` | What it says. |
| `i18n` | `name` | A string table key (og.i18n) its text comes from; `text` is the fallback. |
| `font:` | `var(--text-<name>)` | The type step. |
| `color:` | `var(--<role>) \| #rrggbb[aa]` | Its ink. |
| `spans` | `TextSpan[]` | Read-only TextSpan[] ranges over plain text; offsets count Unicode scalars. Styles come from ::part(name). |
| `text-transform:` | `uppercase \| none` | Uppercase in the UI's locale, as a type role with `case upper` is. |
| `text-align:` | `start \| center \| end` | How its lines align. |
| `white-space:` | `nowrap \| normal` | One line. |
| `line-clamp:` | `<number>` | The most lines it shows. |
| `-og-fit-min:` | `body \| xs \| meta \| caption \| sm \| reading \| base \| lg \| xl \| xl2 \| xl3 \| xl4 \| xl5 \| xl6 \| xl7 \| xl8 \| display-sm \| display-md \| display-lg \| display-xl \| display-xl2 \| hero` | Shrinks by measured text fit through type steps down to this literal step (for example -og-fit-min: caption). Extension reason: clamp() and container units cannot choose a type step by measuring shaped text. Author variables may substitute a step name; no custom-property name has engine meaning. |
| `text-overflow:` | `ellipsis \| clip` | Cut with … where it does not fit. `ellipsisText`: What its cut ends in instead of … (`text-overflow: "<string>"`). |
| `font-family:` | `var(--font-<name>)` | The font role. |
| `heading` | `bool` | A heading (a style's flourish frames it). |
| `<og-counter value>` | `number` | The target value of <og-counter value={$score}>. Changes interpolate over its duration in engine ticks. |
| `<og-counter duration>` | `number` | The counter's duration in milliseconds (900 by default; zero also selects the default). |
| `<og-counter decimals>` | `number` | The counter's digits after the decimal point. |
| `<og-typewriter rate>` | `number` | The typewriter's extended grapheme clusters per second; onrevealed fires once when the text is whole. Text changes restart it. Reduced motion shows the whole text. Progress follows engine ticks and checkpoints. |
| `href` | `text` | An `<a>`'s destination: a click sends a `link` event named by it (as `onclick` fires). |

<a id="guide-components-11"></a>
### `components §11` `<button>` (`button`): A button: its text, or its content (an icon, a text, a badge) in a row inside its frame and states; `data-hold` makes it a hold-to-confirm button that fills while held down

A button: its text, or its content (an icon, a text, a badge) in a row inside its frame and states; `data-hold` makes it a hold-to-confirm button that fills while held down. A container with nested children: `<button id="Name">…</button>`.

Its own props, beside every element's:

| Write | Value | What |
|---|---|---|
| `data-action` | `name` | On `<kbd>`, the action whose glyph it shows on the device used last (its key, its pad button, or its touch icon); it follows the device live. On a `<button data-hold>`, the action whose hold holds it as a press does. |
| `text` | `text` | What it says. |
| `i18n` | `name` | A string table key (og.i18n) its text comes from; `text` is the fallback. |
| `variant` | `secondary \| primary \| quiet \| danger \| ghost` | The button's role. |
| `size` | `small \| medium \| large` | The control height (`-og-control-size`). |
| `disabled` | `bool` | Takes no input. |
| `autofocus` | `bool` | Takes the focus when it appears. |
| `square` | `bool` | A square of the control height: an icon button. |
| `data-og-touch` | `name` | A touch button (`<button data-og-touch="jump">`): the input source of og-input's action `jump`, held while a touch that went down on it is held, whatever else is held (a thumb on the stick). |

<a id="guide-components-12"></a>
### `components §12` `<kbd>` (`kbd`): A key or an action's glyph: `<kbd>Esc</kbd>` a keycap, `<kbd data-action="jump"></kbd>` the action's glyph on the device used last

A key or an action's glyph: `<kbd>Esc</kbd>` a keycap, `<kbd data-action="jump"></kbd>` the action's glyph on the device used last. A leaf; text-bearing controls accept text content: `<kbd id="Name">…</kbd>`.

Its own props, beside every element's:

| Write | Value | What |
|---|---|---|
| `data-action` | `name` | On `<kbd>`, the action whose glyph it shows on the device used last (its key, its pad button, or its touch icon); it follows the device live. On a `<button data-hold>`, the action whose hold holds it as a press does. |
| `glyph-device` | `text` | The device it draws for (bound to `$ui.device` by the parser: a device change rebuilds it). |
| `data-og-rebind` | `bool` | A press captures the next key as its binding (`bind:value`), Escape cancels. |
| `binding` | `text` | With dataOgRebind: the key code it captured (`bind:value`), written back. Two-way only with an explicit bind: attribute. |
| `text` | `text` | What it says. |
| `i18n` | `name` | A string table key (og.i18n) its text comes from; `text` is the fallback. |

<a id="guide-components-13"></a>
### `components §13` `<img>` (`image`): A texture. CSS filter with an SVG colour matrix multiplies its RGBA channels; use a background for a solid fill

A texture. CSS filter with an SVG colour matrix multiplies its RGBA channels; use a background for a solid fill. A leaf; text-bearing controls accept text content: `<img id="Name" />`.

Its own props, beside every element's:

| Write | Value | What |
|---|---|---|
| `src` | `number` | A texture handle (og.load.texture), usually bound. |
| `object-view-box:` | `inset(<t>% [<r>% [<b>% [<l>%]]])` | The part of the texture it shows (`object-view-box: inset(…)` in percentages of the texture, ordered top, right, bottom, left). |
| `object-fit:` | `cover \| contain \| fill \| none \| scale-down \| nine-slice \| tile` | CSS `object-fit`: how the image fills its box (fill, contain, cover, none, scale-down) or the art its slot (also nineSlice and tile). |
| `object-position:` | `<position>: one to four of left, center, right, top, bottom, <n>px, <n>%` | CSS `object-position`: where contain, cover, none or scale-down place the image in its box, as background-position is written (50% 50% by default). |

<a id="guide-components-14"></a>
### `components §14` `<art>` (`art`): An art slot (docs/ui.md §3.3): a spec the art pipeline reads, its texture once there is one

An art slot (docs/ui.md §3.3): a spec the art pipeline reads, its texture once there is one. A leaf; text-bearing controls accept text content: `<art id="Name">…</art>`.

Its own props, beside every element's:

| Write | Value | What |
|---|---|---|
| `src` | `number` | A texture handle (og.load.texture), usually bound. |
| `kind` | `name` | The slot's kind (Appendix F: itemIcon, bust, keyArt…). |
| `ratio` | `x y` | The slot's aspect, width and height in whole numbers. |
| `focal` | `x y` | The point cropping keeps, 0–1. |
| `safe` | `4 numbers` | Where text or controls lie over it: x0 y0 x1 y1, 0–1. |
| `object-fit:` | `cover \| contain \| fill \| none \| scale-down \| nine-slice \| tile` | CSS `object-fit`: how the image fills its box (fill, contain, cover, none, scale-down) or the art its slot (also nineSlice and tile). |
| `object-position:` | `<position>: one to four of left, center, right, top, bottom, <n>px, <n>%` | CSS `object-position`: where contain, cover, none or scale-down place the image in its box, as background-position is written (50% 50% by default). |
| `item-cells` | `x y` | An item icon's grid cells, columns and rows (1 1, 1 2, 2 3 or 2 4). |
| `slice` | `insets` | A nine-slice's insets as fractions of the art: all, x y, or left top right bottom. |
| `art-set` | `name` | The set it is generated with for a consistent look (the kind's name by default). |

<a id="guide-components-15"></a>
### `components §15` `<og-icon>` (`icon`): An icon of the style's set

An icon of the style's set. A leaf; text-bearing controls accept text content: `<og-icon id="Name" />`.

Its own props, beside every element's:

| Write | Value | What |
|---|---|---|
| `color:` | `var(--<role>) \| #rrggbb[aa]` | Its ink. |
| `secondary` | `name` | The second icon layer, in the same atlas. |
| `icon` | `name` | The icon's name in the style's set. |
| `icon-size` | `number` | CSS px (24 by default). |
| `cue` | `name` | On <og-icon cue="danger">, the style's icon for that status, rarity, team or damage type, so its meaning never rests on colour alone. Its `icon`, when set, wins. Style the primary and secondary layers with ::part(image) and ::part(secondary). |

<a id="guide-components-16"></a>
### `components §16` `<progress>` (`progress`): A progress bar

A progress bar. A leaf; text-bearing controls accept text content: `<progress id="Name">…</progress>`.

Its own props, beside every element's:

| Write | Value | What |
|---|---|---|
| `value` | `number` | Its value (a slider writes it back). Two-way only with an explicit bind: attribute. |

<a id="guide-components-17"></a>
### `components §17` `<input type="range">` (`slider`): A slider (two-way `value`)

A slider (two-way `value`). A leaf; text-bearing controls accept text content: `<input type="range" id="Name" />`.

Its own props, beside every element's:

| Write | Value | What |
|---|---|---|
| `disabled` | `bool` | Takes no input. |
| `value` | `number` | Its value (a slider writes it back). Two-way only with an explicit bind: attribute. |
| `min` | `number` | The range's low end; a number input under it is invalid (`rangeUnderflow`). |
| `max` | `number` | The range's high end; a number input over it is invalid (`rangeOverflow`). |
| `show-value` | `bool` | Show the slider's value readout (default true); false when an adjacent label supplies units. |
| `data-fill-origin` | `number` | The value its fill grows from (`data-fill-origin`; default min): 0 on a -1 to 1 balance fills either way from the centre. |
| `mark-values` | `names` | The values of the `<datalist>` its `list` names: a tick at each. |
| `mark-labels` | `names` | The marks' labels, under the track (an `<option label>`'s, or its text); empty for a tick only. |
| `step` | `number` | A slider snaps to multiples of this (0: continuous). A number input's value off the step grid from `min` is invalid (default 1; `any` or 0: no step). |
| `low` | `number` | A meter's top of the low region (HTML `low`; default `min`). On a range input with `high`, its low knob (a slider writes it back). Two-way only with an explicit bind: attribute. |
| `high` | `number` | A meter's bottom of the high region (HTML `high`; default `max`). On a range input, its high knob: two knobs on one track, `low` to `high` (a slider writes both back). Two-way only with an explicit bind: attribute. |
| `data-min-gap` | `number` | The least distance between a range input's two knobs (`data-min-gap`, in its values; default 0). |

<a id="guide-components-18"></a>
### `components §18` `<input type="checkbox">`, `<input type="checkbox" switch>`, `<label>` (`toggle`): A switch with a label (two-way `checked`)

A switch with a label (two-way `checked`). A leaf; text-bearing controls accept text content: `<input type="checkbox" id="Name" />` or `<input type="checkbox" switch id="Name" />` or `<label id="Name"><input type="checkbox" /> Caption</label>`.

Its own props, beside every element's:

| Write | Value | What |
|---|---|---|
| `text` | `text` | What it says. |
| `i18n` | `name` | A string table key (og.i18n) its text comes from; `text` is the fallback. |
| `size` | `small \| medium \| large` | The control height (`-og-control-size`). |
| `disabled` | `bool` | Takes no input. |
| `switch` | `bool` | Draws the checkbox as a switch (HTML `switch`); without it, a box with a check. |
| `indeterminate` | `bool` | Neither on nor off: a dash in its box (HTML `indeterminate`); a press turns it on. |
| `data-on` | `text` | What a switch's value says while on (`data-on="On"`), for a screen reader and the probe; empty for none. |
| `data-off` | `text` | And while off (`data-off="Off"`). |
| `checked` | `bool` | Whether it is on (written back). Two-way only with an explicit bind: attribute. |
| `description` | `text` | A line under its caption: a `<small>` in its `<label>`. |
| `side` | `start \| end` | Where its caption sits, from the source order in its `<label>`: start before the switch (caption first), end after it. |
| `flex-direction:` | `row \| row-reverse \| column \| column-reverse` | `flex-direction` on a checkbox's label: rowReverse swaps its caption's side. |
| `invalid` | `bool` | Its value is wrong (`aria-invalid`): the danger frame. |

<a id="guide-components-19"></a>
### `components §19` `<input>` (`color`): A colour input: a swatch that opens a picker (two-way `value`, `#rrggbb`)

A colour input: a swatch that opens a picker (two-way `value`, `#rrggbb`). A leaf; text-bearing controls accept text content: `<input type="color" id="Name" bind:value={$tint} />`.

Its own props, beside every element's:

| Write | Value | What |
|---|---|---|
| `disabled` | `bool` | Takes no input. |
| `color-value` | `text` | Its colour as `#rrggbb` (HTML `value`; written back). Two-way only with an explicit bind: attribute. |

<a id="guide-components-20"></a>
### `components §20` `<datetime>` (`datetime`): A date or time input (`<input type="date|time|datetime-local|month|week">`): a stepper per part, two-way `dateValue` in HTML's format

A date or time input (`<input type="date|time|datetime-local|month|week">`): a stepper per part, two-way `dateValue` in HTML's format. A leaf; text-bearing controls accept text content: `<datetime id="Name">…</datetime>`.

Its own props, beside every element's:

| Write | Value | What |
|---|---|---|
| `disabled` | `bool` | Takes no input. |
| `step` | `number` | A slider snaps to multiples of this (0: continuous). A number input's value off the step grid from `min` is invalid (default 1; `any` or 0: no step). |
| `date-value` | `text` | Its value in HTML's format (`2026-11-08`, `14:30`, `2026-11-08T14:30`, `2026-11`, `2026-W45`; written back); empty until a part is stepped. Two-way only with an explicit bind: attribute. |
| `date-type` | `date \| time \| datetime-local \| month \| week` | Which parts it has (HTML `type`). |
| `date-min` | `text` | The earliest value (HTML `min`, in its format); the picker keeps to it and is the step base. |
| `date-max` | `text` | The latest value (HTML `max`, in its format). |
| `readonly` | `bool` | Its text is selected and copied but never edited; it keeps its focus stop. |
| `required` | `bool` | Empty is invalid (`valueMissing`). |

<a id="guide-components-21"></a>
### `components §21` `<video>` (`video`): HTML's `<video>`: the frame of the og.VideoPlayer entity `src` binds, with Svelte's media bindings and optional controls

HTML's `<video>`: the frame of the og.VideoPlayer entity `src` binds, with Svelte's media bindings and optional controls. A leaf; text-bearing controls accept text content: `<video id="Name">…</video>`.

Its own props, beside every element's:

| Write | Value | What |
|---|---|---|
| `media-src` | `number` | The player (HTML `src`): an entity with og.VideoPlayer (a video's) or og.AudioSource, from a store. |
| `paused` | `bool` | Whether it is paused (Svelte's `bind:paused`): written to play or pause it, written back as it ends. Two-way only with an explicit bind: attribute. |
| `current-time` | `number` | Seconds in (`bind:currentTime`): written back as it plays; a value off the player's seeks it. Two-way only with an explicit bind: attribute. |
| `duration` | `number` | Its length in seconds (`bind:duration`; written back). Two-way only with an explicit bind: attribute. |
| `volume` | `number` | Its sound's gain, 0 to 1 (`bind:volume`; default 1). Two-way only with an explicit bind: attribute. |
| `muted` | `bool` | Silenced whatever its volume (`bind:muted`). Two-way only with an explicit bind: attribute. |
| `ended` | `bool` | It reached its end without loop (`bind:ended`; written back). Two-way only with an explicit bind: attribute. |
| `autoplay` | `bool` | Plays as it first appears. |
| `loop` | `bool` | Starts again at its end. |
| `poster` | `number` | A texture handle shown until the first frame plays (HTML `poster`), usually bound. |
| `controls` | `bool` | Draws a play and pause button and its time. |

<a id="guide-components-22"></a>
### `components §22` `<audio>` (`audio`): HTML's `<audio>`: the og.AudioSource entity `src` binds, played and paused by `paused`; drawn only with `controls`

HTML's `<audio>`: the og.AudioSource entity `src` binds, played and paused by `paused`; drawn only with `controls`. A leaf; text-bearing controls accept text content: `<audio id="Name">…</audio>`.

Its own props, beside every element's:

| Write | Value | What |
|---|---|---|
| `media-src` | `number` | The player (HTML `src`): an entity with og.VideoPlayer (a video's) or og.AudioSource, from a store. |
| `paused` | `bool` | Whether it is paused (Svelte's `bind:paused`): written to play or pause it, written back as it ends. Two-way only with an explicit bind: attribute. |
| `volume` | `number` | Its sound's gain, 0 to 1 (`bind:volume`; default 1). Two-way only with an explicit bind: attribute. |
| `muted` | `bool` | Silenced whatever its volume (`bind:muted`). Two-way only with an explicit bind: attribute. |
| `autoplay` | `bool` | Plays as it first appears. |
| `loop` | `bool` | Starts again at its end. |
| `controls` | `bool` | Draws a play and pause button and its time. |

<a id="guide-components-23"></a>
### `components §23` `<file>` (`file`): A file input (`<input type="file">`), built only where the host opens a file picker (og.ui.filePicker): a button whose press sends `filepick`; the host writes the names chosen to two-way `fileValue`

A file input (`<input type="file">`), built only where the host opens a file picker (og.ui.filePicker): a button whose press sends `filepick`; the host writes the names chosen to two-way `fileValue`. A leaf; text-bearing controls accept text content: `<file id="Name">…</file>`.

Its own props, beside every element's:

| Write | Value | What |
|---|---|---|
| `disabled` | `bool` | Takes no input. |
| `file-value` | `text` | The names of the files chosen, comma-separated (HTML `value`; the host writes it). Two-way only with an explicit bind: attribute. |
| `accept` | `text` | The types it takes (HTML `accept`: `.png,image/*`). |
| `multiple` | `bool` | More than one file may be chosen. |

<a id="guide-components-24"></a>
### `components §24` `<input>` (`radio`): One choice of a radio group with its caption: checked while its group (`bind:group`) holds its value, which a press writes

One choice of a radio group with its caption: checked while its group (`bind:group`) holds its value, which a press writes. A leaf; text-bearing controls accept text content: `<label><input type="radio" value="easy" bind:group={$level} /> Easy</label>`.

Its own props, beside every element's:

| Write | Value | What |
|---|---|---|
| `text` | `text` | What it says. |
| `i18n` | `name` | A string table key (og.i18n) its text comes from; `text` is the fallback. |
| `disabled` | `bool` | Takes no input. |
| `group` | `text` | The store its group's choice is in (`bind:group`): it is checked while that holds its value, and a press writes its value. Two-way only with an explicit bind: attribute. |
| `choice` | `text` | Its value (HTML `value`), what its group holds when it is checked. |
| `checked` | `bool` | Whether it is on (written back). Two-way only with an explicit bind: attribute. |
| `description` | `text` | A line under its caption: a `<small>` in its `<label>`. |
| `side` | `start \| end` | Where its caption sits, from the source order in its `<label>`: start before the switch (caption first), end after it. |

<a id="guide-components-25"></a>
### `components §25` `<tabs>` (`tabs`): A tab list (two-way `selected`), using the core tabs widget

A tab list (two-way `selected`), using the core tabs widget. A leaf; text-bearing controls accept text content: `<tabs id="Name">…</tabs>`.

Its own props, beside every element's:

| Write | Value | What |
|---|---|---|
| `text` | `text` | What it says. |
| `i18n` | `name` | A string table key (og.i18n) its text comes from; `text` is the fallback. |
| `size` | `small \| medium \| large` | The control height (`-og-control-size`). |
| `disabled` | `bool` | Takes no input. |
| `items` | `names` | The tab or option labels, literal names or a bound list of strings (a `<select>`'s `<option>`s set them). |
| `keys` | `names` | A string table key (og.i18n) per tab its label comes from; its item is the fallback. |
| `icons` | `names` | An icon of the style's set per tab, before its label (`icons="sword shield"`, or a bound list); an empty name is none. |
| `badges` | `names` | A count badge per tab, after its label (`badges={$unread}`, a list of strings); an empty one is none. |
| `values` | `names` | A name per tab (a screen's mode) that `current` holds instead of `selected`'s index. |
| `current` | `name` | The selected item's name from `values` (a `<select>`'s `value`); on tabs it takes over from `selected` when `values` is set. Two-way only with an explicit bind: attribute. |
| `selected` | `number` | The selected tab's zero-based index. Two-way only with an explicit bind: attribute. |
| `fill` | `bool` | Equal tabs filling the available width. |

<a id="guide-components-26"></a>
### `components §26` `<input type="text">`, `<textarea>` (`field`): A text control (two-way `input`): `<input>` for one line, `<textarea>` for several; `type="number"` keeps editable text and adds increment/decrement buttons. Min, max and step constrain stepping; data-wrap, data-large-step and data-format configure it

A text control (two-way `input`): `<input>` for one line, `<textarea>` for several; `type="number"` keeps editable text and adds increment/decrement buttons. Min, max and step constrain stepping; data-wrap, data-large-step and data-format configure it. A leaf; text-bearing controls accept text content: `<input type="text" id="Name" />` or `<textarea id="Name" rows="4" />`.

Its own props, beside every element's:

| Write | Value | What |
|---|---|---|
| `disabled` | `bool` | Takes no input. |
| `min` | `number` | The range's low end; a number input under it is invalid (`rangeUnderflow`). |
| `max` | `number` | The range's high end; a number input over it is invalid (`rangeOverflow`). |
| `step` | `number` | A slider snaps to multiples of this (0: continuous). A number input's value off the step grid from `min` is invalid (default 1; `any` or 0: no step). |
| `data-wrap` | `bool` | A number input past its last valid step at or below max goes to min and back (`data-wrap`; needs both bounds). |
| `data-large-step` | `number` | The larger number-input step for Shift adjustment or a button held past a second (`data-large-step="10"`); 0 is ten steps. |
| `data-format` | `text` | How an unfocused number input presents its value: an ICU message over `value` in the UI's locale (`data-format="{value, number} m"`, `{value, number, compact}`, `{value, plural, =0 {Off} =1 {Low} other {High}}`). Editing retains the raw string. |
| `appearance:` | `auto \| none` | none drops the control's own frame: its box, borders and background are the document's. |
| `suggestions` | `names` | A text input's suggestions, from the `<datalist>` its `list` names (its options' values, or its `items={$names}`): a list under the field while it has the focus, those containing the text first by prefix, choosing one writes it to the field. |
| `<datalist><option label>` | `names` | The labels of the input's datalist options, separate from the values written on selection. |
| `<datalist><option i18n>` | `names` | Translation keys for datalist option labels; missing translations use the option's literal label. |
| `value` | `text` | The field's text (written back). Two-way only with an explicit bind: attribute. |
| `placeholder` | `text` | Shown while it is empty. |
| `readonly` | `bool` | Its text is selected and copied but never edited; it keeps its focus stop. |
| `required` | `bool` | Empty is invalid (`valueMissing`). |
| `minlength` | `number` | Fewer characters are invalid (HTML `minlength`). |
| `pattern` | `text` | A regular expression the whole text must match (HTML `pattern`); refused when it is not one. |
| `rows` | `number` | A `<textarea>`'s lines shown at once (HTML `rows`; 0 is 4); more scroll. |
| `<textarea>` | `bool` | Set by `<textarea>`: Enter starts a line and the text wraps. Not written as an attribute. |
| `maxlength` | `number` | The most characters it takes. |
| `tight` | `bool` | No inset and its text's line height: the text sits where the field is laid (in a box the screen draws). |
| `component` | `name` | The component its style dresses it as (`Poem`, `Search[large]`): `<component>.frame`, `.text` and `.placeholder`; else Field. |
| `invalid` | `bool` | Its value is wrong (`aria-invalid`): the danger frame. |
| `type` | `text \| password \| number \| email \| url \| tel \| search` | What it takes, by HTML's names: `password` is drawn masked with a reveal toggle (`<component>.reveal[revealed]`) and never copied or cut; `number` takes only a number's characters; a `number`, `email` or `url` whose text is not one matches `:invalid`. Each picks its virtual keyboard. |
| `inputmode` | `none \| text \| decimal \| numeric \| tel \| search \| email \| url` | The virtual keyboard a touch device shows (HTML `inputmode`; `none`: the game shows its own); unset, its `type`'s. |
| `enterkeyhint` | `enter \| done \| go \| next \| previous \| search \| send` | The on-screen keyboard's enter key (HTML `enterkeyhint`): `Search`, `Send`, `Go` and so on; unset, `Done`. |
| `lang` | `name` | The language typed (HTML `lang`): `ja` offers the on-screen keyboard's kana table. |
| `autocomplete` | `name` | The page's autofill hint (HTML `autocomplete`: `username`, `current-password`, `new-password`, `one-time-code`, `email`, `off`…). |

<a id="guide-components-27"></a>
### `components §27` `<badge>` (`badge`): A small toned label

A small toned label. A leaf; text-bearing controls accept text content: `<badge id="Name">…</badge>`.

Its own props, beside every element's:

| Write | Value | What |
|---|---|---|
| `text` | `text` | What it says. |
| `i18n` | `name` | A string table key (og.i18n) its text comes from; `text` is the fallback. |
| `tone` | `neutral \| info \| success \| warn \| danger \| accent` | Its tone. |

<a id="guide-components-28"></a>
### `components §28` `<spacer>` (`spacer`): Empty room (grows by default)

Empty room (grows by default). A leaf; text-bearing controls accept text content: `<spacer id="Name">…</spacer>`.

It takes every element's props and none of its own.

<a id="guide-components-29"></a>
### `components §29` `<hr>` (`divider`): A hairline

A hairline. A leaf; text-bearing controls accept text content: `<hr id="Name" />`.

Its own props, beside every element's:

| Write | Value | What |
|---|---|---|
| `vertical` | `bool` | Upright. |

<a id="guide-components-30"></a>
### `components §30` `<select>` (`select`): A select (two-way `value`): its `<option>`s, under `<optgroup>`s, in a list that opens over it

A select (two-way `value`): its `<option>`s, under `<optgroup>`s, in a list that opens over it. A leaf; text-bearing controls accept text content: `<select id="Name" bind:value={$x}><optgroup label="G"><option value="a">A</option></optgroup></select>`.

Its own props, beside every element's:

| Write | Value | What |
|---|---|---|
| `disabled` | `bool` | Takes no input. |
| `data-cycle` | `bool` | `<select data-cycle>` draws previous, its option and next; selection, events and bindings remain those of the select. |
| `data-filter-placeholder` | `text` | What the open list's filter field shows while empty (`data-filter-placeholder`); empty is Filter. |
| `items` | `names` | The tab or option labels, literal names or a bound list of strings (a `<select>`'s `<option>`s set them). |
| `keys` | `names` | A string table key (og.i18n) per tab its label comes from; its item is the fallback. |
| `values` | `names` | A name per tab (a screen's mode) that `current` holds instead of `selected`'s index. |
| `current` | `name` | The selected item's name from `values` (a `<select>`'s `value`); on tabs it takes over from `selected` when `values` is set. Two-way only with an explicit bind: attribute. |
| `groups` | `names` | An `<option>`'s `<optgroup>` label, one per option (empty: none); options of one group follow each other. |
| `disabled-items` | `names` | The `<option disabled>`s (and those of an `<optgroup disabled>`), by their zero-based index. |
| `placeholder` | `text` | Shown while it is empty. |
| `invalid` | `bool` | Its value is wrong (`aria-invalid`): the danger frame. |

<a id="guide-components-31"></a>
### `components §31` `<dialog>`, `<og-spotlight>` (`dialog`): A dialog (two-way `open`): `modal` over a backdrop that traps the focus, else a panel in place; Escape or B closes it and the focus returns

A dialog (two-way `open`): `modal` over a backdrop that traps the focus, else a panel in place; Escape or B closes it and the focus returns. A container with nested children: `<dialog id="Name" modal bind:open={$x} aria-label="Quit?">…</dialog>` or `<og-spotlight id="Name">…</og-spotlight>`.

Its own props, beside every element's:

| Write | Value | What |
|---|---|---|
| `<og-spotlight target>` | `name` | The id tracked by <og-spotlight target="Target">. While open it dims the page outside that element and places its content as a callout. |
| `open` | `bool` | Whether it is open (HTML `open`; closing writes false back). Two-way only with an explicit bind: attribute. |
| `closedby` | `any \| closerequest \| none` | What closes it (HTML `closedby`): any adds a press outside a modal (light dismiss) to closerequest's Escape or a pad's B; none leaves closing to the document. Default: closerequest for a modal, none in place. |
| `appearance:` | `auto \| none` | none drops the control's own frame: its box, borders and background are the document's. |
| `modal` | `bool` | Over a backdrop that takes every press, the focus trapped inside (HTML `showModal()`); else a panel in place (`show()`). |
| `data-detents` | `text` | `<dialog modal data-detents="40% 90%">`: a sheet docked to its `data-edge` (the bottom at full width by default), dragged onto its detents and swiped off its edge to close (writing `open` back). Detents are positive px or percentages of viewport height, or width for a side sheet. It opens at the first; drag and snap state is checkpointed. |
| `data-edge` | `block-end \| inline-start \| inline-end` | `data-edge="block-end\|inline-start\|inline-end"` beside `data-detents`: the edge the sheet docks to, in CSS's logical names, so the inline ones mirror right to left (`inline-end` is the right left to right, the left in Arabic); a side sheet is full height, its detents widths, and swipes off sideways. |

<a id="guide-components-32"></a>
### `components §32` `<details>` (`details`): A disclosure (two-way `open`): its `<summary>` row, its content shown only while open

A disclosure (two-way `open`): its `<summary>` row, its content shown only while open. A container with nested children: `<details id="Name" bind:open={$x}><summary>More</summary>…</details>`.

Its own props, beside every element's:

| Write | Value | What |
|---|---|---|
| `disabled` | `bool` | Takes no input. |
| `open` | `bool` | Whether it is open (HTML `open`; closing writes false back). Two-way only with an explicit bind: attribute. |
| `summary` | `text` | Its `<summary>`'s text; empty is `Details`. Written as the first child, `<summary>…</summary>`. |
| `name` | `name` | HTML `name` on a `<details>`, an exclusive accordion: of the disclosures in a document sharing it, one is open at once, and opening one closes the others (its open state written back). |

<a id="guide-components-33"></a>
### `components §33` `<div>`, `<nav>`, `<ul>`, `<ol>` (`box`): A generic HTML block (`<div>`, `<section>`, `<nav>`, `<ul>`, `<li>`…): its layout is its CSS `display`, `block` (children top to bottom) by default, `flex` (a row, or `flex-direction`) or `grid`. Text inside it is an anonymous text element; an `<li>` draws its list marker

A generic HTML block (`<div>`, `<section>`, `<nav>`, `<ul>`, `<li>`…): its layout is its CSS `display`, `block` (children top to bottom) by default, `flex` (a row, or `flex-direction`) or `grid`. Text inside it is an anonymous text element; an `<li>` draws its list marker. A container with nested children: `<div id="Name">…</div>` or `<nav aria-label="Site">…</nav> (section, header, footer, main, aside, article)` or `<ul><li>…</li></ul>` or `<ol start="1"><li>…</li></ol>`.

Its own props, beside every element's:

| Write | Value | What |
|---|---|---|
| `flex-direction:` | `row \| column \| row-reverse \| column-reverse` | Lays a row's or a column's children out the other way (an `at` block's: a phone in landscape puts the actions beside the content). |
| `colspan` | `number` | A table cell's columns (`<td colspan>`), 1 by default; CSS table layout reads it. |
| `rowspan` | `number` | A table cell's rows (`<td rowspan>`), 1 by default, 0 to the end of its row group; CSS table layout reads it. |
| `list-style-type:` | `none \| disc \| circle \| square \| decimal \| lower-alpha \| upper-alpha \| lower-roman \| upper-roman` | An `<li>`'s marker, from the inherited `list-style-type` (`<ul>`: disc, nested circle then square; `<ol>`: decimal). |
| `list-style-position:` | `outside \| inside` | Where an `<li>`'s marker sits, from the inherited `list-style-position`: `outside` hangs it before the item's start edge (the default), `inside` lays the item out as a row after it. |

<a id="guide-components-34"></a>
### `components §34` `<meter>` (`meter`): A gauge of a value in a known range (HTML `<meter>`), its fill dressed by the region `low`, `high` and `optimum` put the value in

A gauge of a value in a known range (HTML `<meter>`), its fill dressed by the region `low`, `high` and `optimum` put the value in. A leaf; text-bearing controls accept text content: `<meter id="Name" value={$hp} min="0" max="100" low="25" high="75" optimum="90" />`.

Its own props, beside every element's:

| Write | Value | What |
|---|---|---|
| `value` | `number` | Its value (a slider writes it back). Two-way only with an explicit bind: attribute. |
| `min` | `number` | The range's low end; a number input under it is invalid (`rangeUnderflow`). |
| `max` | `number` | The range's high end; a number input over it is invalid (`rangeOverflow`). |
| `low` | `number` | A meter's top of the low region (HTML `low`; default `min`). On a range input with `high`, its low knob (a slider writes it back). Two-way only with an explicit bind: attribute. |
| `high` | `number` | A meter's bottom of the high region (HTML `high`; default `max`). On a range input, its high knob: two knobs on one track, `low` to `high` (a slider writes both back). Two-way only with an explicit bind: attribute. |
| `optimum` | `number` | The best value (HTML `optimum`; default the middle): its region is `optimum`, the next `suboptimum`, the far one `poor`. |

<a id="guide-components-35"></a>
### `components §35` `<anchor>` (`anchor`): A box held over a world point (`<anchor entity={u.entity} offset="0 2 0">`): placed every tick from the camera and the entity's pose, its children laid out as a column; `occlusion`, `declutter`, `clamp="edge"`, `priority`, `max-distance`, `scale-at`

A box held over a world point (`<anchor entity={u.entity} offset="0 2 0">`): placed every tick from the camera and the entity's pose, its children laid out as a column; `occlusion`, `declutter`, `clamp="edge"`, `priority`, `max-distance`, `scale-at`. A container with nested children: `<anchor entity={u.entity} offset="0 2.1 0" occlusion="fade" declutter="stack" clamp="edge">…</anchor>`.

Its own props, beside every element's:

| Write | Value | What |
|---|---|---|
| `mass` | `text` | `<anchor entity={h.entity} mass="damage" value={h.amount}>`: a mass element of the template `mass` instead of a box (thousands per frame, drawn on the GPU). With a CSS `animation`, each row spawns one that plays its `@keyframes` (translate, scale, opacity) over the animation's duration; without one it stays while its row does. `value` is the number shown, or with `max` a bar filled to value/max; `color` tints it. |
| `mass-value` | `number` | A mass element's value: the number it shows, or its bar's fill with `max`. |
| `mass-max` | `number` | A mass element's bar: drawn as a bar filled to value/max (an overhead health bar). |
| `entity` | `number` | The entity it follows, by its bits (`entity={u.entity}`): its world pose as the host hands it in. |
| `world-at` | `text` | `at="x y z"`: a world point it follows, without an entity. |
| `world-offset` | `text` | `offset="0 2 0"`: added to the point, world units (over a head). |
| `occlusion` | `none \| fade \| hide` | What the scene in front of its point does to it: faded or hidden. |
| `declutter` | `none \| shift \| stack \| hide` | How it resolves overlapping an anchor of higher priority. |
| `world-clamp` | `none \| edge` | `clamp="edge"`: held on the screen's edge with an arrow while its point is off screen (a waypoint). |
| `priority` | `number` | Higher keeps its place when anchors overlap. |
| `max-distance` | `number` | `max-distance`: hidden past this many world units, fading over its last tenth. |
| `world-scale-at` | `number` | `scale-at="10"`: scaled 1 at this distance, larger nearer and smaller farther (0.5 to 2). |

<a id="guide-components-36"></a>
### `components §36` `<ground>` (`ground`): A box laid flat in the world (`<ground entity={u.entity} size="3 3">…</ground>`): laid out at its CSS size and drawn by the UI onto the horizontal plane `size` world units across at the entity's (or `at`'s) height and draped over the terrain under it as a decal, seen in perspective under the screen's elements (a selection ring, a range marker, a decal); `rotation="0 45 0"` turns it. Inert

A box laid flat in the world (`<ground entity={u.entity} size="3 3">…</ground>`): laid out at its CSS size and drawn by the UI onto the horizontal plane `size` world units across at the entity's (or `at`'s) height and draped over the terrain under it as a decal, seen in perspective under the screen's elements (a selection ring, a range marker, a decal); `rotation="0 45 0"` turns it. Inert. A container with nested children: `<ground entity={u.entity} size="3 3"><svg viewBox="0 0 40 40">…</svg></ground>`.

Its own props, beside every element's:

| Write | Value | What |
|---|---|---|
| `world-size` | `text` | A world panel's quad, `width height` world units (default 1 1). |
| `world-rotation` | `text` | A world panel's turn, degrees about x, y and z (`0 30 0`); facing +z unturned. |
| `entity` | `number` | The entity it follows, by its bits (`entity={u.entity}`): its world pose as the host hands it in. |
| `world-at` | `text` | `at="x y z"`: a world point it follows, without an entity. |

<a id="guide-components-37"></a>
### `components §37` `<joystick>` (`joystick`): A virtual stick (`<joystick action="move">`): a base and a knob that follows the touch that started on it, the input source of og-input's two-axis action `action` (from the base's centre over its radius, y up, past `data-dead-zone`). Its CSS width sizes it (default 128 px)

A virtual stick (`<joystick action="move">`): a base and a knob that follows the touch that started on it, the input source of og-input's two-axis action `action` (from the base's centre over its radius, y up, past `data-dead-zone`). Its CSS width sizes it (default 128 px). A leaf; text-bearing controls accept text content: `<joystick id="Stick" action="move" data-dead-zone="0.2" customize />`.

Its own props, beside every element's:

| Write | Value | What |
|---|---|---|
| `disabled` | `bool` | Takes no input. |
| `action` | `name` | The og-input action it is an input source of (`og.input.defineAction`; a two-axis one), read in the tick from the touches that started on it. |
| `data-dead-zone` | `number` | A stick's dead zone, a ratio of its radius (`data-dead-zone="0.2"`; default 0.15): past it the value rises from 0. |

<a id="guide-components-38"></a>
### `components §38` `<dpad>` (`dpad`): A virtual d-pad (`<dpad action="move">`): four arms read by the angle of the touch that started on it, eight ways, the input source of og-input's two-axis action `action` (each part -1, 0 or 1). Its CSS width sizes it (default 132 px)

A virtual d-pad (`<dpad action="move">`): four arms read by the angle of the touch that started on it, eight ways, the input source of og-input's two-axis action `action` (each part -1, 0 or 1). Its CSS width sizes it (default 132 px). A leaf; text-bearing controls accept text content: `<dpad id="Pad" action="move" />`.

Its own props, beside every element's:

| Write | Value | What |
|---|---|---|
| `disabled` | `bool` | Takes no input. |
| `action` | `name` | The og-input action it is an input source of (`og.input.defineAction`; a two-axis one), read in the tick from the touches that started on it. |

<a id="guide-components-39"></a>
### `components §39` `<viewport>` (`viewport`): A camera's view of a scene layer in the layout (`<viewport src={$target} camera-orbit="30deg 75deg 3m" bind:clientWidth={$w}>`): the render target `src` names, its camera written as `<model-viewer>` writes one (`camera-controls` orbits, zooms and flings it within `min-camera-orbit` and `max-camera-orbit`), its pointer events carrying the ray into its scene

A camera's view of a scene layer in the layout (`<viewport src={$target} camera-orbit="30deg 75deg 3m" bind:clientWidth={$w}>`): the render target `src` names, its camera written as `<model-viewer>` writes one (`camera-controls` orbits, zooms and flings it within `min-camera-orbit` and `max-camera-orbit`), its pointer events carrying the ray into its scene. A leaf; text-bearing controls accept text content: `<viewport id="Preview" src={$target} camera-orbit="30deg 75deg 3m" bind:clientWidth={$w} onpointermove={orbit} />`.

Its own props, beside every element's:

| Write | Value | What |
|---|---|---|
| `viewport-src` | `number` | The render target its camera draws into (`src={$target}`, the target's colour texture handle). |
| `camera-orbit` | `text` | `<model-viewer>`'s camera-orbit: yaw, pitch from straight up, and distance from its target (`30deg 75deg 3m`; default `0deg 75deg 3m`), for its pointer events' rays. With `camera-controls`, `bind:cameraOrbit={$orbit}` reads the orbit back as the player turns it (draw the scene from it). Two-way only with an explicit bind: attribute. |
| `camera-controls` | `bool` | `<model-viewer>`'s camera-controls: a drag orbits the camera (a fling keeps it turning, slowing to rest; still under reduced motion), the wheel and a two-finger pinch zoom it, and with the focus (Tab, a pad) Left and Right turn it 15° and Up and Down tilt it 10°, within `min-camera-orbit` and `max-camera-orbit`. The orbit is UI state; a new authored `camera-orbit` takes over. |
| `min-camera-orbit` | `text` | `<model-viewer>`'s min-camera-orbit: the least yaw, pitch and distance camera-controls reach, each a value or `auto` (`auto 40deg 1.5m`; default `auto 22.5deg auto`). |
| `max-camera-orbit` | `text` | `<model-viewer>`'s max-camera-orbit: the most yaw, pitch and distance (`auto 100deg 6m`; default `auto 157.5deg auto`). |
| `camera-target` | `text` | `<model-viewer>`'s camera-target: the point it orbits (`0m 1m 0m`, the default). |
| `field-of-view` | `text` | `<model-viewer>`'s field-of-view, vertical (`50deg`, the default). |
| `client-width` | `number` | Svelte's `bind:clientWidth`: its laid-out width, CSS px, written when it changes (size its render target by it times the pixel ratio). Two-way only with an explicit bind: attribute. |
| `client-height` | `number` | Svelte's `bind:clientHeight`: its laid-out height, CSS px. Two-way only with an explicit bind: attribute. |

<a id="guide-components-40"></a>
### `components §40` `<svg>` (`svg`): Inline SVG (`<svg viewBox>`): its shapes drawn in its box, the view box fitted by `xMidYMid meet`; its `width` and `height` are CSS px (default: the view box's size)

Inline SVG (`<svg viewBox>`): its shapes drawn in its box, the view box fitted by `xMidYMid meet`; its `width` and `height` are CSS px (default: the view box's size). A container with nested children: `<svg id="Name" viewBox="0 0 40 40">…</svg>`.

Its own props, beside every element's:

| Write | Value | What |
|---|---|---|
| `viewBox` | `text` | SVG `viewBox`: the user space `min-x min-y width height` its box shows. |
| `fill` | `text` | SVG `fill`: a colour, `var(--role)`, `currentColor` (the style's ink), `url(#gradient)` or `none`. |
| `fill-opacity` | `text` | SVG `fill-opacity`. |
| `fill-rule` | `text` | SVG `fill-rule`: nonzero or evenodd. |
| `stroke` | `text` | SVG `stroke`: a colour, `var(--role)`, `currentColor`, `url(#gradient)` or `none`. |
| `stroke-opacity` | `text` | SVG `stroke-opacity`. |
| `stroke-width` | `text` | SVG `stroke-width`, user units (scaled with the view box). |
| `stroke-dasharray` | `text` | SVG `stroke-dasharray`: dash and gap lengths, an odd list repeated (user units, or of `pathLength`). |
| `stroke-dashoffset` | `text` | SVG `stroke-dashoffset`. |
| `stroke-linecap` | `text` | SVG `stroke-linecap`. |
| `stroke-linejoin` | `text` | SVG `stroke-linejoin`. |
| `stroke-miterlimit` | `text` | SVG `stroke-miterlimit`. |
| `x` | `text` | SVG `x`. |
| `y` | `text` | SVG `y`. |
| `font-size` | `text` | SVG `font-size`, user units (scaled with the view box; default 16). |
| `font-weight` | `text` | SVG `font-weight`: normal, bold or 100–900 (600 and up is bold). |
| `font-style` | `text` | SVG `font-style`: normal or italic. |
| `text-anchor` | `text` | SVG `text-anchor`: start, middle or end of the line on `x`. |
| `dominant-baseline` | `text` | SVG `dominant-baseline`: auto (alphabetic), middle, central, hanging, text-before-edge, text-after-edge, ideographic or mathematical on `y`. |
| `width` | `text` | SVG `width` (a `<rect>`, `<use>`, `<pattern>`, `<mask>`). |
| `height` | `text` | SVG `height`. |
| `cx` | `text` | SVG `cx`. |
| `cy` | `text` | SVG `cy`. |
| `r` | `text` | SVG `r`. |
| `x1` | `text` | SVG `x1`. |
| `y1` | `text` | SVG `y1`. |
| `x2` | `text` | SVG `x2`. |
| `y2` | `text` | SVG `y2`. |
| `marker-start` | `text` | SVG `marker-start="url(#id)"`: the `<marker>` drawn at a shape's first vertex. |
| `marker-mid` | `text` | SVG `marker-mid`: the `<marker>` drawn at every vertex between the first and the last. |
| `marker-end` | `text` | SVG `marker-end`: the `<marker>` drawn at a shape's last vertex (an arrowhead). |
| `preserveAspectRatio` | `text` | SVG `preserveAspectRatio` of a `<symbol>`, a `<pattern>` or an `<image>`. |

<a id="guide-components-41"></a>
### `components §41` `<g>` (`svgGroup`): SVG's `<g>`: a transform and presentation attributes its shapes inherit

SVG's `<g>`: a transform and presentation attributes its shapes inherit. A container with nested children: `<g transform="rotate(-90 20 20)" stroke="var(--accent)">…</g>`.

Its own props, beside every element's:

| Write | Value | What |
|---|---|---|
| `transform` | `text` | SVG `transform` (matrix, translate, scale, rotate, skewX, skewY); on a gradient, `gradientTransform`. |
| `fill` | `text` | SVG `fill`: a colour, `var(--role)`, `currentColor` (the style's ink), `url(#gradient)` or `none`. |
| `fill-opacity` | `text` | SVG `fill-opacity`. |
| `fill-rule` | `text` | SVG `fill-rule`: nonzero or evenodd. |
| `stroke` | `text` | SVG `stroke`: a colour, `var(--role)`, `currentColor`, `url(#gradient)` or `none`. |
| `stroke-opacity` | `text` | SVG `stroke-opacity`. |
| `stroke-width` | `text` | SVG `stroke-width`, user units (scaled with the view box). |
| `stroke-dasharray` | `text` | SVG `stroke-dasharray`: dash and gap lengths, an odd list repeated (user units, or of `pathLength`). |
| `stroke-dashoffset` | `text` | SVG `stroke-dashoffset`. |
| `stroke-linecap` | `text` | SVG `stroke-linecap`. |
| `stroke-linejoin` | `text` | SVG `stroke-linejoin`. |
| `stroke-miterlimit` | `text` | SVG `stroke-miterlimit`. |
| `x` | `text` | SVG `x`. |
| `y` | `text` | SVG `y`. |
| `font-size` | `text` | SVG `font-size`, user units (scaled with the view box; default 16). |
| `font-weight` | `text` | SVG `font-weight`: normal, bold or 100–900 (600 and up is bold). |
| `font-style` | `text` | SVG `font-style`: normal or italic. |
| `text-anchor` | `text` | SVG `text-anchor`: start, middle or end of the line on `x`. |
| `dominant-baseline` | `text` | SVG `dominant-baseline`: auto (alphabetic), middle, central, hanging, text-before-edge, text-after-edge, ideographic or mathematical on `y`. |
| `width` | `text` | SVG `width` (a `<rect>`, `<use>`, `<pattern>`, `<mask>`). |
| `height` | `text` | SVG `height`. |
| `cx` | `text` | SVG `cx`. |
| `cy` | `text` | SVG `cy`. |
| `r` | `text` | SVG `r`. |
| `x1` | `text` | SVG `x1`. |
| `y1` | `text` | SVG `y1`. |
| `x2` | `text` | SVG `x2`. |
| `y2` | `text` | SVG `y2`. |
| `marker-start` | `text` | SVG `marker-start="url(#id)"`: the `<marker>` drawn at a shape's first vertex. |
| `marker-mid` | `text` | SVG `marker-mid`: the `<marker>` drawn at every vertex between the first and the last. |
| `marker-end` | `text` | SVG `marker-end`: the `<marker>` drawn at a shape's last vertex (an arrowhead). |
| `clip-path` | `text` | SVG `clip-path="url(#id)"`: the `<clipPath>` whose shapes cut it. |
| `mask` | `text` | SVG `mask="url(#id)"`: the `<mask>` whose luminance cuts it (white shapes, or one gradient-filled shape). |
| `filter` | `text` | SVG `filter="url(#id)"`: the `<filter>` the element and its subtree are drawn through. |

<a id="guide-components-42"></a>
### `components §42` `<path>`, `<circle>`, `<rect>`, `<polygon>` (`svgShape`): An SVG shape (`<path>`, `<rect>`, `<circle>`, `<ellipse>`, `<line>`, `<polyline>`, `<polygon>`): one element drawn as an analytic path, its attributes bindable

An SVG shape (`<path>`, `<rect>`, `<circle>`, `<ellipse>`, `<line>`, `<polyline>`, `<polygon>`): one element drawn as an analytic path, its attributes bindable. A leaf; text-bearing controls accept text content: `<path d={$line} fill="none" stroke="currentColor" />` or `<circle cx="20" cy="20" r="18" pathLength="1" stroke-dasharray="1 1" stroke-dashoffset={1 - $ready} />` or `<rect x={b.x} y={40 - b.h} width="8" height={b.h} rx="2" />` or `<polygon points="0,0 10,0 5,8" /> (ellipse, line, polyline)`.

Its own props, beside every element's:

| Write | Value | What |
|---|---|---|
| `transform` | `text` | SVG `transform` (matrix, translate, scale, rotate, skewX, skewY); on a gradient, `gradientTransform`. |
| `fill` | `text` | SVG `fill`: a colour, `var(--role)`, `currentColor` (the style's ink), `url(#gradient)` or `none`. |
| `fill-opacity` | `text` | SVG `fill-opacity`. |
| `fill-rule` | `text` | SVG `fill-rule`: nonzero or evenodd. |
| `stroke` | `text` | SVG `stroke`: a colour, `var(--role)`, `currentColor`, `url(#gradient)` or `none`. |
| `stroke-opacity` | `text` | SVG `stroke-opacity`. |
| `stroke-width` | `text` | SVG `stroke-width`, user units (scaled with the view box). |
| `stroke-dasharray` | `text` | SVG `stroke-dasharray`: dash and gap lengths, an odd list repeated (user units, or of `pathLength`). |
| `stroke-dashoffset` | `text` | SVG `stroke-dashoffset`. |
| `stroke-linecap` | `text` | SVG `stroke-linecap`. |
| `stroke-linejoin` | `text` | SVG `stroke-linejoin`. |
| `stroke-miterlimit` | `text` | SVG `stroke-miterlimit`. |
| `pathLength` | `text` | SVG `pathLength`: the length its dashes and offset measure the outline in (`pathLength="1"` makes `stroke-dashoffset` a fraction). |
| `d` | `text` | A `<path>`'s path data. |
| `x` | `text` | SVG `x`. |
| `y` | `text` | SVG `y`. |
| `width` | `text` | SVG `width` (a `<rect>`, `<use>`, `<pattern>`, `<mask>`). |
| `height` | `text` | SVG `height`. |
| `rx` | `text` | SVG `rx`. |
| `ry` | `text` | SVG `ry`. |
| `cx` | `text` | SVG `cx`. |
| `cy` | `text` | SVG `cy`. |
| `r` | `text` | SVG `r`. |
| `x1` | `text` | SVG `x1`. |
| `y1` | `text` | SVG `y1`. |
| `x2` | `text` | SVG `x2`. |
| `y2` | `text` | SVG `y2`. |
| `points` | `text` | A `<polyline>`'s or `<polygon>`'s points. |
| `marker-start` | `text` | SVG `marker-start="url(#id)"`: the `<marker>` drawn at a shape's first vertex. |
| `marker-mid` | `text` | SVG `marker-mid`: the `<marker>` drawn at every vertex between the first and the last. |
| `marker-end` | `text` | SVG `marker-end`: the `<marker>` drawn at a shape's last vertex (an arrowhead). |
| `href` | `text` | SVG `href`: an `<image>`'s atlas icon by name or render texture handle (`href={$target}`), what a `<use>` draws (`#id` of a symbol, group or shape), the gradient a gradient takes its stops from, or an `feImage`'s atlas image, texture handle or element (`#id` of a shape or group, drawn as `<use>` draws it). |
| `clip-path` | `text` | SVG `clip-path="url(#id)"`: the `<clipPath>` whose shapes cut it. |
| `mask` | `text` | SVG `mask="url(#id)"`: the `<mask>` whose luminance cuts it (white shapes, or one gradient-filled shape). |
| `clip-rule` | `text` | SVG `clip-rule` of a shape in a `<clipPath>`: nonzero or evenodd (a ring cuts a hole). |
| `filter` | `text` | SVG `filter="url(#id)"`: the `<filter>` the element and its subtree are drawn through. |
| `preserveAspectRatio` | `text` | SVG `preserveAspectRatio` of a `<symbol>`, a `<pattern>` or an `<image>`. |

<a id="guide-components-43"></a>
### `components §43` `<svgText>` (`svgText`): SVG's `<text x y text-anchor dominant-baseline font-size white-space>`: a line shaped by og-ui's text, its anchor on the point fitted into the `<svg>`'s box and scaled with the view box; `<tspan>` runs inside it take `font-weight`, `font-style`, `text-decoration` and `fill`, a `<tspan x>` (with `y` or `dy`) starts another line, and a `<textPath>` runs it along a path

SVG's `<text x y text-anchor dominant-baseline font-size white-space>`: a line shaped by og-ui's text, its anchor on the point fitted into the `<svg>`'s box and scaled with the view box; `<tspan>` runs inside it take `font-weight`, `font-style`, `text-decoration` and `fill`, a `<tspan x>` (with `y` or `dy`) starts another line, and a `<textPath>` runs it along a path. A leaf; text-bearing controls accept text content: `<svgText id="Name">…</svgText>`.

Its own props, beside every element's:

| Write | Value | What |
|---|---|---|
| `fill` | `text` | SVG `fill`: a colour, `var(--role)`, `currentColor` (the style's ink), `url(#gradient)` or `none`. |
| `fill-opacity` | `text` | SVG `fill-opacity`. |
| `fill-rule` | `text` | SVG `fill-rule`: nonzero or evenodd. |
| `stroke` | `text` | SVG `stroke`: a colour, `var(--role)`, `currentColor`, `url(#gradient)` or `none`. |
| `stroke-opacity` | `text` | SVG `stroke-opacity`. |
| `stroke-width` | `text` | SVG `stroke-width`, user units (scaled with the view box). |
| `stroke-dasharray` | `text` | SVG `stroke-dasharray`: dash and gap lengths, an odd list repeated (user units, or of `pathLength`). |
| `stroke-dashoffset` | `text` | SVG `stroke-dashoffset`. |
| `stroke-linecap` | `text` | SVG `stroke-linecap`. |
| `stroke-linejoin` | `text` | SVG `stroke-linejoin`. |
| `stroke-miterlimit` | `text` | SVG `stroke-miterlimit`. |
| `x` | `text` | SVG `x`. |
| `y` | `text` | SVG `y`. |
| `dx` | `text` | A `<text>`'s `dx` (added to `x`, user units or `em`), or `feOffset`'s and `feDropShadow`'s. |
| `dy` | `text` | A `<text>`'s `dy` (added to `y`, user units or `em`; `1.2em`: a line down), or `feOffset`'s and `feDropShadow`'s. |
| `font-size` | `text` | SVG `font-size`, user units (scaled with the view box; default 16). |
| `font-weight` | `text` | SVG `font-weight`: normal, bold or 100–900 (600 and up is bold). |
| `font-style` | `text` | SVG `font-style`: normal or italic. |
| `text-anchor` | `text` | SVG `text-anchor`: start, middle or end of the line on `x`. |
| `dominant-baseline` | `text` | SVG `dominant-baseline`: auto (alphabetic), middle, central, hanging, text-before-edge, text-after-edge, ideographic or mathematical on `y`. |
| `width` | `text` | SVG `width` (a `<rect>`, `<use>`, `<pattern>`, `<mask>`). |
| `height` | `text` | SVG `height`. |
| `cx` | `text` | SVG `cx`. |
| `cy` | `text` | SVG `cy`. |
| `r` | `text` | SVG `r`. |
| `x1` | `text` | SVG `x1`. |
| `y1` | `text` | SVG `y1`. |
| `x2` | `text` | SVG `x2`. |
| `y2` | `text` | SVG `y2`. |
| `marker-start` | `text` | SVG `marker-start="url(#id)"`: the `<marker>` drawn at a shape's first vertex. |
| `marker-mid` | `text` | SVG `marker-mid`: the `<marker>` drawn at every vertex between the first and the last. |
| `marker-end` | `text` | SVG `marker-end`: the `<marker>` drawn at a shape's last vertex (an arrowhead). |
| `startOffset` | `text` | A `<textPath>`'s `startOffset`: where the text's anchor sits along the path, user units or a percentage of its length. |
| `side` | `text` | A `<textPath>`'s `side`: left (the default) or right (the path run backwards). |
| `method` | `text` | A `<textPath>`'s `method`: align (each glyph turned rigidly onto the path; stretch draws as align, as browsers do). |
| `spacing` | `text` | A `<textPath>`'s `spacing`: exact or auto (both keep the font's advances). |
| `path` | `text` | A `<textPath>`'s `path`: its own path data instead of `href`. |
| `svg-text-path-href` | `text` | A `<textPath>`'s `href="#id"`: the path or shape the text runs along. |

<a id="guide-components-44"></a>
### `components §44` `<foreignObject>` (`svgForeign`): SVG's `<foreignObject x y width height>`: document elements laid out in a box of `width` by `height` user units at `(x, y)`, scaled with the view box as the shapes are

SVG's `<foreignObject x y width height>`: document elements laid out in a box of `width` by `height` user units at `(x, y)`, scaled with the view box as the shapes are. A container with nested children: `<foreignObject x="10" y="10" width="80" height="30"><button onclick={buy}>Buy</button></foreignObject>`.

Its own props, beside every element's:

| Write | Value | What |
|---|---|---|
| `fill` | `text` | SVG `fill`: a colour, `var(--role)`, `currentColor` (the style's ink), `url(#gradient)` or `none`. |
| `fill-opacity` | `text` | SVG `fill-opacity`. |
| `fill-rule` | `text` | SVG `fill-rule`: nonzero or evenodd. |
| `stroke` | `text` | SVG `stroke`: a colour, `var(--role)`, `currentColor`, `url(#gradient)` or `none`. |
| `stroke-opacity` | `text` | SVG `stroke-opacity`. |
| `stroke-width` | `text` | SVG `stroke-width`, user units (scaled with the view box). |
| `stroke-dasharray` | `text` | SVG `stroke-dasharray`: dash and gap lengths, an odd list repeated (user units, or of `pathLength`). |
| `stroke-dashoffset` | `text` | SVG `stroke-dashoffset`. |
| `stroke-linecap` | `text` | SVG `stroke-linecap`. |
| `stroke-linejoin` | `text` | SVG `stroke-linejoin`. |
| `stroke-miterlimit` | `text` | SVG `stroke-miterlimit`. |
| `x` | `text` | SVG `x`. |
| `y` | `text` | SVG `y`. |
| `width` | `text` | SVG `width` (a `<rect>`, `<use>`, `<pattern>`, `<mask>`). |
| `height` | `text` | SVG `height`. |
| `cx` | `text` | SVG `cx`. |
| `cy` | `text` | SVG `cy`. |
| `r` | `text` | SVG `r`. |
| `x1` | `text` | SVG `x1`. |
| `y1` | `text` | SVG `y1`. |
| `x2` | `text` | SVG `x2`. |
| `y2` | `text` | SVG `y2`. |
| `marker-start` | `text` | SVG `marker-start="url(#id)"`: the `<marker>` drawn at a shape's first vertex. |
| `marker-mid` | `text` | SVG `marker-mid`: the `<marker>` drawn at every vertex between the first and the last. |
| `marker-end` | `text` | SVG `marker-end`: the `<marker>` drawn at a shape's last vertex (an arrowhead). |

<a id="guide-components-45"></a>
### `components §45` `<use>` (`svgUse`): SVG's `<use href="#id" x y width height>`: the symbol, group or shape `#id` drawn again here, inheriting this element's presentation attributes; a `<symbol>`'s view box is fitted into its width and height

SVG's `<use href="#id" x y width height>`: the symbol, group or shape `#id` drawn again here, inheriting this element's presentation attributes; a `<symbol>`'s view box is fitted into its width and height. A container with nested children: `<use href="#Coin" x={i * 12} y="0" width="10" height="10" />`.

Its own props, beside every element's:

| Write | Value | What |
|---|---|---|
| `fill` | `text` | SVG `fill`: a colour, `var(--role)`, `currentColor` (the style's ink), `url(#gradient)` or `none`. |
| `fill-opacity` | `text` | SVG `fill-opacity`. |
| `fill-rule` | `text` | SVG `fill-rule`: nonzero or evenodd. |
| `stroke` | `text` | SVG `stroke`: a colour, `var(--role)`, `currentColor`, `url(#gradient)` or `none`. |
| `stroke-opacity` | `text` | SVG `stroke-opacity`. |
| `stroke-width` | `text` | SVG `stroke-width`, user units (scaled with the view box). |
| `stroke-dasharray` | `text` | SVG `stroke-dasharray`: dash and gap lengths, an odd list repeated (user units, or of `pathLength`). |
| `stroke-dashoffset` | `text` | SVG `stroke-dashoffset`. |
| `stroke-linecap` | `text` | SVG `stroke-linecap`. |
| `stroke-linejoin` | `text` | SVG `stroke-linejoin`. |
| `stroke-miterlimit` | `text` | SVG `stroke-miterlimit`. |
| `pathLength` | `text` | SVG `pathLength`: the length its dashes and offset measure the outline in (`pathLength="1"` makes `stroke-dashoffset` a fraction). |
| `d` | `text` | A `<path>`'s path data. |
| `x` | `text` | SVG `x`. |
| `y` | `text` | SVG `y`. |
| `width` | `text` | SVG `width` (a `<rect>`, `<use>`, `<pattern>`, `<mask>`). |
| `height` | `text` | SVG `height`. |
| `rx` | `text` | SVG `rx`. |
| `ry` | `text` | SVG `ry`. |
| `cx` | `text` | SVG `cx`. |
| `cy` | `text` | SVG `cy`. |
| `r` | `text` | SVG `r`. |
| `x1` | `text` | SVG `x1`. |
| `y1` | `text` | SVG `y1`. |
| `x2` | `text` | SVG `x2`. |
| `y2` | `text` | SVG `y2`. |
| `points` | `text` | A `<polyline>`'s or `<polygon>`'s points. |
| `marker-start` | `text` | SVG `marker-start="url(#id)"`: the `<marker>` drawn at a shape's first vertex. |
| `marker-mid` | `text` | SVG `marker-mid`: the `<marker>` drawn at every vertex between the first and the last. |
| `marker-end` | `text` | SVG `marker-end`: the `<marker>` drawn at a shape's last vertex (an arrowhead). |
| `href` | `text` | SVG `href`: an `<image>`'s atlas icon by name or render texture handle (`href={$target}`), what a `<use>` draws (`#id` of a symbol, group or shape), the gradient a gradient takes its stops from, or an `feImage`'s atlas image, texture handle or element (`#id` of a shape or group, drawn as `<use>` draws it). |
| `clip-path` | `text` | SVG `clip-path="url(#id)"`: the `<clipPath>` whose shapes cut it. |
| `mask` | `text` | SVG `mask="url(#id)"`: the `<mask>` whose luminance cuts it (white shapes, or one gradient-filled shape). |
| `clip-rule` | `text` | SVG `clip-rule` of a shape in a `<clipPath>`: nonzero or evenodd (a ring cuts a hole). |
| `filter` | `text` | SVG `filter="url(#id)"`: the `<filter>` the element and its subtree are drawn through. |

<a id="guide-components-46"></a>
### `components §46` `<linearGradient>`, `<symbol>` (`svgDef`): An SVG definition (`<defs>`, `<symbol>`, `<clipPath>`, `<mask>`, `<pattern>`, `<linearGradient>`, `<radialGradient>`, `<stop>`, `<title>`, `<desc>`): not drawn where written; `<use>`, `clip-path`, `mask` and paints reference them by `#id`, and a `<title>` names its parent

An SVG definition (`<defs>`, `<symbol>`, `<clipPath>`, `<mask>`, `<pattern>`, `<linearGradient>`, `<radialGradient>`, `<stop>`, `<title>`, `<desc>`): not drawn where written; `<use>`, `clip-path`, `mask` and paints reference them by `#id`, and a `<title>` names its parent. A container with nested children: `<linearGradient id="Up" x2="0" y2="1"><stop offset="0" stop-color="#fff" />…</linearGradient> (radialGradient, defs)` or `<symbol id="Coin" viewBox="0 0 24 24"><circle cx="12" cy="12" r="10" /></symbol> (clipPath, mask, pattern, title)`.

Its own props, beside every element's:

| Write | Value | What |
|---|---|---|
| `viewBox` | `text` | SVG `viewBox`: the user space `min-x min-y width height` its box shows. |
| `transform` | `text` | SVG `transform` (matrix, translate, scale, rotate, skewX, skewY); on a gradient, `gradientTransform`. |
| `fill` | `text` | SVG `fill`: a colour, `var(--role)`, `currentColor` (the style's ink), `url(#gradient)` or `none`. |
| `fill-opacity` | `text` | SVG `fill-opacity`. |
| `fill-rule` | `text` | SVG `fill-rule`: nonzero or evenodd. |
| `stroke` | `text` | SVG `stroke`: a colour, `var(--role)`, `currentColor`, `url(#gradient)` or `none`. |
| `stroke-opacity` | `text` | SVG `stroke-opacity`. |
| `stroke-width` | `text` | SVG `stroke-width`, user units (scaled with the view box). |
| `stroke-dasharray` | `text` | SVG `stroke-dasharray`: dash and gap lengths, an odd list repeated (user units, or of `pathLength`). |
| `stroke-dashoffset` | `text` | SVG `stroke-dashoffset`. |
| `stroke-linecap` | `text` | SVG `stroke-linecap`. |
| `stroke-linejoin` | `text` | SVG `stroke-linejoin`. |
| `stroke-miterlimit` | `text` | SVG `stroke-miterlimit`. |
| `x` | `text` | SVG `x`. |
| `y` | `text` | SVG `y`. |
| `dx` | `text` | A `<text>`'s `dx` (added to `x`, user units or `em`), or `feOffset`'s and `feDropShadow`'s. |
| `dy` | `text` | A `<text>`'s `dy` (added to `y`, user units or `em`; `1.2em`: a line down), or `feOffset`'s and `feDropShadow`'s. |
| `width` | `text` | SVG `width` (a `<rect>`, `<use>`, `<pattern>`, `<mask>`). |
| `height` | `text` | SVG `height`. |
| `cx` | `text` | SVG `cx`. |
| `cy` | `text` | SVG `cy`. |
| `r` | `text` | SVG `r`. |
| `x1` | `text` | SVG `x1`. |
| `y1` | `text` | SVG `y1`. |
| `x2` | `text` | SVG `x2`. |
| `y2` | `text` | SVG `y2`. |
| `marker-start` | `text` | SVG `marker-start="url(#id)"`: the `<marker>` drawn at a shape's first vertex. |
| `marker-mid` | `text` | SVG `marker-mid`: the `<marker>` drawn at every vertex between the first and the last. |
| `marker-end` | `text` | SVG `marker-end`: the `<marker>` drawn at a shape's last vertex (an arrowhead). |
| `refX` | `text` | A `<marker>`'s `refX`: the x of its content placed on the vertex. |
| `refY` | `text` | A `<marker>`'s `refY`. |
| `markerWidth` | `text` | A `<marker>`'s `markerWidth` (default 3). |
| `markerHeight` | `text` | A `<marker>`'s `markerHeight` (default 3). |
| `markerUnits` | `text` | A `<marker>`'s `markerUnits`: strokeWidth (scaled by the shape's stroke width, the default) or userSpaceOnUse. |
| `orient` | `text` | A `<marker>`'s `orient`: auto (along the path), auto-start-reverse, or an angle in degrees. |
| `fx` | `text` | A radial gradient's `fx`. |
| `fy` | `text` | A radial gradient's `fy`. |
| `offset` | `text` | A `<stop>`'s `offset`. |
| `stop-color` | `text` | A `<stop>`'s `stop-color`. |
| `stop-opacity` | `text` | A `<stop>`'s `stop-opacity`. |
| `gradientUnits` | `text` | A gradient's `gradientUnits`. |
| `gradientTransform` | `text` | A gradient's `gradientTransform`. |
| `spreadMethod` | `text` | A gradient's `spreadMethod`. |
| `href` | `text` | SVG `href`: an `<image>`'s atlas icon by name or render texture handle (`href={$target}`), what a `<use>` draws (`#id` of a symbol, group or shape), the gradient a gradient takes its stops from, or an `feImage`'s atlas image, texture handle or element (`#id` of a shape or group, drawn as `<use>` draws it). |
| `filterUnits` | `text` | A `<filter>`'s `filterUnits`: `objectBoundingBox` (default; its region in fractions of the box) or `userSpaceOnUse`. |
| `primitiveUnits` | `text` | A `<filter>`'s `primitiveUnits`: `userSpaceOnUse` (default) or `objectBoundingBox`. |
| `color-interpolation-filters` | `text` | A `<filter>`'s or primitive's `color-interpolation-filters`: `linearRGB` (default) or `sRGB`. |
| `in` | `text` | A filter primitive's `in`: `SourceGraphic`, `SourceAlpha` or an earlier `result` (left out: the previous result). |
| `in2` | `text` | A filter primitive's second input `in2` (`feComposite`, `feBlend`, `feDisplacementMap`). |
| `result` | `text` | A filter primitive's `result` name, read by later `in`/`in2`. |
| `stdDeviation` | `text` | `feGaussianBlur`'s or `feDropShadow`'s `stdDeviation`: one or two numbers. |
| `flood-color` | `text` | `feFlood`'s or `feDropShadow`'s `flood-color`. |
| `flood-opacity` | `text` | `feFlood`'s or `feDropShadow`'s `flood-opacity`. |
| `operator` | `text` | `feComposite`'s `operator` (over, in, out, atop, xor, lighter, arithmetic) or `feMorphology`'s (erode, dilate). |
| `k1` | `text` | `feComposite` arithmetic's `k1`. |
| `k2` | `text` | `feComposite` arithmetic's `k2`. |
| `k3` | `text` | `feComposite` arithmetic's `k3`. |
| `k4` | `text` | `feComposite` arithmetic's `k4`. |
| `mode` | `text` | `feBlend`'s `mode`: a `mix-blend-mode` name. |
| `type` | `text` | `feColorMatrix`'s `type` (matrix, saturate, hueRotate, luminanceToAlpha), a `feFunc*`'s (identity, table, discrete, linear, gamma) or `feTurbulence`'s (turbulence, fractalNoise). |
| `values` | `text` | `feColorMatrix`'s `values`. |
| `tableValues` | `text` | A `feFunc*`'s `tableValues` (at most 16 are read). |
| `slope` | `text` | A linear `feFunc*`'s `slope`. |
| `intercept` | `text` | A linear `feFunc*`'s `intercept`. |
| `amplitude` | `text` | A gamma `feFunc*`'s `amplitude`. |
| `exponent` | `text` | A gamma `feFunc*`'s `exponent`. |
| `radius` | `text` | `feMorphology`'s `radius`: one or two numbers. |
| `scale` | `text` | `feDisplacementMap`'s `scale`. |
| `xChannelSelector` | `text` | `feDisplacementMap`'s `xChannelSelector`: R, G, B or A. |
| `yChannelSelector` | `text` | `feDisplacementMap`'s `yChannelSelector`: R, G, B or A. |
| `baseFrequency` | `text` | `feTurbulence`'s `baseFrequency`: one or two numbers. |
| `numOctaves` | `text` | `feTurbulence`'s `numOctaves`. |
| `seed` | `text` | `feTurbulence`'s `seed`. |
| `stitchTiles` | `text` | `feTurbulence`'s `stitchTiles`: stitch or noStitch. |
| `order` | `text` | `feConvolveMatrix`'s `order`: one or two numbers. |
| `kernelMatrix` | `text` | `feConvolveMatrix`'s `kernelMatrix` (at most 64 values). |
| `divisor` | `text` | `feConvolveMatrix`'s `divisor`. |
| `bias` | `text` | `feConvolveMatrix`'s `bias`. |
| `targetX` | `text` | `feConvolveMatrix`'s `targetX`. |
| `targetY` | `text` | `feConvolveMatrix`'s `targetY`. |
| `edgeMode` | `text` | `feConvolveMatrix`'s `edgeMode`: duplicate, wrap or none. |
| `preserveAlpha` | `text` | `feConvolveMatrix`'s `preserveAlpha`. |
| `kernelUnitLength` | `text` | `kernelUnitLength` of `feConvolveMatrix` and the lighting primitives: a cell's size in primitive units (left out, one device px). |
| `surfaceScale` | `text` | A lighting primitive's `surfaceScale`. |
| `diffuseConstant` | `text` | `feDiffuseLighting`'s `diffuseConstant`. |
| `specularConstant` | `text` | `feSpecularLighting`'s `specularConstant`. |
| `specularExponent` | `text` | `feSpecularLighting`'s or `feSpotLight`'s `specularExponent`. |
| `lighting-color` | `text` | A lighting primitive's `lighting-color`. |
| `azimuth` | `text` | `feDistantLight`'s `azimuth`, degrees. |
| `elevation` | `text` | `feDistantLight`'s `elevation`, degrees. |
| `z` | `text` | `fePointLight`'s or `feSpotLight`'s `z`. |
| `pointsAtX` | `text` | `feSpotLight`'s `pointsAtX`. |
| `pointsAtY` | `text` | `feSpotLight`'s `pointsAtY`. |
| `pointsAtZ` | `text` | `feSpotLight`'s `pointsAtZ`. |
| `limitingConeAngle` | `text` | `feSpotLight`'s `limitingConeAngle`, degrees. |
| `clipPathUnits` | `text` | A clip path's `clipPathUnits`. |
| `maskContentUnits` | `text` | A mask's `maskContentUnits`. |
| `patternUnits` | `text` | A pattern's `patternUnits`. |
| `patternContentUnits` | `text` | A pattern's `patternContentUnits`. |
| `patternTransform` | `text` | A pattern's `patternTransform`. |
| `preserveAspectRatio` | `text` | SVG `preserveAspectRatio` of a `<symbol>`, a `<pattern>` or an `<image>`. |
