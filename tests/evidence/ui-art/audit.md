# UI art audit (2026-11-02)

The UI art director's read of the kit, by the art bar (docs/ui.md §15.5),
not the defect bar (§15.4). Frames were drawn on a Mac satellite (metal,
macos-27): main at cb1d65bf (`og test templates/ui-kit --filter
tests/frames-desktop`, pixel-identical to `crates/og-ui/references/metal`),
U6c's films at 83f7f3a6 and 47a3bbc5 (`examples/ui-components`,
film-desktop and film-phone), and U6d's patterns at 2a7ec0bf and f108ee2f
(`--filter tests/patterns`). Concepts: `concepts/<direction>-<screen>.jpg`
(A dark fantasy ARPG, B clean sci-fi, C bright stylised anime), each made
with the current frame as its content reference.

Each item: what, where, why it reads as amateur, and what a shipped game
does instead. Ranked by how much each costs the frame.

## Across every screen (fix once, everywhere)

1. **Line-icon placeholders where art belongs.** Shop items, inventory
   cells, ability slots, buffs, the gallery and the toasts use the 1.5 px
   iconoir outlines at 16–24 px in grey. Shipped games put painted,
   lit, saturated art in every item, ability and portrait slot (Diablo IV,
   Genshin), and keep line glyphs for system chrome (settings, close,
   back). Rule §15.5.3. Fixed by the kit art set (§15.5, item 4 of this
   wave).
2. **Flat grey boxes as the only material.** Panels are `#262626`-ish
   rectangles with no edge light, no shadow, no texture, no depth step
   between panel, card and control; the same grey is the panel, the
   button and the empty cell. Commercial UI separates three to four
   elevations with value and a lit edge (Destiny's glass with hairline,
   Diablo's iron frame, Genshin's cream card with gold trim).
3. **No dominant element.** Main menu, pause, results and settings are a
   small centred card on an empty backdrop; title, buttons and body share
   one weight. A shipped title screen gives 60 % of the frame to key art
   and one element (the logo) the most contrast; the menu is text on the
   art, not a box.
4. **Primary actions don't read as primary.** "Play", "Buy", "Retry",
   "Done", "End turn" are flat light-grey rectangles of the same size as
   the secondaries, so selection and primary look identical (white fill).
   One primary per layer with the style's accent, material and a key
   glyph; secondaries quiet.
5. **Selection and focus are a white outline or a white fill.** Shop's
   Iron sword, the ability bar's D, the players' Mage and Rogue, the
   tutorial's Play. Shipped games use a glow plus a lift, in the style's
   accent (gold in A, cyan in B), never pure white over grey.
6. **Type has no hierarchy and is undersized for desktop.** Body 12–13 px
   at 1280×720 frames (≈ 17 px at 1080p) for titles, labels and values
   alike; titles in the same weight as body. Commercial menus run a
   display face for titles (30–60 px at 1080p), small caps or tracked
   caps for section labels, 18–22 px body, tabular numerals for values.
7. **The backdrop is a placeholder.** Every kit frame sits on the same
   flat sunset with a pixel sun; the "Gallery" chip sits on every screen.
   A kit example shows the UI over real key art or a game scene.
8. **The default quiet-glass style is all the kit shows.** Its frames
   reached the owner as "the UI". The style is for origin-agent and
   prototypes (§12.1); the kit's showcase frames should wear the seed
   styles at their best, with the default as one column.

## HUD (U6c: abilities, meters, frames, shooter, strategy, racing, world)

Ability bar (at 47a3bbc5, already redone with painted art; sent to U6c):

1. Ultimate: a hard gold line across the art reads as a glitch; it
   should be a larger slot with a gold charge ring (concepts A, B).
2. Slots have no material (1 px black edge on flat art); need the style's
   frame: radius, top-lit rim, inner shadow, drop shadow.
3. Cooldown sweep too weak over bright art; numerals need an outline.
4. Hotkey letters carry state colour (red W, blue D); state belongs on
   the slot. Labels float 20 px under the slot.
5. Cost numeral red on busy art without a scrim.
6. Locked art stays saturated.

