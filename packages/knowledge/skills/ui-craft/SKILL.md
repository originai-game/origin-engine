---
name: ui-craft
description: The design craft of a game's UI, beyond the API — hierarchy (one primary action per layer, what the eye reads first), the layout grammar of each genre family's screens and HUDs (where the health, the map, the score and the actions go, and why), motion personality (crisp, heavy, bouncy) chosen from the premise, the conventions players of each genre expect, and the loop that proves a UI on every surface and language (og ui preview, og ui check). Use when designing or reviewing a screen, a HUD or a flow in a project with a project.ogt; og-ui has the calls, og-taste the art direction.
---

A game's UI is part of its picture, not a form laid over it: players read
it in a glance while doing something else, on a phone held in one hand, in
a language its author does not speak. This skill is how to design it so
it survives that; `og-ui` is how to build it (`og.ui`), `og-taste` how it
should look (the style, colour, type). Every rule below that can be
measured is one of `og ui check`'s, so a design is proven, not judged by
eye (§7).

## Polish on the prebuilt engine

Change TypeScript, `.ogt` styles and documents, ui-material WGSL and assets
only. Iterate with `og dev` and Tune; UI polish never compiles Rust.
Keep missing engine capabilities in one list with the desired appearance
and a reproducing screen. Hand that list to an engine thread for a batch
implementation, then continue polish on its prebuilt engine.

## Mock it up before building: static, then motion

Commercial UI (DICE's GDC 2018 pipeline, Persona 5's menus, Hades') is
designed in two mockups before a line of the screen is built: a still,
then its motion. Do the same for every screen and HUD a game adds, and
keep both under `mockups/<screen>/` at the project's root (outside
`assets/`, so the cook never bundles them):

1. **Static mockup.** One picture of the screen in the game's style at
   desktop and phone portrait: what is primary, where every block sits,
   the painted frames, icons and ornaments it uses, its type steps. Make
   it from the art brief (`styleArtBriefs` and an `artBrief` of kind
   `menuBackground` or `keyArt` with the style's concept sheets as
   references), or by laying out the screen's document with fixtures and
   `og ui preview` before any game code feeds it. Write beside it
   `mockup.md`: the one primary action, the three levels of §1, which
   painted parts and material presets (`og ui add materials`) it needs,
   and what is still missing art.
2. **Motion mockup.** The same screen as a filmstrip of what moves and
   when, read against the style's timing table (the design language's
   `motion.timing`, docs/ui.md §7.7): the entrance cascade, the hover
   and press on the primary, the toast, the transition out.
   `og ui preview . --screen menu=show:menu --film "<commands>" --at
   0,0.05,0.1,0.2,0.4` writes it; name each still's moment in
   `mockup.md` ("0.1 s: second row half in"). Retune the style's
   motion rows here, not in widget code.
3. **Build** to the two mockups. visual-acceptance §2 checks the built
   screen against them: a built screen that drifted from its mockup is
   either fixed or the mockup is redrawn and the reason written down.

## 1. Hierarchy: what the eye reads first

- **One primary action per layer.** A screen (or a dialog over it) has one
  thing it is for: Play on a title, Buy on a shop item, Confirm on a
  purchase. That one is the `primary` button; everything else is
  `secondary` or `quiet`. Two primaries make the player stop and read;
  `og ui check` reports them (`primary-actions`).
- **Three levels, no more.** The thing being decided (a card, an item's
  art, the score) is largest; its facts (price, stats, the timer) are the
  body step; meta (captions, hints, a version string) is the smallest
  readable step. A fourth level is noise.
- **Group by what the player does, not by data type.** Health beside the
  abilities that spend it, not beside the score.
- **Numbers that change are the hero of a HUD.** Give them a fixed-width
  box and a count-up (og.ui.counter), so a change is seen and nothing
  around it moves.
- **Nothing important in the centre third during play.** The play surface
  is the hero (og-taste §2): HUD in the corners and along the edges, the
  centre free except for a crosshair, a prompt or a hit.
- **Contrast is hierarchy too.** The primary action has the strongest
  value contrast on the screen; disabled controls the weakest (but text
  still at 4.5:1 when it must be read).

## 2. Layout grammar per genre family

Where things go is a convention players have learned; breaking it costs
them a second every time they look. The families below cover the genres
of docs/ui.md Appendix A; a game mixing families takes the HUD of the one
it plays like.

