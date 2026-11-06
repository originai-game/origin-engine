# UI and the UI packs

Screens and HUDs for an Origin Engine game. The UI packs (genre kits and style packs) and how to install and change them, documents versus code, the thresholds for text, contrast and touch, og ui check and preview, and languages with their fonts.

The API: `og api show ui` (elements, components, layers, motion, styles)
and `og api show i18n` (string tables, plurals, fonts, line breaking,
right to left). `og ui list` names every installed and bundled kit and
style with its documents.

## 1. Order of work

1. List the screens the game needs from the brief: title, the HUD, pause,
   settings, results, and the genre's own (inventory, shop, deck).
2. For each, start from what exists: a genre kit's component or screen
   (§2), else a library screen (`og ui eject --list`), else og.ui's
   generic components. Write a screen from primitives only when none fits.
3. Fill it with the game's data through its viewmodel; never edit text
   into a document that the game should bind.
4. Dress it with the look's style (§3), then preview and check (§5)
   at every breakpoint the create block's `platform` implies.

## 2. Genre kits

A kit is a directory with an `extension.json`, TypeScript components
registered with `registerUiExtension` and screen documents. What each
bundled kit draws, and how to use, change, review and make one: kits §1
to kits §5; the shader finishes of the `materials` kit: ui-materials §1.

- `og ui add <kit>` installs an editable copy under `assets/ui-kits/<kit>`;
  the game imports the installed `index.ts`. Edit the copy freely; an
  installation never replaces an existing one silently.
- `og ui eject <kit>/<screen>` copies one screen document with its
  companions into `assets/ui/`, where the game owns it.
- A kit never adds members to `og.ui`; its components are the namespace
  the kit exports. State a component keeps lives in the world's UI
  memory, not module variables (determinism §1).
- New components of the game's own register with `registerUiExtension`
  the same way, so they are dressed by styles like the kit's.

## 3. Style packs

A style pack (`assets/styles/<name>/style.ogt`, or a bundled one named by
slug) sets tokens (colours, type, radii, spacing, motion), part dressings,
sounds and haptics for every component. The look's `ui "<slug>"` picks it;
`og style list` names the bundled ones.

- Change a style by ejecting it (`og style eject <slug>`) and editing the
  copy, or by `og style add`. Never restyle components one by one in code:
  a style change then misses them.
- `og style new` makes a game its own style from the look and a few
  choices, then fills its art slots through the art brief (art-review §2).
- Hand-painted frames go through `og style pack <dir>` (nine-patches named
  `<part>.9.png`), which writes the atlas and slice lines.
- In a pack, `style.ogt` is the data, in the CSS dialect (docs/ui.md
  §12.7): palette custom properties set colour (`--primary: #d4993a`),
  `typography` sets text roles, selector blocks paint named parts and
  states (`Button::base.primary:hover { -og-fill: var(--primary-hover) }`). Edit it directly and run `og cook`; never regenerate a pack.
  `choices.json` only seeds `og style new` and overwrites nothing.
- Fonts, atlases, import settings and their licences stay with the pack;
  asset paths are relative to the style unless they share `@styles/`
  assets. `og style add ./path` installs a local pack.
- A results screen is a document, not style: eject it
  (`og ui eject --list`), edit it under `assets/ui/` and keep game data in
  its viewmodel.

## 4. Documents and code

- Static structure (layout, parts, slots, states, motion) belongs in
  `type screen` and `type component` documents: Tune edits them live and
  saves back, `og dev` reloads them in place keeping UI state.
- Code builds what depends on data at run time (a list of the player's
  items) and binds the rest. A screen written wholly in code loses live
  editing and the checks' element paths.
- Every word is a string key with an English fallback; no literal player
  text in code (§6).

## 5. Thresholds, preview and check

`og ui check` lays out every screen per style, breakpoint and locale and
reports findings and measures; it never passes or fails a screen. The
bars it does not hold, this guide does:

- **Text size**: at least 18 px at 1080p for menus, body and controls. A
  dense HUD readout may go to 12 px if its group sets
  `ElementProps.dense` and it still grows with the player's text scale.
- **Text scale**: every screen usable at 150 % text, nothing cut or
  overlapping.
- **Contrast**: live text at least 4.5:1 against what it is drawn on,
  3:1 for text over 24 px and for icons and control outlines; 7:1 in the
  high-contrast mode. No meaning by colour alone.
- **Touch**: a control's shorter side at least 44 CSS px on a coarse
  pointer, and 8 px between neighbouring targets; primary actions in the
  bottom half on a phone held in portrait.
- **Pad**: every interactive element reachable from the default focus,
  Back on every layer, no focus trap.
- **Primary actions**: one filled primary button per layer.
- **Cost**: a dense menu (300 elements, 40 animating) within 0.5 ms of
  CPU a tick, a HUD within 1 ms; the element count `og ui check` reports
  is where to look first.

Every finding is fixed (an undeclared overlap, cut text, the safe area,
missing names, missing feedback, placeholder art). A declared overlap is
for elements that are meant to overlap, never to silence a finding.

`og ui preview` renders the contact sheet (a row per breakpoint, a column
per locale) and, with `--film`, a motion's filmstrip: read it before
saying a screen is done. Review UI frames over the style's plain ground,
not a busy scene; a HUD over the scene is judged by the scene's frames.
Read a phone layout at 390×844 and a desktop one at 1920×1080.

## 6. Languages and fonts

- Every create-block language needs its string table
  (`assets/strings/<lang>.ogt`) and a font chain that covers its script;
  the default fonts cover Latin only. CJK, Arabic and Hebrew fonts and
  their licences ship in the engine's locale resources: copy the ones the
  game needs into `assets/`, name them in the table's fonts line, and load
  the table with og.i18n. `og check` warns while one is missing.
- Load only the font chains of the languages in use; a CJK font is
  megabytes and delays the first frame.
- `og i18n extract --check` lists missing keys per table; run it before
  each build.
- Size boxes for the pseudo-locale (40 % longer) and check a CJK and a
  right-to-left locale with `og ui check --locales`; a box sized to its
  English text overflows in German and mirrors wrongly in Arabic.
- Numbers, plurals and dates go through og.i18n; never concatenate
  translated fragments.
