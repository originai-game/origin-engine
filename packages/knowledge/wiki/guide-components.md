<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `components` guide: The UI component catalogue

Core elements and their reflected properties for scriptless Svelte documents. Unknown elements, properties, and values fail cooking. The `screens` guide describes contracts, bindings, styles, and content composition.

Write `<button id="Buy" onclick={buy}>Buy</button>`. Put CSS declarations such as `gap: 8px` in a scoped `<style>` block. Bind attributes with `value={$amount}` and writable controls with `bind:value={$amount}`. Use `<text>` for the reflected label kind and `<img>` for image. Named content uses snippets and `{@render name()}`, not legacy slot elements.

<a id="guide-components-1"></a>
### `components §1` Every element

What every kind takes: its box, layout, paint, motion and input.

| Write | Value | What |
|---|---|---|
| `width:` | `fit-content \| stretch \| <n>px \| <n>% (min-/max- bounds)` | Its width (fit by default). |
| `height:` | `fit-content \| stretch \| <n>px \| <n>% (min-/max- bounds)` | Its height (fit by default). |
| `padding:` | `1 to 4 lengths (top right bottom left)` | Space inside its edges, CSS px. |
| `gap:` | `<n>px` | Space between its children, CSS px. |
| `justify-content:` | `start \| center \| end \| space-between` | Where children sit along the main axis. |
| `align-items:` | `start \| center \| end \| stretch \| baseline` | Where children sit across it. |
| `flex-wrap:` | `wrap \| nowrap` | A row breaks its children into lines. |
| `background:` | `var(--<role>) \| #rrggbb[aa]` | Its fill. |
| `border-radius:` | `<n>px` | Its corner radius, CSS px. |
| `-og-frame:` | `panel \| card \| button \| button-hover \| button-pressed \| button-disabled \| field \| slot \| tooltip \| hud \| track \| fill` | The style frame its background takes. |
| `aspect-ratio:` | `<number>` | Width over height (0: none). |
| `interactive` | `bool` | It reports clicks (`on click`). |
| `focusable` | `bool` | With interactive: keys and pads reach it and press it. |
| `adjustable` | `bool` | With focusable: Left and Right step it while it has the focus instead of moving the focus. |
| `visible` | `bool` | False leaves it (and its subtree) out of the tree. |
| `aria-label` | `text` | What a screen reader calls it. |
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
| `grid-template-columns:` | `120px 1fr auto 25% minmax(80px, 200px)` | A grid's column tracks. |
| `grid-template-rows:` | `120px 1fr auto 25% minmax(80px, 200px)` | A grid's row tracks. |
| `grid-template-areas:` | `"<a>" "<b>" …` | A grid's named areas, one string per row. |
| `grid-auto-flow:` | `row \| column [dense]` | `flow`: How a grid auto-places. `dense`: A grid back-fills holes. |
| `grid-area:` | `<name>` | The named grid area it takes. |
| `grid-column:`, `grid-row:` | `<n> \| <n> / span <n> \| span <n>` | `cell`: Its grid column and row, 1-based. `span`: Columns and rows it spans. |
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
| `cursor:` | `auto \| default \| pointer \| text \| grab \| grabbing \| move \| not-allowed \| crosshair \| help \| wait \| progress \| ew-resize \| ns-resize \| nesw-resize \| nwse-resize \| zoom-in \| zoom-out \| none` | The pointer's shape over it and its subtree. |
| `customize` | `bool` | The player can move, scale and fade it in the HUD edit mode. |
| `customize-key` | `name` | With customize: its name in the saved HUD layout (default: its path). |
| `-og-reduced-motion:` | `fade \| none \| keep` | Its reduced-motion fallback. |

Style interaction with `#Buy:hover { scale: 1.04 }`. The screens guide lists state selectors and their scoped cascade. Bind game state with `class:active={$selected}` and style `.active`. Custom properties inherit computed values from the mounted parent; `var(--name, fallback)` resolves after specificity and `!important`. CSS motion uses shared timing values; markup contains no state blocks or timelines. `transition` and its longhands (`transition-property`, `-duration`, `-timing-function`, `-delay`) and `animation` and its longhands (`animation-name`, `-duration`, `-timing-function`, `-delay`, `-iteration-count`, `-direction`, `-fill-mode`, `-play-state`) follow CSS Transitions and Animations Level 1 over the animatable values: opacity, translate, scale, rotate, -og-skew, border-radius, background, color and z-index (keyframes animate the scalar ones).

<a id="guide-components-2"></a>
### `components §2` `<column>` (`column`): Children top to bottom

Children top to bottom. A container with nested children: `<column id="Name">…</column>`.

