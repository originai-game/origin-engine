<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `components` guide: The UI component catalogue

Core elements and their reflected properties for scriptless Svelte documents. Unknown elements, properties, and values fail cooking. The `screens` guide describes contracts, bindings, styles, and content composition.

Write `<button id="Buy" onclick={buy}>Buy</button>`. Put CSS declarations such as `gap: 8px` in a scoped `<style>` block. Bind attributes with `value={$amount}` and writable controls with `bind:value={$amount}`. Use `<text>` for the reflected label kind and `<img>` for image. Named content uses snippets and `{@render name()}`, not legacy slot elements.

<a id="guide-components-1"></a>
### `components §1` Every element

What every kind takes: its box, layout, paint, motion and input.

| Write | Value | What |
|---|---|---|
| `width:` | `auto \| fit-content \| fit-content(<n>px) \| min-content \| max-content \| stretch \| <n>px \| <n>% \| calc(…) \| min(…) \| max(…) \| clamp(…) \| round(…) …` | Its width (fit by default). |
| `height:` | `auto \| fit-content \| fit-content(<n>px) \| min-content \| max-content \| stretch \| <n>px \| <n>% \| calc(…) \| min(…) \| max(…) \| clamp(…) \| round(…) …` | Its height (fit by default). |
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
| `scrollbar-width:` | `auto \| thin \| none` | Its scroll bars: none hides them (it still scrolls). |
| `-og-frame:` | `panel \| card \| button \| button-hover \| button-pressed \| button-disabled \| field \| slot \| tooltip \| hud \| track \| fill` | The style frame its background takes. |
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
| `translate:` | `<x>px [<y>px]` | `x`: Moved right after layout, CSS px. `y`: Moved down after layout, CSS px. |
| `scale:` | `<both> \| <x> <y>` | `scale`: Both scales. `scaleX`: Its horizontal scale. `scaleY`: Its vertical scale. |
| `rotate:` | `<n>deg \| <n>rad \| <n>turn` | A turn, radians, clockwise. |
| `-og-skew:` | `<x>deg [<y>deg]` | `skewX`: A skew, radians. `skewY`: A skew, radians. |
| `transform-origin:` | `<x>% <y>%` | The transform's origin, fractions of its box. |
| `opacity:` | `<number>` | 0 to 1, multiplied down the tree. |
| `z-index:` | `<number>` | Drawing order among its siblings. |
| `position:` | `static \| absolute \| fixed \| -og-anchored` | In the flow, placed by its insets in its parent, by its insets on the whole screen (full bleed, under the safe area), or against another element. |
| `inset-inline-start:` | `<n>px` | The inset from its parent's start edge (absolute). |
| `top:` | `<n>px` | The inset from the top. |
| `inset-inline-end:` | `<n>px` | The inset from the end edge. |
| `bottom:` | `<n>px` | The inset from the bottom. |
| `-og-center-x:` | `true \| false` | Centred across its parent. |
| `-og-center-y:` | `true \| false` | Centred down its parent. |
| `-og-anchor:` | `"<text>"` | The path of the element an anchored element sits against. |
| `-og-anchor-at:` | `top-left \| top \| top-right \| left \| center \| right \| bottom-left \| bottom \| bottom-right` | The target's point it sits on. |
| `-og-anchor-self:` | `top-left \| top \| top-right \| left \| center \| right \| bottom-left \| bottom \| bottom-right` | Its own point that sits there. |
| `-og-hit-slop:` | `<n>px` | Extra pointer reach, CSS px. |
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
| `cursor:` | `[url(<icon or texture>) [<x> <y>],]* auto \| default \| none \| context-menu \| help \| pointer \| progress \| wait \| cell \| crosshair \| text \| vertical-text \| alias \| copy \| move \| no-drop \| not-allowed \| grab \| grabbing \| all-scroll \| col-resize \| row-resize \| n-resize \| e-resize \| s-resize \| w-resize \| ne-resize \| nw-resize \| se-resize \| sw-resize \| ew-resize \| ns-resize \| nesw-resize \| nwse-resize \| zoom-in \| zoom-out` | CSS `cursor`: the pointer's shape over it and its subtree, any CSS keyword. `url(<icon or texture>) [<x> <y>]` images before it are accepted and kept; the keyword is what the host shows. |
| `pointer-events:` | `auto \| none \| visiblePainted \| visibleFill \| visibleStroke \| visible \| painted \| fill \| stroke \| bounding-box \| all` | CSS `pointer-events`, inherited: none lets the pointer through to what is under it (keys and pads still reach it; a child saying auto still takes it). On SVG shapes the SVG values pick the fill, the stroke or the bounding box. |
| `touch-action:` | `auto \| none \| [pan-x \| pan-left \| pan-right] \|\| [pan-y \| pan-up \| pan-down] \|\| pinch-zoom \| manipulation` | CSS `touch-action`: the pans (pan-x, pan-left…, pan-y…) and pinch-zoom a touch starting on it may make; none keeps the touch for its own handlers. The used value intersects its ancestors'. |
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
| `data-hold` | `number` | How long a press is held before `onhold` fires, in ms (`data-hold`; default 500). |
| `display:` | `block \| inline \| inline-block \| flow-root \| list-item \| flex \| inline-flex \| grid \| inline-grid \| table \| inline-table \| contents \| table-row-group \| table-header-group \| table-footer-group \| table-row \| table-cell \| table-caption \| table-column \| table-column-group` | Its CSS `display`. On a box its layout: `block` stacks its children top to bottom (the default), `flex` lays them out as a row (or `flex-direction`), `grid` in tracks, `table` (and its parts) as a table, `flowRoot` a block whose children's margins stay inside, `contents` no box at all (its children are its parent's). On any element `inline…` puts it in lines beside its inline-level siblings. |
| `vertical-align:` | `baseline \| top \| middle \| bottom \| text-top \| text-bottom \| sub \| super` | CSS `vertical-align`: where an inline-level element sits in its line, and a table cell's content in its row (`middle` by default). |
| `caption-side:` | `top \| bottom` | CSS `caption-side`: a table caption above (default) or below its table. |
| `-og-emitter:` | `<n>px \| <n>%` | `-og-emitter: <rate> [square\|circle\|diamond] [<size>px] [<colour>…]`: particles a second from the element while it stands (`.rare { -og-emitter: 12 diamond 6px #ffd166 #fff; }`), drawn over it; `none` stops. Silent under reduced motion. |
| `-og-meta:` | `<n>px \| <n>%` | `-og-meta: vignette [<colour>] \| tint <colour>`: a full-screen overlay drawn with the element, never taking input, its strength the element's `opacity` (`#Hurt { -og-meta: vignette #a00; opacity: var(--hit); }`). |
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
| `backdrop-filter:` | `blur(<n>px) brightness(<n>) contrast(<n>) drop-shadow([<colour>] <x> <y> [<blur>]) grayscale(<n>) hue-rotate(<angle>) invert(<n>) opacity(<n>) saturate(<n>) sepia(<n>) … \| none` | CSS `backdrop-filter`: what is under the element blurred and its colour adjusted (glass), with `filter`'s functions; the colour functions fold into one matrix and `drop-shadow()` casts nothing on the opaque backdrop. |
| `filter:` | `blur(<n>px) brightness(<n>) contrast(<n>) drop-shadow([<colour>] <x> <y> [<blur>]) grayscale(<n>) hue-rotate(<angle>) invert(<n>) opacity(<n>) saturate(<n>) sepia(<n>) … \| none` | CSS `filter`: `blur()`, `brightness()`, `contrast()`, `drop-shadow()` (of the alpha, not the box), `grayscale()`, `hue-rotate()`, `invert()`, `opacity()`, `saturate()` and `sepia()` on the element and its subtree, applied in the order written (`filter: drop-shadow(0 4px 6px #0008) saturate(1.4)`). |
| `mix-blend-mode:` | `normal \| multiply \| screen \| plus-lighter \| overlay \| soft-light \| color-dodge` | CSS `mix-blend-mode`: normal, multiply, screen, plus-lighter, overlay, soft-light or color-dodge. |
| `text-shadow:` | `<x> <y> [<blur>] [<colour>], … \| none` | CSS `text-shadow: <x> <y> [<blur>] [<colour>], …` on the element's text. |
| `clip-path:` | `inset(…) \| circle(…) \| ellipse(…) \| polygon(…) \| path("…") \| none` | CSS `clip-path`: `inset()`, `circle()`, `ellipse()`, `polygon()` or `path()` cuts the element, its background and its children to the shape (by its path's coverage, either fill rule). |
| `-og-material:` | `<material>(<n> …[, <colour>]) \| none` | `-og-material: <name>(<n> …[, <colour>])`: the element and its subtree composited through one of the engine's built-in materials (dissolve, shine, hologram, glitch, ripple, pixelate, color-adjust, outline, blur), its parameters in the material's order. No CSS standard has shader materials. |
| `scroll-snap-type:` | `none \| x \| y \| block \| inline \| both [mandatory \| proximity]` | CSS `scroll-snap-type: x \| y \| both [mandatory \| proximity]` on a scroll container: a drag, fling or wheel settles on the snap position of the area nearest where it would stop (`proximity`: only within a third of the view). |
| `scroll-snap-align:` | `none \| start \| center \| end [<inline>]` | CSS `scroll-snap-align: start \| center \| end` (block, then inline) of an area inside a snapping scroll container: the edge or centre that lines up with its view's. |

Style interaction with `#Buy:hover { scale: 1.04 }`. The screens guide lists state selectors and their scoped cascade. Bind game state with `class:active={$selected}` and style `.active`. Custom properties inherit computed values from the mounted parent; `var(--name, fallback)` resolves after specificity and `!important`. CSS motion uses shared timing values; markup contains no state blocks or timelines. `transition` and its longhands (`transition-property`, `-duration`, `-timing-function`, `-delay`) and `animation` and its longhands (`animation-name`, `-duration`, `-timing-function`, `-delay`, `-iteration-count`, `-direction`, `-fill-mode`, `-play-state`) follow CSS Transitions and Animations Level 1 over the animatable values: opacity, translate, scale, rotate, -og-skew, border-radius, background, color and z-index (keyframes animate the scalar ones).

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
### `components §10` `<text>`, `<p>`, `<span>`, `<h1>`, `<a>` (`label`): A text

A text. A leaf; text-bearing controls accept text content: `<text id="Name">…</text>` or `<p id="Name">Read the <a href="rules">rules</a>.</p>` or `<span id="Name">…</span>` or `<h1 id="Name">…</h1> (to <h6>)` or `<a id="Name" href="shop">…</a>`.

Its own props, beside every element's:

| Write | Value | What |
|---|---|---|
| `text` | `text` | What it says. |
| `i18n` | `name` | A string table key (og.i18n) its text comes from; `text` is the fallback. |
| `font:` | `var(--text-<name>)` | The type step. |
| `color:` | `var(--<role>) \| #rrggbb[aa]` | Its ink. |
| `spans` | `TextSpan[]` | Read-only TextSpan[] ranges over plain text; offsets count Unicode scalars. Styles come from ::part(name). |
| `-og-on-scene:` | `true \| false` | Scene ink and separation shadow. |
| `text-transform:` | `uppercase \| none` | Uppercase in the UI's locale, as a type role with `case upper` is. |
| `text-align:` | `start \| center \| end` | How its lines align. |
| `white-space:` | `nowrap \| normal` | One line. |
| `line-clamp:` | `<number>` | The most lines it shows. |
| `-og-fit-min:` | `var(--text-<name>)` | Shrinks by type steps to fit its width, down to this step. |
| `text-overflow:` | `ellipsis \| clip` | Cut with … where it does not fit. `ellipsisText`: What its cut ends in instead of … (`text-overflow: "<string>"`). |
| `font-family:` | `var(--font-<name>)` | The font role. |
| `heading` | `bool` | A heading (a style's flourish frames it). |
| `count` | `number` | A number it counts to as it changes (`count={$score}`), over `-og-count-duration` ms. |
| `-og-count-duration:` | `<number>` | How long a change of its count takes, in ms (`-og-count-duration`; 0: 900). |
| `-og-count-decimals:` | `<number>` | Digits its count shows after the point (`-og-count-decimals`). |
| `-og-reveal:` | `<number>` | Types its text out at this many characters a second (`-og-reveal`); `onrevealed` fires when it is whole. |
| `href` | `text` | An `<a>`'s destination: a click sends a `link` event named by it (as `onclick` fires). |

<a id="guide-components-11"></a>
### `components §11` `<button>` (`button`): A button: its text, or its content (an icon, a text, a badge) in a row inside its frame and states

A button: its text, or its content (an icon, a text, a badge) in a row inside its frame and states. A container with nested children: `<button id="Name">…</button>`.

Its own props, beside every element's:

| Write | Value | What |
|---|---|---|
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
| `data-action` | `name` | The action whose glyph it shows on the device used last (its key, its pad button, or its touch icon); it follows the device live. |
| `glyph-device` | `text` | The device it draws for (bound to `$ui.device` by the parser: a device change rebuilds it). |
| `data-og-rebind` | `bool` | A press captures the next key as its binding (`bind:value`), Escape cancels. |
| `binding` | `text` | With dataOgRebind: the key code it captured (`bind:value`), written back. Two-way only with an explicit bind: attribute. |
| `text` | `text` | What it says. |
| `i18n` | `name` | A string table key (og.i18n) its text comes from; `text` is the fallback. |

<a id="guide-components-13"></a>
### `components §13` `<img>` (`image`): A texture (or a tint)

A texture (or a tint). A leaf; text-bearing controls accept text content: `<img id="Name" />`.

Its own props, beside every element's:

| Write | Value | What |
|---|---|---|
| `src` | `number` | A texture handle (og.load.texture), usually bound. |
| `-og-tint:` | `var(--<role>) \| #rrggbb[aa]` | Multiplies the texture (or fills without one). |
| `object-view-box:` | `inset(<t>% <r>% <b>% <l>%) \| -og-uv: <u0> <v0> <u1> <v1>` | The part of the texture it shows, u0 v0 u1 v1 with v down (`object-view-box: inset(…)` in percentages, or `-og-uv`). |

<a id="guide-components-14"></a>
### `components §14` `<art>` (`art`): An art slot (docs/ui.md §3.3): a spec the art pipeline reads, its texture once there is one

An art slot (docs/ui.md §3.3): a spec the art pipeline reads, its texture once there is one. A leaf; text-bearing controls accept text content: `<art id="Name">…</art>`.

Its own props, beside every element's:

| Write | Value | What |
|---|---|---|
| `src` | `number` | A texture handle (og.load.texture), usually bound. |
| `-og-tint:` | `var(--<role>) \| #rrggbb[aa]` | Multiplies the texture (or fills without one). |
| `kind` | `name` | The slot's kind (Appendix F: itemIcon, bust, keyArt…). |
| `ratio` | `x y` | The slot's aspect, width and height in whole numbers. |
| `focal` | `x y` | The point cropping keeps, 0–1. |
| `safe` | `4 numbers` | Where text or controls lie over it: x0 y0 x1 y1, 0–1. |
| `object-fit:` | `cover \| contain \| fill \| nine-slice \| tile` | How the art fills the slot. |
| `item-cells` | `x y` | An item icon's grid cells, columns and rows (1 1, 1 2, 2 3 or 2 4). |
| `slice` | `insets` | A nine-slice's insets as fractions of the art: all, x y, or left top right bottom. |
| `art-set` | `name` | The set it is generated with for a consistent look (the kind's name by default). |

<a id="guide-components-15"></a>
### `components §15` `<icon>` (`icon`): An icon of the style's set

An icon of the style's set. A leaf; text-bearing controls accept text content: `<icon id="Name">…</icon>`.

Its own props, beside every element's:

| Write | Value | What |
|---|---|---|
| `color:` | `var(--<role>) \| #rrggbb[aa]` | Its ink. |
| `secondary` | `name` | The second icon layer, in the same atlas. |
| `-og-secondary-tint:` | `var(--<role>) \| #rrggbb[aa]` | The second layer's ink; accent by default. |
| `icon` | `name` | The icon's name in the style's set. |
| `icon-size` | `number` | CSS px (24 by default). |
| `-og-cue:` | `<name>` | A colour token whose cue it draws (`-og-cue: danger`, docs/ui.md §3.2): the style's icon for that status, rarity, team or damage type, so its meaning never rests on colour alone. Its `icon`, when set, wins. |

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
| `-og-fill-origin:` | `<number>` | The value its fill grows from (`-og-fill-origin`; default min): 0 on a -1 to 1 balance fills either way from the centre. |
| `mark-values` | `names` | The values of the `<datalist>` its `list` names: a tick at each. |
| `mark-labels` | `names` | The marks' labels, under the track (an `<option label>`'s, or its text); empty for a tick only. |
| `step` | `number` | A slider snaps to multiples of this (0: continuous). A number input's value off the step grid from `min` is invalid (default 1; `any` or 0: no step). |
| `low` | `number` | A meter's top of the low region (HTML `low`; default `min`). On a range input with `high`, its low knob (a slider writes it back). Two-way only with an explicit bind: attribute. |
| `high` | `number` | A meter's bottom of the high region (HTML `high`; default `max`). On a range input, its high knob: two knobs on one track, `low` to `high` (a slider writes both back). Two-way only with an explicit bind: attribute. |
| `-og-min-gap:` | `<number>` | The least distance between a range input's two knobs (`-og-min-gap`, in its values; default 0). |

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
| `values` | `names` | A name per tab (a screen's mode) that `current` holds instead of `selected`'s index. |
| `current` | `name` | The selected item's name from `values` (a `<select>`'s `value`); on tabs it takes over from `selected` when `values` is set. Two-way only with an explicit bind: attribute. |
| `selected` | `number` | The selected tab's zero-based index. Two-way only with an explicit bind: attribute. |
| `fill` | `bool` | Equal tabs filling the available width. |

<a id="guide-components-26"></a>
### `components §26` `<input type="text">`, `<textarea>` (`field`): A text control (two-way `input`): `<input>` for one line, `<textarea>` for several

A text control (two-way `input`): `<input>` for one line, `<textarea>` for several. A leaf; text-bearing controls accept text content: `<input type="text" id="Name" />` or `<textarea id="Name" rows="4" />`.

Its own props, beside every element's:

| Write | Value | What |
|---|---|---|
| `disabled` | `bool` | Takes no input. |
| `min` | `number` | The range's low end; a number input under it is invalid (`rangeUnderflow`). |
| `max` | `number` | The range's high end; a number input over it is invalid (`rangeOverflow`). |
| `step` | `number` | A slider snaps to multiples of this (0: continuous). A number input's value off the step grid from `min` is invalid (default 1; `any` or 0: no step). |
| `-og-stepper:` | `none \| buttons` | A number input drawn as minus, its value and plus (`-og-stepper: buttons`). |
| `appearance:` | `auto \| none` | none drops the control's own frame: its box, borders and background are the document's. |
| `suggestions` | `names` | A text input's suggestions, from the `<datalist>` its `list` names (its options' values, or its `items={$names}`): a list under the field while it has the focus, those containing the text first by prefix, choosing one writes it to the field. |
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
| `-og-select:` | `dropdown \| cycle` | cycle draws it as previous, its option and next (`-og-select`). |
| `items` | `names` | The tab or option labels, literal names or a bound list of strings (a `<select>`'s `<option>`s set them). |
| `keys` | `names` | A string table key (og.i18n) per tab its label comes from; its item is the fallback. |
| `values` | `names` | A name per tab (a screen's mode) that `current` holds instead of `selected`'s index. |
| `current` | `name` | The selected item's name from `values` (a `<select>`'s `value`); on tabs it takes over from `selected` when `values` is set. Two-way only with an explicit bind: attribute. |
| `groups` | `names` | An `<option>`'s `<optgroup>` label, one per option (empty: none); options of one group follow each other. |
| `disabled-items` | `names` | The `<option disabled>`s (and those of an `<optgroup disabled>`), by their zero-based index. |
| `placeholder` | `text` | Shown while it is empty. |
| `invalid` | `bool` | Its value is wrong (`aria-invalid`): the danger frame. |

<a id="guide-components-31"></a>
### `components §31` `<dialog>` (`dialog`): A dialog (two-way `open`): `modal` over a backdrop that traps the focus, else a panel in place; Escape or B closes it and the focus returns

A dialog (two-way `open`): `modal` over a backdrop that traps the focus, else a panel in place; Escape or B closes it and the focus returns. A container with nested children: `<dialog id="Name" modal bind:open={$x} aria-label="Quit?">…</dialog>`.

Its own props, beside every element's:

| Write | Value | What |
|---|---|---|
| `-og-spotlight:` | `<name>` | Open, it dims the page but for the element of this id, its content a callout against it (`-og-spotlight`). |
| `open` | `bool` | Whether it is open (HTML `open`; closing writes false back). Two-way only with an explicit bind: attribute. |
| `closedby` | `any \| closerequest \| none` | What closes it (HTML `closedby`): any adds a press outside a modal (light dismiss) to closerequest's Escape or a pad's B; none leaves closing to the document. Default: closerequest for a modal, none in place. |
| `appearance:` | `auto \| none` | none drops the control's own frame: its box, borders and background are the document's. |
| `modal` | `bool` | Over a backdrop that takes every press, the focus trapped inside (HTML `showModal()`); else a panel in place (`show()`). |
| `-og-sheet-detents:` | `<n>% \| <n>px …` | `-og-sheet-detents: 40% 90%` on a modal `<dialog>`: a bottom sheet docked to the bottom at full width, dragged onto its detents (heights, `%` of the viewport's) and swiped down to close (writing `open` back). It opens at the first. No CSS standard has detents. |

<a id="guide-components-32"></a>
### `components §32` `<details>` (`details`): A disclosure (two-way `open`): its `<summary>` row, its content shown only while open

A disclosure (two-way `open`): its `<summary>` row, its content shown only while open. A container with nested children: `<details id="Name" bind:open={$x}><summary>More</summary>…</details>`.

Its own props, beside every element's:

| Write | Value | What |
|---|---|---|
| `disabled` | `bool` | Takes no input. |
| `open` | `bool` | Whether it is open (HTML `open`; closing writes false back). Two-way only with an explicit bind: attribute. |
| `summary` | `text` | Its `<summary>`'s text; empty is `Details`. Written as the first child, `<summary>…</summary>`. |

<a id="guide-components-33"></a>
### `components §33` `<div>`, `<nav>`, `<ul>`, `<ol>` (`box`): A generic HTML block (`<div>`, `<section>`, `<nav>`, `<ul>`, `<li>`…): its layout is its CSS `display`, `block` (children top to bottom) by default, `flex` (a row, or `flex-direction`) or `grid`. Text inside it is an anonymous text element; an `<li>` draws its list marker

A generic HTML block (`<div>`, `<section>`, `<nav>`, `<ul>`, `<li>`…): its layout is its CSS `display`, `block` (children top to bottom) by default, `flex` (a row, or `flex-direction`) or `grid`. Text inside it is an anonymous text element; an `<li>` draws its list marker. A container with nested children: `<div id="Name">…</div>` or `<nav aria-label="Site">…</nav> (section, header, footer, main, aside, article)` or `<ul><li>…</li></ul>` or `<ol start="1"><li>…</li></ol>`.

Its own props, beside every element's:

| Write | Value | What |
|---|---|---|
| `flex-direction:` | `row \| column \| row-reverse \| column-reverse` | Lays a row's or a column's children out the other way (an `at` block's: a phone in landscape puts the actions beside the content). |
| `colspan` | `number` | A table cell's columns (`<td colspan>`), 1 by default; CSS table layout reads it. |
| `rowspan` | `number` | A table cell's rows (`<td rowspan>`), 1 by default, 0 to the end of its row group; CSS table layout reads it. |
| `-og-marker-type:` | `none \| disc \| circle \| square \| decimal \| lower-alpha \| upper-alpha \| lower-roman \| upper-roman` | An `<li>`'s marker, from the inherited `list-style-type` (`<ul>`: disc, nested circle then square; `<ol>`: decimal). |
| `-og-marker-position:` | `outside \| inside` | Where an `<li>`'s marker sits, from the inherited `list-style-position`: `outside` hangs it before the item's start edge (the default), `inside` lays the item out as a row after it. |

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
### `components §39` `<viewport>` (`viewport`): A camera's view of a scene layer in the layout (`<viewport src={$target} camera-orbit="30deg 75deg 3m" bind:clientWidth={$w}>`): the render target `src` names, its camera written as `<model-viewer>` writes one, its pointer events carrying the ray into its scene

A camera's view of a scene layer in the layout (`<viewport src={$target} camera-orbit="30deg 75deg 3m" bind:clientWidth={$w}>`): the render target `src` names, its camera written as `<model-viewer>` writes one, its pointer events carrying the ray into its scene. A leaf; text-bearing controls accept text content: `<viewport id="Preview" src={$target} camera-orbit="30deg 75deg 3m" bind:clientWidth={$w} onpointermove={orbit} />`.

Its own props, beside every element's:

| Write | Value | What |
|---|---|---|
| `viewport-src` | `number` | The render target its camera draws into (`src={$target}`, the target's colour texture handle). |
| `camera-orbit` | `text` | `<model-viewer>`'s camera-orbit: yaw, pitch from straight up, and distance from its target (`30deg 75deg 3m`; default `0deg 75deg 3m`), for its pointer events' rays. |
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
| `href` | `text` | SVG `href`: an `<image>`'s atlas icon by name or render texture handle (`href={$target}`), what a `<use>` draws (`#id` of a symbol, group or shape), or the gradient a gradient takes its stops from. |
| `clip-path` | `text` | SVG `clip-path="url(#id)"`: the `<clipPath>` whose shapes cut it. |
| `mask` | `text` | SVG `mask="url(#id)"`: the `<mask>` whose luminance cuts it (white shapes, or one gradient-filled shape). |
| `clip-rule` | `text` | SVG `clip-rule` of a shape in a `<clipPath>`: nonzero or evenodd (a ring cuts a hole). |
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
| `dx` | `text` | A `<text>`'s `dx`: added to `x`, user units or `em`. |
| `dy` | `text` | A `<text>`'s `dy`: added to `y`, user units or `em` (`1.2em`: a line down). |
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
| `href` | `text` | SVG `href`: an `<image>`'s atlas icon by name or render texture handle (`href={$target}`), what a `<use>` draws (`#id` of a symbol, group or shape), or the gradient a gradient takes its stops from. |
| `clip-path` | `text` | SVG `clip-path="url(#id)"`: the `<clipPath>` whose shapes cut it. |
| `mask` | `text` | SVG `mask="url(#id)"`: the `<mask>` whose luminance cuts it (white shapes, or one gradient-filled shape). |
| `clip-rule` | `text` | SVG `clip-rule` of a shape in a `<clipPath>`: nonzero or evenodd (a ring cuts a hole). |

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
| `href` | `text` | SVG `href`: an `<image>`'s atlas icon by name or render texture handle (`href={$target}`), what a `<use>` draws (`#id` of a symbol, group or shape), or the gradient a gradient takes its stops from. |
| `clipPathUnits` | `text` | A clip path's `clipPathUnits`. |
| `maskContentUnits` | `text` | A mask's `maskContentUnits`. |
| `patternUnits` | `text` | A pattern's `patternUnits`. |
| `patternContentUnits` | `text` | A pattern's `patternContentUnits`. |
| `patternTransform` | `text` | A pattern's `patternTransform`. |
| `preserveAspectRatio` | `text` | SVG `preserveAspectRatio` of a `<symbol>`, a `<pattern>` or an `<image>`. |
