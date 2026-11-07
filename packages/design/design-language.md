# The design language of Origin Engine and origin-agent

One design language for the engine's own UI (og-ui's default style, the
engine's tools) and origin-agent's surfaces (its web, Stage and iOS
clients), decoupled from the platform's (`origingame/docs/design-language.md`
styles the portal, ops and mail; the owner, 2026-11-02). Values live in
[`tokens.json`](tokens.json); this document says how they are used. It was
seeded from the platform's language and tokens at origingame d25f1f15
(2026-11-02) and evolves here: §Changes from the seed lists where it
already differs.

**Where it binds.** The language binds the engine's **default style** alone
(`crates/og-ui/src/theme.rs`, generated from `tokens.json`; docs/ui.md
§3) and origin-agent. A game may draw in the default style, a built-in
pack or a style of its own (generated or hand-made, a look's `ui` line);
the engine requires none of them. A game's style may restyle, re-dress, re-shape
or replace anything here above the accessibility floor of docs/ui.md §6.1.

**Carriers.** og-ui reads the generated constants (palette, space, radii,
type ramp, densities, durations, springs, eases, press scale) and the
default style's part dressings; origin-agent's web components and its
SwiftUI views read the same file through their own generators. None of
them defines a second palette, scale or component grammar.

## Principles

The language is **editorial** (owner decision 2026-10-11, direction C: Apple
Arcade / App Store Today), for the public site and the creation workspace
alike.

- **Content leads, chrome floats.** Game art — covers, Stage frames, captured
  frames — is the largest thing on a page; its dominant colour spills softly
  into the surround (`--art-glow`), and text sits on art only over a scrim
  gradient. Chrome (titlebars, dock switchers, toolbars, composers, tab bars,
  menus, sheets) floats on glass above content. Real pages show only real
  covers and data; a game without art gets a gradient of two token hues,
  never stock art.
- **A flat neutral ladder.** Surfaces are neutral steps on a black (dark) or
  grouped-grey (light) ground, separated by spacing and a 0.5px hairline
  edge; nothing on the content layer is lit, glossed or raised (owner,
  2026-10-25: Apple keeps content flat and depth to the floating layer). Hairlines only where material separation
  is insufficient (tables, lists, split panes).
- **The chrome is neutral; meaning has hue.** The one primary is the
  high-contrast neutral (2026-11-02); selection is a neutral fill or light
  edge; only semantic and game roles are coloured.
- **Glass is for floating chrome only**: never on content cards or rows; at
  most three glass layers on screen; solid under `prefers-reduced-transparency`.
- **Editorial type.** A display face with optical sizing for large titles and
  key numbers; reading text at 15px; semibold only for titles; zero tracking
  for Chinese.
- **Unboxed reading.** Prose and conversation flow like an article; only typed
  evidence (parameters, frames, recordings, playtests, deliveries) gets a
  surface.
- **Symbols before words.** Functions are iconoir icons with a tooltip and an
  `aria-label`; words are for the person's intent, conclusions, titles and values.
- **Explanations behind a "?"**. Pages show labels, values, actions and
  one meta line; rules, calculations, limits and reasons live in an `InfoTip`
  (a popover from 768px, a bottom sheet below), reached by keyboard like any
  control. Errors stay visible with their reason and next step, and an
  irreversible consequence shows in one line at the confirm step — never in a tip.
- **One icon, one meaning; icons are neutral**. A glyph names one thing
  across the product and sits in `fg-muted`/`fg-subtle`; no colored icon tiles
  (the lint rejects them). Hues mark categories, levels and avatars, not icons.
- **Real content first.** A module with nothing real to show is hidden, or is
  one line and one action; never a placeholder grid or invented numbers.
- **Phone first.** Columns become grouped lists or horizontal scrollers below
  768px; sheets and popovers rise from the bottom.
- **State has a shape.** A dot, ring or tint plus motion: pulse = needs you,
  breathe = running, red = failed, hollow = planned. Never hue alone.
- **Honest data.** Unavailable, failed, stale and empty are explicit states.
  Motion never hides them and nothing animates toward a number that does not exist.

## Colour

Semantic tokens only (`bg`, `bg-subtle`, `surface`, `surface-raised`,
`surface-hover`, `fill`, `fill-strong`, `thumb`, `border`, `border-strong`,
`fg`, `fg-muted`, `fg-subtle`, `fg-inverse`, `accent*`, `focus`, `danger`,
`success`, `warn`, `info`, `stage*`, materials, `scrim`, twelve `hue-*`).
Light and dark are peers; dark is `data-theme="dark"` on any element, never
`dark:` variants or JSX conditionals.

- **The surface ladder** (E4). Dark: ground `bg` #000, `surface` #1c1c1e (cards,
  panes), `surface-raised` #2c2c2e (dialogs, menus, raised rows),
  `surface-hover` #3a3a3c. Light mirrors Apple's grouped backgrounds: ground
  #f2f2f7, white surfaces, `surface-hover` #e5e5ea. Surfaces take only a
  hairline (`shadow-raised`/`shadow-edge`: 0.5px, inside in dark, outside in
  light); no top highlight and no drop shadow on content.