Its own props, beside every element's:

| Write | Value | What |
|---|---|---|
| `flex-direction:` | `row \| column` | Lays a row's or a column's children out the other way (an `at` block's: a phone in landscape puts the actions beside the content). |
| `-og-frame:` | `rive("<file>.riv#<Artboard>")` | The Rive artboard it plays in its box, `<file>.riv#<Artboard>` relative to assets/ (`-og-frame: rive("ui/chest.riv#Chest")`, docs/ui.md §3.3): loaded with its document, advanced on the tick, drawn over its background and saved with the UI's state; its drawn objects are elements for dump_ui, describe and pick. |

<a id="guide-components-3"></a>
### `components §3` `<row>` (`row`): Children left to right (mirrored right to left)

Children left to right (mirrored right to left). A container with nested children: `<row id="Name">…</row>`.

Its own props, beside every element's:

| Write | Value | What |
|---|---|---|
| `flex-direction:` | `row \| column` | Lays a row's or a column's children out the other way (an `at` block's: a phone in landscape puts the actions beside the content). |
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
### `components §10` `<text>` (`label`): A text

A text. A leaf; text-bearing controls accept text content: `<text id="Name">…</text>`.

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
| `text-overflow:` | `ellipsis \| clip` | Cut with … where it does not fit. |
| `font-family:` | `var(--font-<name>)` | The font role. |
| `heading` | `bool` | A heading (a style's flourish frames it). |

<a id="guide-components-11"></a>
### `components §11` `<button>` (`button`): A button with a label

A button with a label. A leaf; text-bearing controls accept text content: `<button id="Name">…</button>`.

Its own props, beside every element's:

| Write | Value | What |
|---|---|---|
| `text` | `text` | What it says. |
| `i18n` | `name` | A string table key (og.i18n) its text comes from; `text` is the fallback. |
| `variant` | `secondary \| primary \| quiet \| danger \| ghost` | The button's role. |
| `size` | `small \| medium \| large` | The control height. |
| `disabled` | `bool` | Takes no input. |
| `autofocus` | `bool` | Takes the focus when it appears. |
| `square` | `bool` | A square of the control height: an icon button. |

<a id="guide-components-12"></a>
### `components §12` `<img>` (`image`): A texture (or a tint)

A texture (or a tint). A leaf; text-bearing controls accept text content: `<img id="Name" />`.

Its own props, beside every element's:

| Write | Value | What |
|---|---|---|
| `src` | `number` | A texture handle (og.load.texture), usually bound. |
| `-og-tint:` | `var(--<role>) \| #rrggbb[aa]` | Multiplies the texture (or fills without one). |

<a id="guide-components-13"></a>
### `components §13` `<art>` (`art`): An art slot (docs/ui.md §3.3): a spec the art pipeline reads, its texture once there is one

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

<a id="guide-components-14"></a>
### `components §14` `<icon>` (`icon`): An icon of the style's set

An icon of the style's set. A leaf; text-bearing controls accept text content: `<icon id="Name">…</icon>`.

Its own props, beside every element's:

| Write | Value | What |
|---|---|---|
| `color:` | `var(--<role>) \| #rrggbb[aa]` | Its ink. |
| `secondary` | `name` | The second icon layer, in the same atlas. |
| `-og-secondary-tint:` | `var(--<role>) \| #rrggbb[aa]` | The second layer's ink; accent by default. |
| `icon` | `name` | The icon's name in the style's set. |
| `icon-size` | `number` | CSS px (24 by default). |

<a id="guide-components-15"></a>
### `components §15` `<progress>` (`progress`): A progress bar

A progress bar. A leaf; text-bearing controls accept text content: `<progress id="Name">…</progress>`.

Its own props, beside every element's:

| Write | Value | What |
|---|---|---|
| `value` | `number` | Its value (a slider writes it back). Two-way only with an explicit bind: attribute. |

<a id="guide-components-16"></a>
### `components §16` `<input type="range">` (`slider`): A slider (two-way `value`)

A slider (two-way `value`). A leaf; text-bearing controls accept text content: `<input type="range" id="Name" />`.

Its own props, beside every element's:

| Write | Value | What |
|---|---|---|
| `disabled` | `bool` | Takes no input. |
| `value` | `number` | Its value (a slider writes it back). Two-way only with an explicit bind: attribute. |
| `min` | `number` | The range's low end; a number input under it is invalid (`rangeUnderflow`). |
| `max` | `number` | The range's high end; a number input over it is invalid (`rangeOverflow`). |
| `show-value` | `bool` | Show the slider's value readout (default true); false when an adjacent label supplies units. |
| `step` | `number` | A slider snaps to multiples of this (0: continuous). A number input's value off the step grid from `min` is invalid (default 1; `any` or 0: no step). |

<a id="guide-components-17"></a>
### `components §17` `<input type="checkbox" switch>`, `<label>` (`toggle`): A switch with a label (two-way `checked`)

A switch with a label (two-way `checked`). A leaf; text-bearing controls accept text content: `<input type="checkbox" switch id="Name" />` or `<label id="Name"><input type="checkbox" /> Caption</label>`.

Its own props, beside every element's:

| Write | Value | What |
|---|---|---|
| `text` | `text` | What it says. |
| `i18n` | `name` | A string table key (og.i18n) its text comes from; `text` is the fallback. |
| `disabled` | `bool` | Takes no input. |
| `checked` | `bool` | Whether it is on (written back). Two-way only with an explicit bind: attribute. |

<a id="guide-components-18"></a>
### `components §18` `<tabs>` (`tabs`): A tab list (two-way `selected`), using the core tabs widget

A tab list (two-way `selected`), using the core tabs widget. A leaf; text-bearing controls accept text content: `<tabs id="Name">…</tabs>`.

Its own props, beside every element's:

| Write | Value | What |
|---|---|---|
| `text` | `text` | What it says. |
| `i18n` | `name` | A string table key (og.i18n) its text comes from; `text` is the fallback. |
| `items` | `names` | The tab labels, literal names or a bound list of strings. |
| `keys` | `names` | A string table key (og.i18n) per tab its label comes from; its item is the fallback. |
| `values` | `names` | A name per tab (a screen's mode) that `current` holds instead of `selected`'s index. |
| `current` | `name` | The selected tab's name from `values`; it takes over from `selected` when `values` is set. Two-way only with an explicit bind: attribute. |
| `selected` | `number` | The selected tab's zero-based index. Two-way only with an explicit bind: attribute. |
| `fill` | `bool` | Equal tabs filling the available width. |

<a id="guide-components-19"></a>
### `components §19` `<input type="text">`, `<textarea>` (`field`): A text control (two-way `input`): `<input>` for one line, `<textarea>` for several

A text control (two-way `input`): `<input>` for one line, `<textarea>` for several. A leaf; text-bearing controls accept text content: `<input type="text" id="Name" />` or `<textarea id="Name" rows="4" />`.

Its own props, beside every element's:

| Write | Value | What |
|---|---|---|
| `disabled` | `bool` | Takes no input. |
| `min` | `number` | The range's low end; a number input under it is invalid (`rangeUnderflow`). |
| `max` | `number` | The range's high end; a number input over it is invalid (`rangeOverflow`). |
| `step` | `number` | A slider snaps to multiples of this (0: continuous). A number input's value off the step grid from `min` is invalid (default 1; `any` or 0: no step). |
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
| `invalid` | `bool` | Its text is wrong: the danger frame. |
| `type` | `text \| password \| number \| email \| url \| tel \| search` | What it takes, by HTML's names: `password` is drawn masked with a reveal toggle (`<component>.reveal[revealed]`) and never copied or cut; `number` takes only a number's characters; a `number`, `email` or `url` whose text is not one matches `:invalid`. Each picks its virtual keyboard. |
| `inputmode` | `none \| text \| decimal \| numeric \| tel \| search \| email \| url` | The virtual keyboard a touch device shows (HTML `inputmode`; `none`: the game shows its own); unset, its `type`'s. |
| `autocomplete` | `name` | The page's autofill hint (HTML `autocomplete`: `username`, `current-password`, `new-password`, `one-time-code`, `email`, `off`…). |

<a id="guide-components-20"></a>
### `components §20` `<badge>` (`badge`): A small toned label

A small toned label. A leaf; text-bearing controls accept text content: `<badge id="Name">…</badge>`.

Its own props, beside every element's:

| Write | Value | What |
|---|---|---|
| `text` | `text` | What it says. |
| `i18n` | `name` | A string table key (og.i18n) its text comes from; `text` is the fallback. |
| `tone` | `neutral \| info \| success \| warn \| danger \| accent` | Its tone. |

<a id="guide-components-21"></a>
### `components §21` `<spacer>` (`spacer`): Empty room (grows by default)

Empty room (grows by default). A leaf; text-bearing controls accept text content: `<spacer id="Name">…</spacer>`.

It takes every element's props and none of its own.

<a id="guide-components-22"></a>
### `components §22` `<hr>` (`divider`): A hairline

A hairline. A leaf; text-bearing controls accept text content: `<hr id="Name" />`.

Its own props, beside every element's:

| Write | Value | What |
|---|---|---|
| `vertical` | `bool` | Upright. |