| Family | HUD grammar | Screens players expect |
|---|---|---|
| Action and platformer (PLT, RUN, TPA) | health or pips top left, currency and score top right, the timer top centre; touch: move bottom left, jump and actions bottom right, pause top right | world or level map, death and retry (fast: one press), loadout |
| Shooter and battle royale (SHO, BR) | crosshair centre only; health and armour bottom left, ammo and weapon bottom right, minimap top right (or left), killfeed top right under it, compass top centre | loadout, scoreboard (hold), item shop, battle pass, lobby |
| MOBA and arena | ability bar bottom centre with cooldown sweeps, minimap bottom right, scores top centre, item shop on a key | draft and ban with timers, post-game graphs |
| RPG (JRPG, ARPG, CRPG) and MMO | party or unit frames top left, target frame top centre, hotbar or command menu bottom, buffs beside the frame they belong to, quest tracker right edge | inventory (grid or list), character and stats, skill tree (pan and zoom), quest log, dialogue with choices, save grid |
| Roguelike and card (RGL, CARD, AUTO) | hand fanned along the bottom, piles in the bottom corners, energy left of the hand, relics or traits along the top | run map, pick one of three, deck and collection, run summary |
| Strategy (RTS, 4X, TAC, TD) | resources along the top, minimap bottom left, selection and command card bottom (centre and right), notifications down the right edge | tech tree, ledgers (sortable tables), diplomacy, post-match graphs |
| Management (CITY, IDLE, LIFE) | money and rates top, build toolbar bottom, overlays toggled from a side rail | dashboards, budget graphs, catalogues with search, prestige |
| Puzzle and casual (M3, CAS) | goals and moves top (one row), boosters bottom; one thumb reaches everything that is pressed | saga map, level start (goal, boosters), win and fail (stars, retry, next) |
| Gacha and collection | currencies top right, menu tabs bottom or right rail, events top left | banners, pull results with rarity reveal, roster grid, mailbox, login calendar |
| Rhythm, racing and sports (RHY, RACE, SPT) | rhythm: judgement and combo near the notes, score top; racing: position and lap top left, speed bottom right, map left; sports: scorebug top left | song or track select with preview, garage, results breakdown |
| Fighting and party (FGT, PARTY) | twin bars across the top meeting at the timer, round pips under them, super meters bottom | character select with simultaneous cursors, versus, podium |
| Narrative and VN | the text box bottom with the name plate on its top edge; auto, skip, log and save on the box | backlog, save and load with thumbnails, gallery, route chart |
| Survival, horror and open world (SURV, HOR, OW) | hotbar bottom centre, vitals bottom left, compass top centre; horror: as little HUD as possible, diegetic where it can be | inventory and crafting, world map with fog and pins, item inspect |

On a phone in portrait the grammar folds: top corners for status, the
bottom third for thumbs, nothing in the top centre that a notch or the
player shell's Back covers (the safe area and the covered region are
checks). Each HUD element is placed for each breakpoint (a breakpoint
variant in og.ui.props, or an anchor per breakpoint), never shrunk from
desktop and hoped to fit: two top-centre and top-right groups that sit far
apart at 1280 px wide overlap at 390 px.

## 3. Space and type

- One spacing scale (the style's `space` tokens), one radius per role, one
  type ramp. A value outside them is a bug.
- Boxes fit their text. No box is sized to its English text: German is
  30 % longer, Japanese wraps anywhere and is taller, Arabic mirrors.
  Labels in a row either wrap or are cut with an ellipsis and a tooltip.
- Touch targets are at least 44 CSS px on a coarse pointer, with at least
  8 px between them; a small icon gets hit slop, not a smaller target.
- Minimum text: 18 px at 1080p on desktop (scaled with the surface), and
  never under 12 CSS px on a phone (docs/ui.md's accessibility bar, after the Xbox Accessibility Guidelines).

## 4. Motion personality

A style's motion tokens are its personality; choose them from the premise
as the palette is chosen (og-taste §1), then let every component use the
tokens, never a duration of its own.

| Personality | Springs and eases | Fits |
|---|---|---|
| Crisp | short (120–200 ms), no overshoot, ease-out enters, fast exits | shooters, strategy, sports, sci-fi menus |
| Heavy | longer (250–400 ms), weighty settle, slight overshoot on the hero only | fantasy RPG, horror, ornate gacha |
| Bouncy | spring with visible overshoot, elastic press, staggered enters | casual, puzzle, party, kids |
| Snappy | very short, a punch (scale 1.05 → 1) on change | rhythm, fighting, arcade |

Rules every personality keeps:

- Motion answers state: a press, a change, an arrival. Nothing loops that
  carries no meaning, and decorative loops stop under reduced motion.
- Exits are faster than entrances; input is never refused longer than the
  entrance takes.
- Under reduced motion every animation has its fallback (a fade, or
  nothing), not the same animation slower.
- Screen changes go through one transition kind per game
  (og.ui.screenTransition), chosen with the personality.

## 5. Feedback

- Every interactive element shows hover (pointer), focus (keys and pads)
  and press, and a styled game sounds them (the style's hover and press
  sounds; og-audio §3).
- A refused action answers too: a shake, an error sound, a disabled look
  that explains itself.
- A change the player caused is seen where they looked: a cost is spent
  from the price they pressed, not only from the balance in the corner.

## 6. Every player, every language

- Pads reach every control from the default focus, and back (focus
  memory per screen, Back closes the topmost layer).
- Every control has an accessible name in the player's language: an icon
  button's name is what it does ("Pause", not "pause-icon").
- No meaning by colour alone: a rarity has a shape or a word too.
- Every word comes from the string table (og-i18n §2): a text that is the
  same in Japanese as in English was never translated.

## 7. Prove it: preview, check, adjust

Before calling a UI done, look at it everywhere and let the checks read
it:

```sh
og ui preview . --screen menu=show:menu --screen hud=show:play   # contact sheet: breakpoints × locales
og ui check . --screen menu=show:menu --screen hud=show:play     # every rule, with paths and the prop to change
og ui preview . --screen hud=show:play --film "damage:30" --at 0,0.05,0.1,0.2,0.4   # a motion as a filmstrip
```

Read the sheet (`og-out/ui-preview/sheet-<screen>.png`, with
`index.json` naming each cell) as a designer would, then read
`og-out/ui-check/check.txt`: each finding names the element paths, what
was measured, the limit, and the prop to change. Fix the prop, run the
check again; a clean check on every cell is the bar. og-ui §8 has the
commands and the rules in full.