At 83f7f3a6 (the owner's example, phone 200 %): thin grey line icons on
black squares, boxed hotkey caps on the art, two stacked numerals per
slot, a white selection outline, a translucent grey box behind
everything. All fixed by 47a3bbc5 except as above.

Other HUD rows (next U6c round):

7. Cast bars: grey translucent track, line icons in grey squares,
   label and value one weight. Target: a thin framed bar with a lit fill
   and an end spark, the spell's painted icon, label in small caps.
8. Buff and debuff grid: line glyphs with a 2 px green or red box and
   stacks in a different corner per icon; timers below in three sizes.
   Target: painted icons with a thin rarity rim, a radial dim for time
   left, one timer under each.
9. Meters: health and mana as flat saturated bars with label text
   "HP"/"MP" left of them; orbs as flat spheres with a hard highlight.
   Target: the health orb in a cradle (A) or segmented shield over health
   (B) with the numeral in the meter.
10. Unit and party frames: initials in coloured discs instead of
    portraits, raid grid in eight pastel fills with names under the fill.
11. Minimap: a black disc or square with three dots. Target: a painted or
    rendered map in a framed ring with a compass and pings.
12. World labels: nameplates and loot labels overlap and run their text
    on the same flat fill; damage numbers have no crit scale or outline.
13. The test scene: grey slab and pastel boxes on a sage floor make every
    HUD frame look unfinished; the films need a lit scene (or key art)
    behind them.

## Genre patterns (U6d: cards, inventory, rewards, gacha)

1. Placeholder art everywhere: star glyphs as card art, grey busts as
   enemies and banners, a giant brown silhouette as gacha key art, slash
   glyphs as items. The kit art set replaces them.
2. Gacha: the featured characters are saturated full-width colour bars
   (a list widget); Wish buttons flat with a white dot as the price
   glyph; the reveal is a flat orange glow rectangle.
3. Cards: body text truncated with an ellipsis on fanned cards; the cost
   is a grey ringed circle, not a gem; "End turn" a flat grey rectangle.
4. Inventory: empty paperdoll slots show letters (N, H, R, O, F); the
   mannequin is a grey gradient; selection floods cyan over the item
   art; the attribute deltas disappeared at f108ee2f.
5. Rewards: the boon cards are good structurally; their icons are line
   glyphs in a tinted square and the three rarities differ only in
   border hue.

## Menus and screens (main's kit frames)

1. Main menu: a 260 px card centred on the backdrop, four identical
   full-width bars, "Continue" disabled at 40 % grey on grey. Target
   concepts A–C: key art full-bleed, logo dominant, text menu without
   boxes, one focused item with an accent mark.
2. Pause: the same card with three bars; no blur or dim of the game
   under it beyond the scrim.
3. Settings: a 420 px panel with tabs as a grey segmented strip; sliders
   1 px white lines with white knobs; huge empty area under four rows;
   "Done" floats alone bottom right. Target: tabs as a side list or
   bumpered row, a description pane, focused-row highlight, key hints.
4. Shop: three 70 px cards with line icons, a detail pane of grey text,
   the Buy bar full width in white. Target: vendor portrait, painted
   item cards, a detail card with the item large and the primary Buy.
5. Inventory (kit): 6×4 grey cells, line icons, a paragraph of help text
   in the corner.
6. Results: three outline stars, score in body weight +4, three equal
   grey buttons. Target: stars as filled badges landing in turn, the
   score the dominant numeral, one primary.
7. Loading: a white 2 px bar on black. Target: key art or a tip
   illustration, a styled progress and the tip in body type.
8. Dialogue (cutscene): a grey avatar glyph, a 14 px line, the name in
   cyan 10 px. Target: a portrait or sprite, the name in display caps,
   the line at 22–26 px, choices with checks.
9. Levels: tiles are grey boxes with text and outline stars; no level
   thumbnails. Target: thumbnails (the `levelThumb` slot).
10. Leaderboard and toasts: rows of identical grey bars, no rank
    medals, no avatars; toasts with no icon or accent.
11. Phone: the same cards scaled, so every gap above and below is empty;
    the "Gallery" chip is the most contrasted element on several sheets.

## Components and controls (U6a, U6b)

1. Buttons: variants differ by grey value only; the outlined variant is
   the one with an edge. Destructive red text on a dark-red fill reads
   muddy.
2. Cards (containers): grey gradient wash with a line icon as "art".
3. Carousel: a grey gradient with a star glyph.
4. Charts are the strongest frames; their area fills and gridlines would
   ship with a calmer gridline value.
5. Radial wheel: flat grey sectors with 1 px separators; target a ring
   with a lit focused sector and the item's art in the centre.

## Seed styles

1. Fantasy, gilded, storybook and craft read as the same parchment card.
2. The styles change panel paint only: no style changes layout,
   type scale or icon art, so 19 styles read as one UI recoloured.
3. Kawaii's buttons are pink pills on white with a white disabled state:
   no visible edge on the secondaries (already in U8's list).