- **Neutral fills** (`fill`, `fill-strong`; Apple's system fills, translucent
  so they work on any step): fields, secondary buttons, chips, segmented
  tracks, skeletons, hover and **selection**. A selected row, current
  navigation item or palette option sits on `fill-strong`; a segmented thumb
  is `thumb`; a selected filter chip inverts to `fg`.
- **The primary is neutral** (the owner, 2026-11-02: every game brings its
  own colours, so the default chrome carries no hue). `accent`,
  `accent-hover` and `accent-active` are the high-contrast neutral: a
  near-white fill (#e8e8ed) with dark ink (`accent-fg` #111113) on dark
  surfaces, a near-black fill with light ink on light ones. It marks the
  **one** primary action per surface (Start, Create, Publish, Play, the
  composer's send) by fill and weight, never by colour; secondaries stay on
  the translucent `fill`. Emphasised text uses `accent-ink` (the strongest
  ink), text on the primary `accent-fg`. `accent-soft` marks only tiny things
  (a badge, a dot, a step halo). A disabled primary drops to `fill`.
  Amber survives only where a role needs it (`gold`, `warn`, rarity hues).
- **Focus and selection** are a neutral light edge or glow: the `focus` ring
  is `fg` (non-text contrast ≥ 3:1, enforced), a selected card a light rim.
- **Meaning keeps its hues**: `danger`, `success`, `warn`, `info` and the game
  roles (`health`, `mana`, `xp`, `gold`) stay coloured, and a test fails the
  day the chrome takes a hue or a meaning loses one. Warning is orange and
  always paired with an icon.
- **Hues** colour categories, levels, badges, chart series and collections —
  never body text. Art-backed surfaces take their tint from the cover
  (`--art-glow`, `--tint`).
- `tests/tokens.test.ts` fails on any listed pair below 4.5:1 (text) or 3:1 (non-text) in
  either theme. The tokens test also measures every button variant's label
  over its fill as drawn (the `danger` tint at 10 % at rest and 15 % hovered,
  the translucent `fill`s, `fg` at 90 %) composited on `bg`, `bg-subtle`,
  `surface` and `surface-raised`, at rest, hovered and pressed, in both themes, at 4.5:1.
  `danger` is the danger ink (#b20011 light, #ff8a82 dark) dark or light
  enough to read on its own tint.

- **The game roles** are this language's own, beside the platform's: `health`,
  `mana`, `xp` and `gold` (meters and currency; `gold` is also an ink and holds
  4.5:1 on `surface` and `surface-raised`), `hud` (a capsule over the game,
  translucent; `fg` holds 4.5:1 on it over a mid-grey scene), `knob` (slider
  and switch knobs) and `sheen` (a skeleton's passing light). og-ui's 33
  roles map onto these keys one to one (`crates/og-ui/scripts/theme_codegen.rs`).
- **Linear light.** A browser mixes a translucent fill over encoded sRGB;
  the engine's UI pass blends flat fills in linear light, where a light tint
  over a dark ground reads about three times as strong (a white 13 % control
  over dark glass took a current item under 4.5:1). `linear` restates every
  translucent fill for the engine: the alpha whose linear blend over its
  stated ground (`linear.over`) draws what the CSS value draws there
  (`material-control` dark: 13 % in CSS, 3.9 % in linear light). og-ui's
  `control`, `controlStrong`, `primarySoft` and `scrim` take these. Glass
  tints, hairlines and shadows are mixed as CSS mixes them by the glass
  material itself, so they keep their CSS values.
- **Contrast over glass.** `glassContrast` holds text over glass at 4.5:1
  over the brightest and darkest scenes og-tests measures (#e4e4e4 under a
  bright sky, #151515 under a dark scene), a selected row's `fill-strong`
  included, in either scheme over either scene. `glass-thin` carries no
  text. Emphasised text (`accent-ink`) and the neutral primary hold 4.5:1 on
  solid surfaces and every glass weight; on a bar (`glass` regular) the
  current item is `fg` with a light glyph or indicator.

## Density and type

One ruler for every app. `data-density="compact"` (default) or
`"comfortable"` on any element; coarse pointers and widths under 768px raise
controls to touch size automatically.

| | compact | comfortable | touch | pad |
|---|---|---|---|---|
| controls `h-control-sm/md/lg` | 28/32/36 | 32/36/40 | 44/44/48 | 40/48/52 |
| table row `h-row` | 36 | 44 | 48 | 52 |
| page header `h-header` | 48 | 56 | 52 | 64 |

Interface text is 14px (`text-body`/`text-sm`), meta and table secondary text
13px (`text-meta`), **reading text** (conversation prose, descriptions, docs,
comments, Markdown) 15px with 24px leading (`text-reading`), minimum 12px.
Tools (Ops, Origin, portal developer pages) are compact; browsing surfaces
(home, games, community) are comfortable. Spacing is the 4px scale (`xxs` 2 …
`3xl` 48).

**The display face** (E2): `font-display` is Inter with its optical-size axis
(SIL OFL 1.1, self-hosted by the site as a Latin-only subset with
`font-display: swap`, never preloaded); Chinese falls back to the system face
(PingFang, Noto) in the same stack. Use it for large titles, featured tile
titles, rank numerals and key numbers (`numeric`: display face plus tabular
figures; `tabular-nums` for numbers in body text). Body text stays on the
system stack (SF Pro / PingFang, Segoe UI / YaHei, Roboto / Noto) with
`font-optical-sizing: auto`. `font-rounded` for levels and scores; Geist Mono
only for code, IDs, keys, prices and logs. Sentence case; no uppercase labels.

| Display step | Size / leading | Latin tracking | Chinese leading |
|---|---|---|---|---|
| `text-display-sm` | 28 / 32 | −0.012em | 38 |
| `text-display-md` | 34 / 38 | −0.016em | 44 |
| `text-display-lg` | 40 / 44 | −0.02em | 50 |
| `text-display-xl` | 48 / 52 | −0.022em | 60 |
| `text-display-2xl` | 56 / 60 | −0.025em | 68 |

The interface steps `text-xl` 20 … `text-8xl` 64 keep their own tracking.
Tracking belongs to the type step: Latin display steps tighten, Chinese
(`lang="zh…"`) is set at **zero tracking** with looser leading (`font.cjk`),
so no fixed `tracking-tight`.

One weight ladder: regular for body and metadata, medium for controls and
badges, semibold for titles and headlines, bold only for display titles
(`text-display-*`, `text-4xl` and up). Section headers are an eyebrow over a
big title (`SectionHeader eyebrow="10月11日星期日" title="今日"`).
`lint:design` rejects off-ladder weights, bold below display size, heavy
12–13px text and fixed negative tracking.

**In the engine** the densities follow the input: `compact` for a mouse,
`touch` on a coarse pointer or under 768 CSS px, and `pad` (the engine's,
`density.pad`) for a gamepad across a room. Lengths are og-ui units: on a
mouse or pad surface one unit grows with the surface's CSS height against
a 720 base (1 to 2), so 14 px body text is 21 px on a 1080p window and the
smallest step, 12 px, is 18 px there (docs/ui.md §6.1); touch surfaces keep
CSS px. The player's text scale (1 to 2) multiplies over it.

## Shape, elevation, material

**Radii** are one concentric scale, kept small (owner, 2026-10-25):
`rounded-control` 4, `rounded-field` 6, `rounded-block` 8, `rounded-card` 10
(cards, art tiles), `rounded-panel` 12 (panes, dialogs, docks),
`rounded-sheet` 16 (phone sheets), `rounded-button` 6 (buttons, segmented
controls, tabs, titlebar and tab bar items, toasts: rounded rectangles, not
pills); `rounded-capsule` only for tags (filter chips, status capsules);
`rounded-full` only for avatars and dots, `rounded-xs` only for data
marks. A nested element is concentric: inner radius = outer radius − inset
(`radius-outer-card` on the container, `rounded-inner-4` on a child inset by
16px; `rounded-[inherit]` for a full-bleed child). `lint:design` rejects
Tailwind's generic steps (`rounded-md`, `rounded-lg`, …) and arbitrary radii.

**Elevation**: `shadow-edge` and `shadow-raised` (the hairline alone; content
never casts a shadow), `shadow-float` (floating chrome: the hairline and one
soft, short shadow), `shadow-lift` (dialogs, sheets), `shadow-overlay`. Shadows are
theme-aware variables: `[data-theme]` remaps them like colours, and the
`shadow-*` utilities read them at the element.

**Materials** are flat glass, only on floating chrome: the material stays
clear and takes its colour from what passes under it; it has no specular
rim, sheen or pointer light. One rule (`.glass*` in the shared stylesheet)
draws every glass surface from three composable parts, so page CSS that draws
its own glass composes the same material: `--glass-tint` (the light tint),
`--glass-filter` (a modest blur with 200% saturation) and `--glass-rim` (a
0.5px `material-edge` hairline). Three weights: `glass-thin` (10px
blur; play discs and capsules over art), `glass` (22px; bars, tab bars,
titlebars, capsules, sticky sub-bars) and `glass-thick` (32px and a denser
tint; menus, popovers, the palette, dialogs, sheets,
toasts). Each takes a soft shadow (`--glass-shadow`: `shadow-raised` by
default, `shadow-float` on capsules and menus, `shadow-lift` on dialogs and
sheets).

**Refraction** (`lens`, ui-svelte) is a progressive enhancement for the few
large bars (the portal header bar): in Chromium the backdrop bends at
the rounded edge through an SVG displacement map computed for the element's
size and radius (a convex bezel; the centre stays true), with the lighter
`material-blur-lens` so the bend shows. Other engines, reduced transparency
and increased contrast keep the material above unchanged. Never on menus,
dialogs or content.

**Controls on glass are flat fills**, never a second glass (no nested
backdrop blur) and never a second border: one hairline on screen, the bar's. Inside `glass`
and `glass-thin` the tokens' `materialScope` remaps `fill`, `fill-strong` and
`thumb` to the clear control tints (`material-control-hover`,
`material-control`) and `shadow-rim` to `shadow-lens` (none), so every hover,
current and selected item on a bar is a flat tint without per-page colours or
rings. A
selection inside a bar is concentric: the same inset on every side, so its
radius is the bar's minus the inset. Standalone controls on a page (chips,
`secondary` buttons, the segmented thumb on a plain track) take at most
`shadow-rim`, a hairline. The `primary` is the flat neutral fill. Glass over art lives in a `data-theme="dark"` scope so it stays
dark in both themes. The **scrim** dims and blurs (`scrim`, `--scrim-blur`).
Where content passes under floating chrome it fades instead of cutting
(`scroll-edge-b`, `scroll-edge-t`, `scroll-edge-y`, `scroll-edge-x` for
shelves). Under `prefers-reduced-transparency` glass is the solid
`surface-raised`, the scrim does not blur and the control tints fall back to the
surrounding fills; under `prefers-contrast: more` glass is solid with a
`border-strong` edge.

**Art** (E1, E6): `ArtTile` is the art-filled card (cover or token gradient,
title over the scrim, status capsule, rank numeral in display type with award
metals for 1–3, glass play disc); `artGlow` reads a cover's dominant colour
from a 16×16 canvas on the client, clamps it to a lightness band and writes
`--art-glow` (cross-origin covers need CORS, otherwise no glow).

**In the engine** the three weights are og-render's `UiGlassStyle` presets
(`thin`, `regular`, `thick`; elevations `raised`, `float`, `lift`) with the
tints of `material-*`; the dark `regular` tint is 64 % (the platform's 52
%), the least that holds `fg` at 4.5:1 on a bar over a bright sky. The
default style puts `thick` on dialogs, menus, popovers, sheets, tooltips and
toasts, `regular` on HUD capsules and bars, and nothing on content: panels
inside a screen, cards, rows and fields are flat surfaces and fills.
Controls have no border: a button, field, chip or slot is a fill, told
apart from its neighbours by tone; the focus ring is the only outline.

## Motion

Motion is continuity, from the token durations (120/180/240/400ms, spring 520,
bounce 640), easings (`out`, `exit`, `spring`, `snappy`, `bounce`) and springs
(`motion.spring`: damping ratio and response, Apple's parameters — `default`
1.0/0.35 for overlays and highlights, `sheet` 0.85/0.3, `momentum` 0.8/0.4 for
releases that carried velocity; CSS gets `--ease-spring-*`/`--dur-spring-*`,
gestures the runtime spring in `ui-svelte` `motion/spring.ts`, Swift
`OriginTokens.Spring`):

- **Interruptible**: overlays move with transitions from `@starting-style` and
  leave along the same path, so a close mid-open turns around from the value on
  screen; nothing locks input while it moves.
- **Anchored**: menus and popovers scale from their trigger; a dialog grows
  from the press or focused control that opened it (from its centre when the
  URL opened it).
- **Direct**: phone sheets follow the finger 1:1 (grip, header, content at
  its top), resist past their bounds, project the release (0.998) to decide
  dismissal and hand the velocity to the spring. Sheets open only on the
  person's intent, one at a time; approvals take presses only once settled.
- **Answer**: every press springs from pointer-down (`data-pressed`, so touch
  answers too; no tap highlight, no double-tap delay); segmented and
  navigation highlights glide on the default spring; views that replace each
  other (dock views, phone destinations) slide in from their side
  (`ViewSwitch`); content fades where it passes under floating chrome
  (`scroll-edge-b`).
- **Arrive**: routes rise in; lists stagger (at most 8 items); numbers count up
  the first time they are seen; charts draw in; streaming text grows in place.
- **State**: pulse for needs-you, breathe for running.
- **Brand**: the Origin AI symbol has its own monochrome motion set
  (`OriginMark motion`, `OriginSpinner`, `OriginLogo motion="unfold"`; source
  `packages/brand/motion`), one motion per meaning (`BRAND_MOTION`):
  `OriginLogo motion="unfold"` on sign-in and the first screen;
  `OriginLoading` (orbit) while a page or panel waits for its first data;
  `OriginSpinner` (spin) inside buttons, rows and cells — ActionDialog's
  pending submit uses it; `BrandMark state="running"` (breathe) for a
  background job; `state="needsYou"` (beacon) for a queue waiting on the
  person; `CompletionBurst` / `useCompletionCelebration` (complete plus a
  small burst) when a batch, publish or payout really finished;
  `EmptyState mark` (sweep) for an empty list with its next step.
- **Arrive helpers**: `RouteRise` (keyed by route) and `Stagger`
  (`useStagger`, at most 8 steps) wrap the route-rise and list-stagger motion.
- **Celebrate** (bursts, confetti, rings, holo) only when something happened.
- **The timing table** (`motion.timing`) names what each moment moves on, so
  a game's style retunes one row instead of a widget:

  | Moment | Default | A style's override (`motion { … }`) |
  | --- | --- | --- |
  | Hover stickiness | lift on the `default` spring; a pad cursor's magnet 0.5 | `hoverSpring`, `magnet <0–1>` |
  | Press squash | in at once, back on damping 0.55 over `instant` (120 ms) | `pressSpring`, a dressing's `press` |
  | Enter / exit | `base` on `out` / `fast` on `exit` | `ease widgetEnter`, `ease widgetExit`, `duration base`/`fast` |
  | Staggered cascade | 40 ms a step, never past 400 ms | `stagger`, `staggerCap` |
  | Toast | in over `base`, held 3200 ms (an error 6000), out over `fast` | `toast <kind>`, `toastHold <ms> [<error ms>]` |
  | Screen transition | cover then reveal over twice `slow` | `screen <ms>` |

  The engine's widgets read these rows (og-ui's `theme::TIMING`, generated
  from `tokens.json`), and the defaults are the numbers the widgets drew
  with before the table, so no frame moved when it arrived.
- `prefers-reduced-motion` keeps state changes and drops spatial movement:
  colour and opacity still transition (overlays and views cross-fade), every
  rise, scale, slide, glide and drag spring jumps. Motion never delays a
  person's answer.

## Components

The components are the engine's og-ui widgets and the ui-kit components,
and origin-agent's web components; each draws the grammar below from the
tokens and never hard-codes a colour, size or curve. The inventory: primitives
(Button, IconButton, Field, Select, Checkbox, Switch, Menu, Dialog, Sheet,
Popover, Tooltip, Tabs, SegmentedControl, Toast, Badge, Avatar, Kbd), data
(tables, lists, pagination, metric cards, charts, timelines, empty and async
states), status (StatusDot, StatusRing, StatusCapsule), content (Card,
ArtTile, SectionHeader, Markdown) and motion (press, glide, count-up,
draw-in, stagger, burst).

The editorial grammar of the shared components:

- **Button**: a rounded rectangle (`rounded-button`); `primary` the flat neutral fill (one
  per surface; disabled turns neutral), `secondary` a neutral fill, `quiet` text until hovered, `glass`
  over art and media, `inverse` neutral emphasis (approve, sign in), `danger`.
  `IconButton` has the same variants; `glass` is the play disc over art.
- **Field / TextArea / Select / SearchField** (`.field`): a soft neutral fill
  without a border; focus lifts it to the surface with one soft ring;
  invalid is a danger edge.
- **Chip** (`FilterChip`): a neutral fill on a pill (`rounded-capsule`); selected
  inverts to `fg`.
  **Segmented** (`SegmentedControl`, `Tabs`): a neutral track with the `thumb`
  gliding; `glass` puts the track on thin glass over art.
- **Card**: a flat neutral `surface` with a hairline, `rounded-card`, 20px
  padding. **ArtTile**: art-filled (see Art above). **SectionHeader**: eyebrow
  and big title (`size` display, title, small); `tip` moves the description
  behind an `InfoTip` next to the title, as `Field`'s `tip` does for its hint.
- **InfoTip**: a 22px `HelpCircle` button in `fg-subtle` beside a label or
  title; an optional title and one to three short paragraphs.
- **InsetList / ListRow**: the grouped inset list (one `surface`, hairlines
  that start after the leading glyph): neutral glyph, label, optional meta,
  trailing value and a chevron for links and buttons; the current row sits on
  `fill-strong`.
- **Medal**: a disc in one award metal (`--metal-*`: gold, silver, bronze)
  with a dark ink glyph and its name under it; `locked` is a hollow on `fill`.
- **PixelAvatar**: Identity's avatar projection — a sprite on a pastel of its
  token hue (pixelated from 56px), an upload cover-fit, else the initial —
  always round, in sizes 24/32/40/56/88/120, with a clock badge for an
  owner's upload in review.
- **Dialog, Sheet, Menu, Popover, CommandPalette, Toast**: `glass-thick` over
  the blurred scrim; menus are concentric (12px outer, 8px items inside a 4px
  inset); highlighted and selected rows sit on `fill-strong`.
- **TabBar** (phone) and **TopBar**: `glass` bars; the current tab or
  item is a flat `fill-strong` (concentric) with an `fg` glyph. The
  portal header is one glass bar across the page (logo, primary entries,
  then the tools: search, the language switch — a globe opening each
  language in its own name, the current one checked — notifications and the
  account). Second-level navigation is `NavMenu` (Bits UI NavigationMenu):
  a group's glass panel opens when the pointer rests on it (120ms intent,
  then no delay while moving between groups), on a tap or from the keyboard,
  stays open on the way into the panel, and closes on leaving, a press
  outside, Escape or choosing; the open trigger shows its lens and its
  chevron turns. One `glass-thick` panel (the navigation viewport) is
  portaled out of the bar — never a glass nested in glass — stands under the
  open trigger, keeps the bar's theme scope, morphs to each group's size, and
  Tab leaves it for the next entry.
  **CapsuleNav**: the glass capsule with a gliding lens and `fg` ink.
- **StatusCapsule**: dot and word on `fill` (or thin glass over art).
- **EmptyState**: unboxed (like Apple's unavailable-content view); `boxed` for a
  slot in a dense layout. **Skeleton**: the neutral fill with a sheen that stops
  under reduced motion.
- **Markdown**: the reading style — 15px text in `fg`, headings on the weight
  ladder, links in `accent-ink` underlined, code on the stage plate, tables as a quiet fill
  block with regular-weight meta headers, tabular figures and hairline rows.

**One selection grammar**: a segmented control for 2–5 views of the same
content (`SegmentedControl`, or `Tabs` when panels need tab semantics — both
draw the same segmented thumb; `fill` gives equal segments), chips for
filters (`FilterChip`), the glass capsule for navigation (`CapsuleNav`, the
titlebar). No underline tabs on product surfaces.

Button variants: `primary`
(the neutral fill, one per surface), `secondary`, `quiet`, `glass`, `danger`, `inverse`.
A style or a page never overrides the shared variants of the default.

## Agent workspace: symbols, not sentences

The agent workspace (`/agent`, its Library, Session, docks and Stage page) is
used again and again; after the first visit an explanation is only noise.
So the page states things, it does not explain them:

- **No instructional copy on the page.** Words on screen are names (the
  Game, a work, a node), the person's and the agent's own words, numbers and
  the one verb of a primary action. A control's purpose is its glyph, with its
  name in a `Tooltip` and `aria-label`; a hint line under a field, a caption
  under a card or a sentence that says what a control does is a defect.
- **Teach once.** A first meeting with a control may carry a `Coach` mark: a
  small glass bubble beside it, gone for good after the person uses the
  control or closes it (per browser). Never more than one at a time on a
  screen; never a permanent caption. Deeper explanation lives behind an
  `InfoTip` or in the guide, not on the surface.
- **State is a glyph with its motion** (`StateGlyph`), one per meaning:

  | State | Glyph | Motion |
  |---|---|---|
  | preparing (environment, build) | Origin spinner | turns while it lasts |
  | working (the agent) | Origin mark | breathes while it lasts |
  | needs-you (a question, a choice) | question bubble, warn | three beacons, then holds |
  | playable (a build is ready) | play | pops in once |
  | changed (this turn changed the game) | sparks | twinkles once |
  | done | check in a circle | draws itself once |
  | failed | warning circle, danger | one short shake |
  | queued / paused | clock / pause | still, subtle |
  | offline / credit | cloud ✕ / coins | still; the way out sits beside it |

  A row or card leads with its glyph, a name and at most one line of facts;
  the action is one control at its end. Elapsed time and counts are numbers,
  not sentences.
- **Every change of state moves.** Arrivals rise (`.arrive`), replaced glyphs
  spring (`.swap-in`), lists stagger, rows reveal their action arrow on hover
  or focus, presses spring. Motion follows §Motion and stops under reduced
  motion; the glyph and colour still change.
- **Failures** are a glyph, the reason in a few words and the way out as the
  action (`failures.ts`: topup, continue, retry); raw errors sit behind a
  details disclosure.

## Enforcement

- `packages/design/tests/tokens.test.ts` holds every `contrast`, `nonText`
  and `glassContrast` pair in both themes and the `linear` fills against
  their CSS values.
- `crates/og-ui/tests/theme.rs` fails when `src/theme.rs` is not exactly
  what the generator writes from `tokens.json` (`crates/og-ui/scripts/gen-theme.sh`
  regenerates it), and when the tokens' material tints differ from
  og-render's glass presets.
- og-tests `ui_glass_text_holds_contrast` measures text over the three
  weights in rendered frames.

## Changes from the seed

From origingame d25f1f15 (2026-11-02):

- `fg-muted` #5f5f66 → #46464c (light) and #aeaeb2 → #c8c8cd (dark): it held
  3.0–3.8 on a selected row and 3.85–4.29 on a thick menu over a scene of
  the other scheme (docs/ui.md §3.3).
- `fg-subtle` (dark) #98989d → #9e9ea3: a field's placeholder on `fill`.
- `accent-ink` (light) #8f5600 → #7a4800: "Undo" on a light toast over warm
  light read 4.26.
- `material-regular` (dark) 52 % → 64 %.
- Added: the game roles, `density.pad`, `linear`, `glassContrast`.
- Left out: the platform's portal, console and page-template rules, which
  stay the platform's.
