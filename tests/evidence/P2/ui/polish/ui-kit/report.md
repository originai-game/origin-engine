# og polish: ui-kit

2026-10-19, 29 playtest(s) on metal. Mechanical measures of og-taste and og-feel (og-build §5); a score is 0 to 100, n/a when nothing in the playtests exercises it. The measures cannot judge style: read the frames.

## Scores

| | Score | Measure |
|---|---|---|
| **Taste** | **91** | mean of the taste rows |
| Contrast | 84 | text boxes whose ink and ground reach 4.5:1 |
| Text size | 98 | text boxes whose tallest ink line reaches 9 CSS px (a 13 px face's capitals) |
| HUD occupancy | 100 | gameplay frames: HUD at most 15 % of the surface and 5 % of the central third |
| Screen transitions | 96 | screen changes with UI motion around them |
| UI sounds | 65 | UI presses answered on the ui bus |
| Locale keys | 100 | string tables without missing keys (og cook) |
| **Feel** | **70** | mean of the feel rows |
| Input latency | 100 | inputs answered, by the median ticks to the answer (0–1: 100) |
| Hit feedback | 100 | damage moments with hit stop, shake, floating text or a sound (three of four: 100) |
| Idle frames | 75 | ticks where nothing new is seen or heard (10 %: 100, 60 %: 0) |
| Frame time | 0 | p95 step + render (8.3 ms: 100, 33 ms: 0), worst playtest |
| Sound on input | 73 | inputs answered by a sound within 6 ticks |

## Fixes

| Measure | Where | Found | Fix |
|---|---|---|---|
| screen transitions | a11y | 1 of 24 screen changes had no motion in the 20 ticks around them | og-taste §5 (screens change through a transition): og.ui.screenTransition, one kind per game |
| idle frames | a11y | 34.4 % of ticks show and play nothing new (nothing moves or animates, no UI motion, no sound, no tween, text or particle) | og-feel §9 (idle loops: breathing, a pickup's spin) and og-taste §5 (motion): something alive on every screen |
| frame time | drag | p95 step + render 40.59 ms on metal (the frame is 16.7 ms) | og-game §14 (performance): fewer draws and particles, cheaper systems; og run --json measures |
| contrast | frames-desktop confirm: `Confirm` "Delete" | 2.91:1 between ink and ground (body text wants 4.5:1) | og-taste §3 (value before hue): darken the ground or lighten the ink in the skin's palette, never per call |
| contrast | frames-desktop mainMenu: `play` "Play" | 3.16:1 between ink and ground (body text wants 4.5:1) | og-taste §3 (value before hue): darken the ground or lighten the ink in the skin's palette, never per call |
| contrast | frames-desktop tutorial: `play` "Play" | 3.16:1 between ink and ground (body text wants 4.5:1) | og-taste §3 (value before hue): darken the ground or lighten the ink in the skin's palette, never per call |
| idle frames | frames-desktop | 27.18 % of ticks show and play nothing new (nothing moves or animates, no UI motion, no sound, no tween, text or particle) | og-feel §9 (idle loops: breathing, a pickup's spin) and og-taste §5 (motion): something alive on every screen |
| frame time | frames-desktop | p95 step + render 48.83 ms on metal (the frame is 16.7 ms) | og-game §14 (performance): fewer draws and particles, cheaper systems; og run --json measures |
| contrast | frames-phone widgets: `Gallery` "Gallery" | 1.46:1 between ink and ground (body text wants 4.5:1) | og-taste §3 (value before hue): darken the ground or lighten the ink in the skin's palette, never per call |
| contrast | frames-phone confirm: `Confirm` "Delete" | 2.96:1 between ink and ground (body text wants 4.5:1) | og-taste §3 (value before hue): darken the ground or lighten the ink in the skin's palette, never per call |
| contrast | frames-phone shop: `Price` "2,400" | 3.17:1 between ink and ground (body text wants 4.5:1) | og-taste §3 (value before hue): darken the ground or lighten the ink in the skin's palette, never per call |
| screen transitions | frames-phone | 2 of 24 screen changes had no motion in the 20 ticks around them | og-taste §5 (screens change through a transition): og.ui.screenTransition, one kind per game |
| idle frames | frames-phone | 31.14 % of ticks show and play nothing new (nothing moves or animates, no UI motion, no sound, no tween, text or particle) | og-feel §9 (idle loops: breathing, a pickup's spin) and og-taste §5 (motion): something alive on every screen |
| frame time | frames-phone | p95 step + render 21.39 ms on metal (the frame is 16.7 ms) | og-game §14 (performance): fewer draws and particles, cheaper systems; og run --json measures |
| input response | gamepad | 1 of 18 inputs changed nothing seen or heard within 30 ticks | og-feel §1 (answer on the same tick) and og-taste §6: every press shows an answer, a refused one too |
| UI sounds | hit | 1 of 1 UI presses played no sound on the ui bus | og-taste §6 (feedback) and og-audio §3: give the skin or og.ui.configure its press and open sounds |
| contrast | juice iris-opening: `Best` "Best 12,840" | 2.52:1 between ink and ground (body text wants 4.5:1) | og-taste §3 (value before hue): darken the ground or lighten the ink in the skin's palette, never per call |
| contrast | juice landed: `NewBest` "New best" | 4.17:1 between ink and ground (body text wants 4.5:1) | og-taste §3 (value before hue): darken the ground or lighten the ink in the skin's palette, never per call |
| contrast | juice count-up: `retry` "Retry" | 4.35:1 between ink and ground (body text wants 4.5:1) | og-taste §3 (value before hue): darken the ground or lighten the ink in the skin's palette, never per call |
| contrast | locales mainMenu-ar: `play` "العب" | 2.52:1 between ink and ground (body text wants 4.5:1) | og-taste §3 (value before hue): darken the ground or lighten the ink in the skin's palette, never per call |
| contrast | locales mainMenu-ja: `play` "プレイ" | 2.67:1 between ink and ground (body text wants 4.5:1) | og-taste §3 (value before hue): darken the ground or lighten the ink in the skin's palette, never per call |
| contrast | locales confirm-ja: `Confirm` "削除" | 2.75:1 between ink and ground (body text wants 4.5:1) | og-taste §3 (value before hue): darken the ground or lighten the ink in the skin's palette, never per call |
| screen transitions | locales | 4 of 15 screen changes had no motion in the 20 ticks around them | og-taste §5 (screens change through a transition): og.ui.screenTransition, one kind per game |
| idle frames | locales | 57.12 % of ticks show and play nothing new (nothing moves or animates, no UI motion, no sound, no tween, text or particle) | og-feel §9 (idle loops: breathing, a pickup's spin) and og-taste §5 (motion): something alive on every screen |
| UI sounds | multitouch | 2 of 2 UI presses played no sound on the ui bus | og-taste §6 (feedback) and og-audio §3: give the skin or og.ui.configure its press and open sounds |
| UI sounds | phone | 1 of 1 UI presses played no sound on the ui bus | og-taste §6 (feedback) and og-audio §3: give the skin or og.ui.configure its press and open sounds |
| idle frames | phone | 96.43 % of ticks show and play nothing new (nothing moves or animates, no UI motion, no sound, no tween, text or particle) | og-feel §9 (idle loops: breathing, a pickup's spin) and og-taste §5 (motion): something alive on every screen |
| contrast | skin-cartoon-phone widgets: `Gallery` "Gallery" | 1.13:1 between ink and ground (body text wants 4.5:1) | og-taste §3 (value before hue): darken the ground or lighten the ink in the skin's palette, never per call |
| contrast | skin-cartoon-phone racing: `Value` "234" | 1.16:1 between ink and ground (body text wants 4.5:1) | og-taste §3 (value before hue): darken the ground or lighten the ink in the skin's palette, never per call |
| contrast | skin-cartoon-phone shooter: `Weapon` "Carbine" | 1.75:1 between ink and ground (body text wants 4.5:1) | og-taste §3 (value before hue): darken the ground or lighten the ink in the skin's palette, never per call |
| text size | skin-cartoon-phone shop: `Unit` "coins" | ink lines 8.33 CSS px tall (a 13 px face's capitals and digits are about 9) | og-taste §4 and §10.6: use the ramp's body step or larger for anything read on a phone |
| text size | skin-cartoon-phone shop: `Name` "Iron sword" | ink lines 8.33 CSS px tall (a 13 px face's capitals and digits are about 9) | og-taste §4 and §10.6: use the ramp's body step or larger for anything read on a phone |
| text size | skin-cartoon-phone widgets: `New` "New" | ink lines 8.67 CSS px tall (a 13 px face's capitals and digits are about 9) | og-taste §4 and §10.6: use the ramp's body step or larger for anything read on a phone |
| screen transitions | skin-cartoon-phone | 2 of 24 screen changes had no motion in the 20 ticks around them | og-taste §5 (screens change through a transition): og.ui.screenTransition, one kind per game |
| idle frames | skin-cartoon-phone | 25.34 % of ticks show and play nothing new (nothing moves or animates, no UI motion, no sound, no tween, text or particle) | og-feel §9 (idle loops: breathing, a pickup's spin) and og-taste §5 (motion): something alive on every screen |
| contrast | skin-cartoon shooter: `Zone` "Sector 4" | 1.62:1 between ink and ground (body text wants 4.5:1) | og-taste §3 (value before hue): darken the ground or lighten the ink in the skin's palette, never per call |
| contrast | skin-cartoon loading: `Percent` "100%" | 1.67:1 between ink and ground (body text wants 4.5:1) | og-taste §3 (value before hue): darken the ground or lighten the ink in the skin's palette, never per call |
| contrast | skin-cartoon leaderboard: `Rank` "1" | 1.79:1 between ink and ground (body text wants 4.5:1) | og-taste §3 (value before hue): darken the ground or lighten the ink in the skin's palette, never per call |
| text size | skin-cartoon shooter: `Gallery` "Gallery" | ink lines 6 CSS px tall (a 13 px face's capitals and digits are about 9) | og-taste §4 and §10.6: use the ramp's body step or larger for anything read on a phone |
| contrast | skin-fantasy-phone cutscene: `Skip` "Skip" | 1.2:1 between ink and ground (body text wants 4.5:1) | og-taste §3 (value before hue): darken the ground or lighten the ink in the skin's palette, never per call |
| contrast | skin-fantasy-phone racing: `Value` "234" | 1.25:1 between ink and ground (body text wants 4.5:1) | og-taste §3 (value before hue): darken the ground or lighten the ink in the skin's palette, never per call |
| contrast | skin-fantasy-phone widgets: `Gallery` "Gallery" | 1.34:1 between ink and ground (body text wants 4.5:1) | og-taste §3 (value before hue): darken the ground or lighten the ink in the skin's palette, never per call |
| text size | skin-fantasy-phone shop: `Unit` "coins" | ink lines 7.67 CSS px tall (a 13 px face's capitals and digits are about 9) | og-taste §4 and §10.6: use the ramp's body step or larger for anything read on a phone |
| text size | skin-fantasy-phone shop: `Name` "Iron sword" | ink lines 7.67 CSS px tall (a 13 px face's capitals and digits are about 9) | og-taste §4 and §10.6: use the ramp's body step or larger for anything read on a phone |
| text size | skin-fantasy-phone gallery: `Name` "cutscene" | ink lines 8.33 CSS px tall (a 13 px face's capitals and digits are about 9) | og-taste §4 and §10.6: use the ramp's body step or larger for anything read on a phone |
| screen transitions | skin-fantasy-phone | 2 of 25 screen changes had no motion in the 20 ticks around them | og-taste §5 (screens change through a transition): og.ui.screenTransition, one kind per game |
| idle frames | skin-fantasy-phone | 26.21 % of ticks show and play nothing new (nothing moves or animates, no UI motion, no sound, no tween, text or particle) | og-feel §9 (idle loops: breathing, a pickup's spin) and og-taste §5 (motion): something alive on every screen |
| contrast | skin-fantasy cutscene: `Gallery` "Gallery" | 1.23:1 between ink and ground (body text wants 4.5:1) | og-taste §3 (value before hue): darken the ground or lighten the ink in the skin's palette, never per call |
| contrast | skin-fantasy shooter: `Zone` "Sector 4" | 1.36:1 between ink and ground (body text wants 4.5:1) | og-taste §3 (value before hue): darken the ground or lighten the ink in the skin's palette, never per call |
| contrast | skin-fantasy puzzle: `Caption` "moves left" | 1.65:1 between ink and ground (body text wants 4.5:1) | og-taste §3 (value before hue): darken the ground or lighten the ink in the skin's palette, never per call |
| text size | skin-fantasy shop: `Unit` "coins" | ink lines 7 CSS px tall (a 13 px face's capitals and digits are about 9) | og-taste §4 and §10.6: use the ramp's body step or larger for anything read on a phone |
| text size | skin-fantasy widgets: `New` "New" | ink lines 8 CSS px tall (a 13 px face's capitals and digits are about 9) | og-taste §4 and §10.6: use the ramp's body step or larger for anything read on a phone |
| text size | skin-fantasy shop: `Name` "Iron sword" | ink lines 8 CSS px tall (a 13 px face's capitals and digits are about 9) | og-taste §4 and §10.6: use the ramp's body step or larger for anything read on a phone |
| frame time | skin-fantasy | p95 step + render 21.74 ms on metal (the frame is 16.7 ms) | og-game §14 (performance): fewer draws and particles, cheaper systems; og run --json measures |
| contrast | skin-neon-phone shooter: `Armor` "35" | 2.05:1 between ink and ground (body text wants 4.5:1) | og-taste §3 (value before hue): darken the ground or lighten the ink in the skin's palette, never per call |
| contrast | skin-neon-phone shop: `Price` "300" | 2.22:1 between ink and ground (body text wants 4.5:1) | og-taste §3 (value before hue): darken the ground or lighten the ink in the skin's palette, never per call |
| contrast | skin-neon-phone shop: `Price` "220" | 2.25:1 between ink and ground (body text wants 4.5:1) | og-taste §3 (value before hue): darken the ground or lighten the ink in the skin's palette, never per call |
| text size | skin-neon-phone widgets: `New` "New" | ink lines 7.67 CSS px tall (a 13 px face's capitals and digits are about 9) | og-taste §4 and §10.6: use the ramp's body step or larger for anything read on a phone |
| text size | skin-neon-phone widgets: `Sale` "-20%" | ink lines 7.67 CSS px tall (a 13 px face's capitals and digits are about 9) | og-taste §4 and §10.6: use the ramp's body step or larger for anything read on a phone |
| text size | skin-neon-phone widgets: `Rare` "Rare" | ink lines 7.67 CSS px tall (a 13 px face's capitals and digits are about 9) | og-taste §4 and §10.6: use the ramp's body step or larger for anything read on a phone |
| screen transitions | skin-neon-phone | 2 of 25 screen changes had no motion in the 20 ticks around them | og-taste §5 (screens change through a transition): og.ui.screenTransition, one kind per game |
| idle frames | skin-neon-phone | 27.18 % of ticks show and play nothing new (nothing moves or animates, no UI motion, no sound, no tween, text or particle) | og-feel §9 (idle loops: breathing, a pickup's spin) and og-taste §5 (motion): something alive on every screen |
| contrast | skin-neon cutscene: `Gallery` "Gallery" | 1.65:1 between ink and ground (body text wants 4.5:1) | og-taste §3 (value before hue): darken the ground or lighten the ink in the skin's palette, never per call |
| contrast | skin-neon levels: `Name` "8" | 1.89:1 between ink and ground (body text wants 4.5:1) | og-taste §3 (value before hue): darken the ground or lighten the ink in the skin's palette, never per call |
| contrast | skin-neon confirm: `Confirm` "Delete" | 1.99:1 between ink and ground (body text wants 4.5:1) | og-taste §3 (value before hue): darken the ground or lighten the ink in the skin's palette, never per call |
| text size | skin-neon widgets: `New` "New" | ink lines 8 CSS px tall (a 13 px face's capitals and digits are about 9) | og-taste §4 and §10.6: use the ramp's body step or larger for anything read on a phone |
| text size | skin-neon widgets: `Sale` "-20%" | ink lines 8 CSS px tall (a 13 px face's capitals and digits are about 9) | og-taste §4 and §10.6: use the ramp's body step or larger for anything read on a phone |
| text size | skin-neon widgets: `Rare` "Rare" | ink lines 8 CSS px tall (a 13 px face's capitals and digits are about 9) | og-taste §4 and §10.6: use the ramp's body step or larger for anything read on a phone |
| contrast | skin-paper-phone widgets: `Gallery` "Gallery" | 1.12:1 between ink and ground (body text wants 4.5:1) | og-taste §3 (value before hue): darken the ground or lighten the ink in the skin's palette, never per call |
| contrast | skin-paper-phone shooter: `Ammo` "18" | 1.2:1 between ink and ground (body text wants 4.5:1) | og-taste §3 (value before hue): darken the ground or lighten the ink in the skin's palette, never per call |
| contrast | skin-paper-phone racing: `Value` "234" | 1.25:1 between ink and ground (body text wants 4.5:1) | og-taste §3 (value before hue): darken the ground or lighten the ink in the skin's palette, never per call |
| text size | skin-paper-phone inventory: `Count` "4" | ink lines 8.33 CSS px tall (a 13 px face's capitals and digits are about 9) | og-taste §4 and §10.6: use the ramp's body step or larger for anything read on a phone |
| text size | skin-paper-phone idle: `Level` "Lv 1" | ink lines 8.67 CSS px tall (a 13 px face's capitals and digits are about 9) | og-taste §4 and §10.6: use the ramp's body step or larger for anything read on a phone |
| text size | skin-paper-phone shop: `Unit` "coins" | ink lines 8.67 CSS px tall (a 13 px face's capitals and digits are about 9) | og-taste §4 and §10.6: use the ramp's body step or larger for anything read on a phone |
| screen transitions | skin-paper-phone | 2 of 25 screen changes had no motion in the 20 ticks around them | og-taste §5 (screens change through a transition): og.ui.screenTransition, one kind per game |
| idle frames | skin-paper-phone | 26.31 % of ticks show and play nothing new (nothing moves or animates, no UI motion, no sound, no tween, text or particle) | og-feel §9 (idle loops: breathing, a pickup's spin) and og-taste §5 (motion): something alive on every screen |
| contrast | skin-paper cutscene: `Gallery` "Gallery" | 1.47:1 between ink and ground (body text wants 4.5:1) | og-taste §3 (value before hue): darken the ground or lighten the ink in the skin's palette, never per call |
| contrast | skin-paper shooter: `Zone` "Sector 4" | 1.63:1 between ink and ground (body text wants 4.5:1) | og-taste §3 (value before hue): darken the ground or lighten the ink in the skin's palette, never per call |
| contrast | skin-paper idle: `Tap` "Tap" | 1.75:1 between ink and ground (body text wants 4.5:1) | og-taste §3 (value before hue): darken the ground or lighten the ink in the skin's palette, never per call |
| contrast | skin-pixel-phone widgets: `Health` "72 / 100" | 1.66:1 between ink and ground (body text wants 4.5:1) | og-taste §3 (value before hue): darken the ground or lighten the ink in the skin's palette, never per call |
| contrast | skin-pixel-phone racing: `Boost` "Boost" | 1.74:1 between ink and ground (body text wants 4.5:1) | og-taste §3 (value before hue): darken the ground or lighten the ink in the skin's palette, never per call |
| contrast | skin-pixel-phone widgets: `Gallery` "Gallery" | 2.17:1 between ink and ground (body text wants 4.5:1) | og-taste §3 (value before hue): darken the ground or lighten the ink in the skin's palette, never per call |
| screen transitions | skin-pixel-phone | 2 of 24 screen changes had no motion in the 20 ticks around them | og-taste §5 (screens change through a transition): og.ui.screenTransition, one kind per game |
| idle frames | skin-pixel-phone | 30.75 % of ticks show and play nothing new (nothing moves or animates, no UI motion, no sound, no tween, text or particle) | og-feel §9 (idle loops: breathing, a pickup's spin) and og-taste §5 (motion): something alive on every screen |
| contrast | skin-pixel shop: `Price` "2,400" | 2.05:1 between ink and ground (body text wants 4.5:1) | og-taste §3 (value before hue): darken the ground or lighten the ink in the skin's palette, never per call |
| contrast | skin-pixel inventory: `Count` "20" | 2.2:1 between ink and ground (body text wants 4.5:1) | og-taste §3 (value before hue): darken the ground or lighten the ink in the skin's palette, never per call |
| contrast | skin-pixel inventory: `Count` "2" | 2.21:1 between ink and ground (body text wants 4.5:1) | og-taste §3 (value before hue): darken the ground or lighten the ink in the skin's palette, never per call |
| idle frames | skin-pixel | 26.79 % of ticks show and play nothing new (nothing moves or animates, no UI motion, no sound, no tween, text or particle) | og-feel §9 (idle loops: breathing, a pickup's spin) and og-taste §5 (motion): something alive on every screen |
| contrast | skin-scifi-phone shooter: `Armor` "35" | 1.59:1 between ink and ground (body text wants 4.5:1) | og-taste §3 (value before hue): darken the ground or lighten the ink in the skin's palette, never per call |
| contrast | skin-scifi-phone widgets: `Health` "72 / 100" | 1.9:1 between ink and ground (body text wants 4.5:1) | og-taste §3 (value before hue): darken the ground or lighten the ink in the skin's palette, never per call |
| contrast | skin-scifi-phone shooter: `Health` "72" | 1.9:1 between ink and ground (body text wants 4.5:1) | og-taste §3 (value before hue): darken the ground or lighten the ink in the skin's palette, never per call |
| text size | skin-scifi-phone shop: `Unit` "coins" | ink lines 8 CSS px tall (a 13 px face's capitals and digits are about 9) | og-taste §4 and §10.6: use the ramp's body step or larger for anything read on a phone |
| text size | skin-scifi-phone shop: `Name` "Iron sword" | ink lines 8 CSS px tall (a 13 px face's capitals and digits are about 9) | og-taste §4 and §10.6: use the ramp's body step or larger for anything read on a phone |
| text size | skin-scifi-phone gallery: `Blurb` "Pause over a HUD" | ink lines 8.33 CSS px tall (a 13 px face's capitals and digits are about 9) | og-taste §4 and §10.6: use the ramp's body step or larger for anything read on a phone |
| screen transitions | skin-scifi-phone | 2 of 25 screen changes had no motion in the 20 ticks around them | og-taste §5 (screens change through a transition): og.ui.screenTransition, one kind per game |
| idle frames | skin-scifi-phone | 27.95 % of ticks show and play nothing new (nothing moves or animates, no UI motion, no sound, no tween, text or particle) | og-feel §9 (idle loops: breathing, a pickup's spin) and og-taste §5 (motion): something alive on every screen |
| contrast | skin-scifi cutscene: `Gallery` "Gallery" | 1.55:1 between ink and ground (body text wants 4.5:1) | og-taste §3 (value before hue): darken the ground or lighten the ink in the skin's palette, never per call |
| contrast | skin-scifi shooter: `Armor` "35" | 1.59:1 between ink and ground (body text wants 4.5:1) | og-taste §3 (value before hue): darken the ground or lighten the ink in the skin's palette, never per call |
| contrast | skin-scifi widgets: `Health` "72 / 100" | 1.9:1 between ink and ground (body text wants 4.5:1) | og-taste §3 (value before hue): darken the ground or lighten the ink in the skin's palette, never per call |
| text size | skin-scifi shop: `Name` "Iron sword" | ink lines 8 CSS px tall (a 13 px face's capitals and digits are about 9) | og-taste §4 and §10.6: use the ramp's body step or larger for anything read on a phone |
| UI sounds | sounds | 1 of 2 UI presses played no sound on the ui bus | og-taste §6 (feedback) and og-audio §3: give the skin or og.ui.configure its press and open sounds |
| UI sounds | touch | 3 of 9 UI presses played no sound on the ui bus | og-taste §6 (feedback) and og-audio §3: give the skin or og.ui.configure its press and open sounds |
| UI sounds | tutorial | 1 of 4 UI presses played no sound on the ui bus | og-taste §6 (feedback) and og-audio §3: give the skin or og.ui.configure its press and open sounds |

## Playtests

### a11y

every screen's accessibility tree names its controls, and Tab follows its focus order (1074 ticks)

| Measure | Value |
|---|---|
| Text boxes | 0; 0 at 4.5:1; 0 of 0 at 9 px ink |
| HUD | 0 gameplay frame(s), up to 0 % of the surface, 0 % of the centre |
| UI | 24 screen change(s), 23 with motion; 646 ticks of UI motion; 0 press(es), 0 with a ui-bus sound |
| Inputs | 6, 6 answered, median 0 ticks, max 0; 6 with a sound; 68.62 % of input-free ticks also change something |
| Damage | 0 moment(s): hit stop 0, shake 0, floating text 0, sound 0, tween 0, particles 0 |
| Idle | 34.4 % of ticks |
| Particles | peak 0 system(s), 0 burst(s) |
| Frame (metal) | step 1.38 ms median, 2.93 p95; render 0.91 median, 1.31 p95; frame p95 4.11, max 44.07 |
| Audio | 17.9 s, peak 0.16, RMS 0.0155, 0 clipped, 89.57 % silent ticks |

### drag

a drag from one inventory slot to another moves the item (108 ticks)

| Measure | Value |
|---|---|
| Text boxes | 0; 0 at 4.5:1; 0 of 0 at 9 px ink |
| HUD | 0 gameplay frame(s), up to 0 % of the surface, 0 % of the centre |
| UI | 3 screen change(s), 3 with motion; 107 ticks of UI motion; 2 press(es), 2 with a ui-bus sound |
| Inputs | 2, 2 answered, median 0 ticks, max 0; 2 with a sound; 100 % of input-free ticks also change something |
| Damage | 0 moment(s): hit stop 0, shake 0, floating text 0, sound 0, tween 0, particles 0 |
| Idle | 0 % of ticks |
| Particles | peak 0 system(s), 0 burst(s) |
| Frame (metal) | step 2.15 ms median, 12.92 p95; render 1.09 median, 21.15 p95; frame p95 40.59, max 124.33 |
| Audio | 1.8 s, peak 0.21, RMS 0.0243, 0 clipped, 77.78 % silent ticks |

### frames-desktop

every screen at desktop size matches its reference frame (1044 ticks)

| Measure | Value |
|---|---|
| Text boxes | 236; 206 at 4.5:1; 236 of 236 at 9 px ink |
| Lowest contrast | 2.91:1 `Confirm` (confirm); 3.16:1 `play` (mainMenu); 3.16:1 `play` (tutorial) |
| Smallest ink | 9 px `New` (widgets); 9 px `Sale` (widgets); 9 px `Rare` (widgets) |
| HUD | 5 gameplay frame(s), up to 4.24 % of the surface, 0.75 % of the centre |
| UI | 24 screen change(s), 24 with motion; 701 ticks of UI motion; 0 press(es), 0 with a ui-bus sound |
| Inputs | 0, 0 answered, median 0 ticks, max 0; 0 with a sound; 72.05 % of input-free ticks also change something |
| Damage | 0 moment(s): hit stop 0, shake 0, floating text 0, sound 0, tween 0, particles 0 |
| Idle | 27.18 % of ticks |
| Particles | peak 0 system(s), 0 burst(s) |
| Frame (metal) | step 2.04 ms median, 34.51 p95; render 1.21 median, 27.2 p95; frame p95 48.83, max 444.41 |
| Audio | 17.4 s, peak 0.16, RMS 0.0156, 0 clipped, 90.42 % silent ticks |

![frames-desktop gallery](frames/frames-desktop-gallery.png) 

gallery (tick 66, menu: HUD 8.52 %, 19 texts, 0 under 4.5:1, 0 small)

### frames-phone

every screen on a phone in portrait matches its reference frame (1044 ticks)

| Measure | Value |
|---|---|
| Text boxes | 230; 210 at 4.5:1; 230 of 230 at 9 px ink |
| Lowest contrast | 1.46:1 `Gallery` (widgets); 2.96:1 `Confirm` (confirm); 3.17:1 `Price` (shop) |
| Smallest ink | 9 px `Level` (idle); 9 px `Count` (inventory); 9 px `Count` (inventory) |
| HUD | 5 gameplay frame(s), up to 14.3 % of the surface, 1.43 % of the centre |
| UI | 24 screen change(s), 22 with motion; 661 ticks of UI motion; 0 press(es), 0 with a ui-bus sound |
| Inputs | 0, 0 answered, median 0 ticks, max 0; 0 with a sound; 68.09 % of input-free ticks also change something |
| Damage | 0 moment(s): hit stop 0, shake 0, floating text 0, sound 0, tween 0, particles 0 |
| Idle | 31.14 % of ticks |
| Particles | peak 0 system(s), 0 burst(s) |
| Frame (metal) | step 1.7 ms median, 6.21 p95; render 1.09 median, 3.14 p95; frame p95 21.39, max 988.92 |
| Audio | 17.4 s, peak 0.16, RMS 0.0156, 0 clipped, 90.42 % silent ticks |

![frames-phone gallery](frames/frames-phone-gallery.png) 

gallery (tick 66, menu: HUD 27.92 %, 23 texts, 0 under 4.5:1, 0 small)

### gamepad

a gamepad's d-pad, stick, A and B drive the gallery through og.Gamepad (168 ticks)

| Measure | Value |
|---|---|
| Text boxes | 0; 0 at 4.5:1; 0 of 0 at 9 px ink |
| HUD | 0 gameplay frame(s), up to 0 % of the surface, 0 % of the centre |
| UI | 3 screen change(s), 3 with motion; 133 ticks of UI motion; 0 press(es), 0 with a ui-bus sound |
| Inputs | 18, 17 answered, median 0 ticks, max 0; 14 with a sound; 87.5 % of input-free ticks also change something |
| Damage | 0 moment(s): hit stop 0, shake 0, floating text 0, sound 0, tween 0, particles 0 |
| Idle | 20.89 % of ticks |
| Particles | peak 0 system(s), 0 burst(s) |
| Frame (metal) | step 1.95 ms median, 2.24 p95; render 0.86 median, 1 p95; frame p95 3.2, max 4.09 |
| Audio | 2.8 s, peak 0.26, RMS 0.0201, 0 clipped, 83.93 % silent ticks |

### hit

a hit stops time briefly, shakes, floats its number and thuds on the sfx bus (72 ticks)

| Measure | Value |
|---|---|
| Text boxes | 0; 0 at 4.5:1; 0 of 0 at 9 px ink |
| HUD | 0 gameplay frame(s), up to 0 % of the surface, 0 % of the centre |
| UI | 1 screen change(s), 1 with motion; 73 ticks of UI motion; 1 press(es), 0 with a ui-bus sound |
| Inputs | 1, 1 answered, median 0 ticks, max 0; 1 with a sound; 100 % of input-free ticks also change something |
| Damage | 1 moment(s) from `hp`: hit stop 1, shake 1, floating text 1, sound 1, tween 0, particles 0 |
| Idle | 0 % of ticks |
| Particles | peak 0 system(s), 0 burst(s) |
| Frame (metal) | step 2.23 ms median, 2.51 p95; render 0.94 median, 1.35 p95; frame p95 3.83, max 4.18 |
| Audio | 1.2 s, peak 0.31, RMS 0.0213, 0 clipped, 87.5 % silent ticks |

### juice

an iris transition switches screens at full cover, the score counts up, UI events sound (210 ticks)

| Measure | Value |
|---|---|
| Text boxes | 49; 45 at 4.5:1; 49 of 49 at 9 px ink |
| Lowest contrast | 2.52:1 `Best` (iris-opening); 4.17:1 `NewBest` (landed); 4.35:1 `retry` (count-up) |
| Smallest ink | 9 px `Blurb` (iris-closing); 9 px `NewBest` (landed); 10 px `Blurb` (iris-closing) |
| HUD | 0 gameplay frame(s), up to 0 % of the surface, 0 % of the centre |
| UI | 5 screen change(s), 5 with motion; 191 ticks of UI motion; 1 press(es), 1 with a ui-bus sound |
| Inputs | 1, 1 answered, median 0 ticks, max 0; 1 with a sound; 90.22 % of input-free ticks also change something |
| Damage | 0 moment(s): hit stop 0, shake 0, floating text 0, sound 0, tween 0, particles 0 |
| Idle | 9 % of ticks |
| Particles | peak 0 system(s), 0 burst(s) |
| Frame (metal) | step 1.41 ms median, 3.73 p95; render 1.03 median, 1.31 p95; frame p95 5.01, max 96.11 |
| Audio | 3.5 s, peak 0.21, RMS 0.0182, 0 clipped, 87.62 % silent ticks |

![juice iris-closing](frames/juice-iris-closing.png) 

iris-closing (tick 27, menu: HUD 8.01 %, 19 texts, 0 under 4.5:1, 0 small)

### keys

keyboard focus opens a screen, steps a slider and closes the settings with a save (204 ticks)

| Measure | Value |
|---|---|
| Text boxes | 0; 0 at 4.5:1; 0 of 0 at 9 px ink |
| HUD | 0 gameplay frame(s), up to 0 % of the surface, 0 % of the centre |
| UI | 4 screen change(s), 4 with motion; 175 ticks of UI motion; 0 press(es), 0 with a ui-bus sound |
| Inputs | 7, 7 answered, median 0 ticks, max 0; 5 with a sound; 100 % of input-free ticks also change something |
| Damage | 0 moment(s): hit stop 0, shake 0, floating text 0, sound 0, tween 0, particles 0 |
| Idle | 13.92 % of ticks |
| Particles | peak 0 system(s), 0 burst(s) |
| Frame (metal) | step 1.4 ms median, 2.21 p95; render 0.87 median, 1.02 p95; frame p95 3.19, max 5.26 |
| Audio | 3.4 s, peak 0.21, RMS 0.0217, 0 clipped, 80.88 % silent ticks |

### locales

the menu, pause and confirm screens in en, zh-CN, ja, ko and ar match their reference frames (768 ticks)

| Measure | Value |
|---|---|
| Text boxes | 75; 61 at 4.5:1; 75 of 75 at 9 px ink |
| Lowest contrast | 2.52:1 `play` (mainMenu-ar); 2.67:1 `play` (mainMenu-ja); 2.75:1 `Confirm` (confirm-ja) |
| Smallest ink | 10 px `Footer` (mainMenu-en); 11 px `Subtitle` (mainMenu-en); 11 px `resume` (pause-en) |
| HUD | 0 gameplay frame(s), up to 0 % of the surface, 0 % of the centre |
| UI | 15 screen change(s), 11 with motion; 334 ticks of UI motion; 0 press(es), 0 with a ui-bus sound |
| Inputs | 0, 0 answered, median 0 ticks, max 0; 0 with a sound; 42.88 % of input-free ticks also change something |
| Damage | 0 moment(s): hit stop 0, shake 0, floating text 0, sound 0, tween 0, particles 0 |
| Idle | 57.12 % of ticks |
| Particles | peak 0 system(s), 0 burst(s) |
| Frame (metal) | step 1.16 ms median, 2.06 p95; render 0.83 median, 1.06 p95; frame p95 3.2, max 266.2 |
| Audio | 12.8 s, peak 0.16, RMS 0.0182, 0 clipped, 86.98 % silent ticks |

![locales mainMenu-en](frames/locales-mainMenu-en.png) 

mainMenu-en (tick 60, menu: HUD 10.17 %, 7 texts, 1 under 4.5:1, 0 small)

### multitouch

a stick held by one touch and a button tapped by another work together (72 ticks)

| Measure | Value |
|---|---|
| Text boxes | 0; 0 at 4.5:1; 0 of 0 at 9 px ink |
| HUD | 0 gameplay frame(s), up to 0 % of the surface, 0 % of the centre |
| UI | 0 screen change(s), 0 with motion; 71 ticks of UI motion; 2 press(es), 0 with a ui-bus sound |
| Inputs | 2, 2 answered, median 0 ticks, max 0; 1 with a sound; 100 % of input-free ticks also change something |
| Damage | 1 moment(s) from `hp`: hit stop 1, shake 1, floating text 1, sound 1, tween 0, particles 0 |
| Idle | 0 % of ticks |
| Particles | peak 0 system(s), 0 burst(s) |
| Frame (metal) | step 2.27 ms median, 3.08 p95; render 1.01 median, 1.55 p95; frame p95 5.15, max 9.5 |
| Audio | 1.2 s, peak 0.31, RMS 0.0213, 0 clipped, 87.5 % silent ticks |

### pad

gamepad navigation moves through the gallery, opens a screen and backs out (216 ticks)

| Measure | Value |
|---|---|
| Text boxes | 0; 0 at 4.5:1; 0 of 0 at 9 px ink |
| HUD | 0 gameplay frame(s), up to 0 % of the surface, 0 % of the centre |
| UI | 3 screen change(s), 3 with motion; 215 ticks of UI motion; 0 press(es), 0 with a ui-bus sound |
| Inputs | 0, 0 answered, median 0 ticks, max 0; 0 with a sound; 100 % of input-free ticks also change something |
| Damage | 0 moment(s): hit stop 0, shake 0, floating text 0, sound 0, tween 0, particles 0 |
| Idle | 0 % of ticks |
| Particles | peak 0 system(s), 0 burst(s) |
| Frame (metal) | step 2.05 ms median, 2.43 p95; render 0.86 median, 1.07 p95; frame p95 3.5, max 4.81 |
| Audio | 3.6 s, peak 0.26, RMS 0.0204, 0 clipped, 81.94 % silent ticks |

### phone

on a phone the HUD keeps to the safe area and its touch controls to 44 px (66 ticks)

| Measure | Value |
|---|---|
| Text boxes | 0; 0 at 4.5:1; 0 of 0 at 9 px ink |
| HUD | 0 gameplay frame(s), up to 0 % of the surface, 0 % of the centre |
| UI | 1 screen change(s), 1 with motion; 6 ticks of UI motion; 1 press(es), 0 with a ui-bus sound |
| Inputs | 1, 1 answered, median 1 ticks, max 1; 0 with a sound; 0 % of input-free ticks also change something |
| Damage | 0 moment(s): hit stop 0, shake 0, floating text 0, sound 0, tween 0, particles 0 |
| Idle | 96.43 % of ticks |
| Particles | peak 0 system(s), 0 burst(s) |
| Frame (metal) | step 1.52 ms median, 2.29 p95; render 1.07 median, 1.55 p95; frame p95 3.53, max 5.92 |
| Audio | 1.1 s, peak 0, RMS 0, 0 clipped, 100 % silent ticks |

### players

pads 1 and 2 move their own rings and pick for their own players (150 ticks)

| Measure | Value |
|---|---|
| Text boxes | 20; 20 at 4.5:1; 20 of 20 at 9 px ink |
| Lowest contrast | 8.04:1 `Name` (players); 8.47:1 `Pick` (players); 8.47:1 `Pick` (players) |
| Smallest ink | 11 px `Pick` (players); 11 px `Pick` (players); 11 px `Pick` (players) |
| HUD | 0 gameplay frame(s), up to 0 % of the surface, 0 % of the centre |
| UI | 1 screen change(s), 1 with motion; 149 ticks of UI motion; 2 press(es), 2 with a ui-bus sound |
| Inputs | 12, 12 answered, median 0 ticks, max 0; 7 with a sound; 100 % of input-free ticks also change something |
| Damage | 0 moment(s): hit stop 0, shake 0, floating text 0, sound 0, tween 0, particles 0 |
| Idle | 0 % of ticks |
| Particles | peak 0 system(s), 0 burst(s) |
| Frame (metal) | step 1.57 ms median, 2.42 p95; render 0.94 median, 1.2 p95; frame p95 3.53, max 80.79 |
| Audio | 2.5 s, peak 0.21, RMS 0.0224, 0 clipped, 82.67 % silent ticks |

![players players](frames/players-players.png) 

players (tick 90, menu: HUD 6.47 %, 20 texts, 0 under 4.5:1, 0 small)

### refuse

the shop refuses an item the balance cannot buy: Buy shakes and sounds (150 ticks)

| Measure | Value |
|---|---|
| Text boxes | 0; 0 at 4.5:1; 0 of 0 at 9 px ink |
| HUD | 0 gameplay frame(s), up to 0 % of the surface, 0 % of the centre |
| UI | 1 screen change(s), 1 with motion; 150 ticks of UI motion; 2 press(es), 2 with a ui-bus sound |
| Inputs | 2, 2 answered, median 0 ticks, max 0; 2 with a sound; 100 % of input-free ticks also change something |
| Damage | 0 moment(s): hit stop 0, shake 0, floating text 0, sound 0, tween 0, particles 0 |
| Idle | 0 % of ticks |
| Particles | peak 0 system(s), 0 burst(s) |
| Frame (metal) | step 1.44 ms median, 2.01 p95; render 0.83 median, 1.06 p95; frame p95 3, max 4.38 |
| Audio | 2.5 s, peak 0.39, RMS 0.0312, 0 clipped, 79.33 % silent ticks |

### skin-cartoon-phone

every screen in the cartoon style pack on a phone matches its cell of kit-skin-cartoon-phone.png (1044 ticks)

| Measure | Value |
|---|---|
| Text boxes | 223; 192 at 4.5:1; 214 of 223 at 9 px ink |
| Lowest contrast | 1.13:1 `Gallery` (widgets); 1.16:1 `Value` (racing); 1.75:1 `Weapon` (shooter) |
| Smallest ink | 8.33 px `Unit` (shop); 8.33 px `Name` (shop); 8.67 px `New` (widgets) |
| HUD | 5 gameplay frame(s), up to 14.61 % of the surface, 1.43 % of the centre |
| UI | 24 screen change(s), 22 with motion; 738 ticks of UI motion; 0 press(es), 0 with a ui-bus sound |
| Inputs | 0, 0 answered, median 0 ticks, max 0; 0 with a sound; 74.66 % of input-free ticks also change something |
| Damage | 0 moment(s): hit stop 0, shake 0, floating text 0, sound 0, tween 0, particles 0 |
| Idle | 25.34 % of ticks |
| Particles | peak 0 system(s), 0 burst(s) |
| Frame (metal) | step 1.28 ms median, 2.88 p95; render 0.93 median, 1.2 p95; frame p95 4.06, max 318.51 |
| Audio | 17.4 s, peak 0.16, RMS 0.0156, 0 clipped, 90.42 % silent ticks |

![skin-cartoon-phone gallery](frames/skin-cartoon-phone-gallery.png) 

gallery (tick 66, menu: HUD 26.16 %, 23 texts, 0 under 4.5:1, 0 small)

### skin-cartoon

every screen in the cartoon style pack matches its cell of kit-skin-cartoon.png (1044 ticks)

| Measure | Value |
|---|---|
| Text boxes | 227; 192 at 4.5:1; 226 of 227 at 9 px ink |
| Lowest contrast | 1.62:1 `Zone` (shooter); 1.67:1 `Percent` (loading); 1.79:1 `Rank` (leaderboard) |
| Smallest ink | 6 px `Gallery` (shooter); 9 px `Blurb` (gallery); 9 px `Chip#2` (widgets) |
| HUD | 5 gameplay frame(s), up to 4.28 % of the surface, 0.75 % of the centre |
| UI | 29 screen change(s), 29 with motion; 778 ticks of UI motion; 0 press(es), 0 with a ui-bus sound |
| Inputs | 0, 0 answered, median 0 ticks, max 0; 0 with a sound; 78.63 % of input-free ticks also change something |
| Damage | 0 moment(s): hit stop 0, shake 0, floating text 0, sound 0, tween 0, particles 0 |
| Idle | 21.37 % of ticks |
| Particles | peak 0 system(s), 0 burst(s) |
| Frame (metal) | step 1.29 ms median, 2.99 p95; render 0.95 median, 1.3 p95; frame p95 4.12, max 99.05 |
| Audio | 17.4 s, peak 0.16, RMS 0.0156, 0 clipped, 90.42 % silent ticks |

![skin-cartoon gallery](frames/skin-cartoon-gallery.png) 

gallery (tick 66, menu: HUD 8.01 %, 19 texts, 0 under 4.5:1, 0 small)

### skin-fantasy-phone

every screen in the fantasy style pack on a phone matches its cell of kit-skin-fantasy-phone.png (1044 ticks)

| Measure | Value |
|---|---|
| Text boxes | 225; 192 at 4.5:1; 214 of 225 at 9 px ink |
| Lowest contrast | 1.2:1 `Skip` (cutscene); 1.25:1 `Value` (racing); 1.34:1 `Gallery` (widgets) |
| Smallest ink | 7.67 px `Unit` (shop); 7.67 px `Name` (shop); 8.33 px `Name` (gallery) |
| HUD | 5 gameplay frame(s), up to 14.5 % of the surface, 1.43 % of the centre |
| UI | 25 screen change(s), 23 with motion; 730 ticks of UI motion; 0 press(es), 0 with a ui-bus sound |
| Inputs | 0, 0 answered, median 0 ticks, max 0; 0 with a sound; 73.79 % of input-free ticks also change something |
| Damage | 0 moment(s): hit stop 0, shake 0, floating text 0, sound 0, tween 0, particles 0 |
| Idle | 26.21 % of ticks |
| Particles | peak 0 system(s), 0 burst(s) |
| Frame (metal) | step 1.42 ms median, 2.81 p95; render 0.92 median, 1.15 p95; frame p95 3.9, max 850.42 |
| Audio | 17.4 s, peak 0.16, RMS 0.0156, 0 clipped, 90.42 % silent ticks |

![skin-fantasy-phone gallery](frames/skin-fantasy-phone-gallery.png) 

gallery (tick 66, menu: HUD 28.17 %, 23 texts, 0 under 4.5:1, 1 small)

### skin-fantasy

every screen in the fantasy style pack matches its cell of kit-skin-fantasy.png (1044 ticks)

| Measure | Value |
|---|---|
| Text boxes | 229; 182 at 4.5:1; 226 of 229 at 9 px ink |
| Lowest contrast | 1.23:1 `Gallery` (cutscene); 1.36:1 `Zone` (shooter); 1.65:1 `Caption` (puzzle) |
| Smallest ink | 7 px `Unit` (shop); 8 px `New` (widgets); 8 px `Name` (shop) |
| HUD | 5 gameplay frame(s), up to 4.27 % of the surface, 0.75 % of the centre |
| UI | 27 screen change(s), 27 with motion; 771 ticks of UI motion; 0 press(es), 0 with a ui-bus sound |
| Inputs | 0, 0 answered, median 0 ticks, max 0; 0 with a sound; 77.76 % of input-free ticks also change something |
| Damage | 0 moment(s): hit stop 0, shake 0, floating text 0, sound 0, tween 0, particles 0 |
| Idle | 22.24 % of ticks |
| Particles | peak 0 system(s), 0 burst(s) |
| Frame (metal) | step 1.74 ms median, 8.88 p95; render 1.05 median, 3.92 p95; frame p95 21.74, max 545.71 |
| Audio | 17.4 s, peak 0.16, RMS 0.0156, 0 clipped, 90.42 % silent ticks |

### skin-neon-phone

every screen in the neon style pack on a phone matches its cell of kit-skin-neon-phone.png (1044 ticks)

| Measure | Value |
|---|---|
| Text boxes | 231; 164 at 4.5:1; 217 of 231 at 9 px ink |
| Lowest contrast | 2.05:1 `Armor` (shooter); 2.22:1 `Price` (shop); 2.25:1 `Price` (shop) |
| Smallest ink | 7.67 px `New` (widgets); 7.67 px `Sale` (widgets); 7.67 px `Rare` (widgets) |
| HUD | 5 gameplay frame(s), up to 14.56 % of the surface, 1.43 % of the centre |
| UI | 25 screen change(s), 23 with motion; 718 ticks of UI motion; 0 press(es), 0 with a ui-bus sound |
| Inputs | 0, 0 answered, median 0 ticks, max 0; 0 with a sound; 72.82 % of input-free ticks also change something |
| Damage | 0 moment(s): hit stop 0, shake 0, floating text 0, sound 0, tween 0, particles 0 |
| Idle | 27.18 % of ticks |
| Particles | peak 0 system(s), 0 burst(s) |
| Frame (metal) | step 1.84 ms median, 4.12 p95; render 1.16 median, 1.94 p95; frame p95 5.92, max 537.6 |
| Audio | 17.4 s, peak 0.16, RMS 0.0156, 0 clipped, 90.42 % silent ticks |

### skin-neon

every screen in the neon style pack matches its cell of kit-skin-neon.png (1044 ticks)

| Measure | Value |
|---|---|
| Text boxes | 237; 180 at 4.5:1; 231 of 237 at 9 px ink |
| Lowest contrast | 1.65:1 `Gallery` (cutscene); 1.89:1 `Name` (levels); 1.99:1 `Confirm` (confirm) |
| Smallest ink | 8 px `New` (widgets); 8 px `Sale` (widgets); 8 px `Rare` (widgets) |
| HUD | 5 gameplay frame(s), up to 4.23 % of the surface, 0.75 % of the centre |
| UI | 28 screen change(s), 28 with motion; 757 ticks of UI motion; 0 press(es), 0 with a ui-bus sound |
| Inputs | 0, 0 answered, median 0 ticks, max 0; 0 with a sound; 76.79 % of input-free ticks also change something |
| Damage | 0 moment(s): hit stop 0, shake 0, floating text 0, sound 0, tween 0, particles 0 |
| Idle | 23.21 % of ticks |
| Particles | peak 0 system(s), 0 burst(s) |
| Frame (metal) | step 1.47 ms median, 3.02 p95; render 0.99 median, 1.21 p95; frame p95 4.18, max 118 |
| Audio | 17.4 s, peak 0.16, RMS 0.0156, 0 clipped, 90.42 % silent ticks |

### skin-paper-phone

every screen in the paper style pack on a phone matches its cell of kit-skin-paper-phone.png (1044 ticks)

| Measure | Value |
|---|---|
| Text boxes | 226; 197 at 4.5:1; 219 of 226 at 9 px ink |
| Lowest contrast | 1.12:1 `Gallery` (widgets); 1.2:1 `Ammo` (shooter); 1.25:1 `Value` (racing) |
| Smallest ink | 8.33 px `Count` (inventory); 8.67 px `Level` (idle); 8.67 px `Unit` (shop) |
| HUD | 5 gameplay frame(s), up to 14.59 % of the surface, 1.43 % of the centre |
| UI | 25 screen change(s), 23 with motion; 727 ticks of UI motion; 0 press(es), 0 with a ui-bus sound |
| Inputs | 0, 0 answered, median 0 ticks, max 0; 0 with a sound; 73.69 % of input-free ticks also change something |
| Damage | 0 moment(s): hit stop 0, shake 0, floating text 0, sound 0, tween 0, particles 0 |
| Idle | 26.31 % of ticks |
| Particles | peak 0 system(s), 0 burst(s) |
| Frame (metal) | step 1.52 ms median, 2.92 p95; render 0.97 median, 1.41 p95; frame p95 4.27, max 747.49 |
| Audio | 17.4 s, peak 0.16, RMS 0.0156, 0 clipped, 90.42 % silent ticks |

### skin-paper

every screen in the paper style pack matches its cell of kit-skin-paper.png (1044 ticks)

| Measure | Value |
|---|---|
| Text boxes | 229; 178 at 4.5:1; 229 of 229 at 9 px ink |
| Lowest contrast | 1.47:1 `Gallery` (cutscene); 1.63:1 `Zone` (shooter); 1.75:1 `Tap` (idle) |
| Smallest ink | 9 px `New` (widgets); 9 px `Sale` (widgets); 9 px `Rare` (widgets) |
| HUD | 5 gameplay frame(s), up to 4.25 % of the surface, 0.75 % of the centre |
| UI | 29 screen change(s), 29 with motion; 768 ticks of UI motion; 0 press(es), 0 with a ui-bus sound |
| Inputs | 0, 0 answered, median 0 ticks, max 0; 0 with a sound; 77.66 % of input-free ticks also change something |
| Damage | 0 moment(s): hit stop 0, shake 0, floating text 0, sound 0, tween 0, particles 0 |
| Idle | 22.34 % of ticks |
| Particles | peak 0 system(s), 0 burst(s) |
| Frame (metal) | step 1.65 ms median, 3.51 p95; render 1.09 median, 1.73 p95; frame p95 5.3, max 142.37 |
| Audio | 17.4 s, peak 0.16, RMS 0.0156, 0 clipped, 90.42 % silent ticks |

### skin-pixel-phone

every screen in the pixel style pack on a phone matches its cell of kit-skin-pixel-phone.png (1044 ticks)

| Measure | Value |
|---|---|
| Text boxes | 231; 208 at 4.5:1; 231 of 231 at 9 px ink |
| Lowest contrast | 1.66:1 `Health` (widgets); 1.74:1 `Boost` (racing); 2.17:1 `Gallery` (widgets) |
| Smallest ink | 9.33 px `Up` (platformer); 9.33 px `Down` (platformer); 9.33 px `Unit` (shop) |
| HUD | 5 gameplay frame(s), up to 14.61 % of the surface, 1.43 % of the centre |
| UI | 24 screen change(s), 22 with motion; 668 ticks of UI motion; 0 press(es), 0 with a ui-bus sound |
| Inputs | 0, 0 answered, median 0 ticks, max 0; 0 with a sound; 68.28 % of input-free ticks also change something |
| Damage | 0 moment(s): hit stop 0, shake 0, floating text 0, sound 0, tween 0, particles 0 |
| Idle | 30.75 % of ticks |
| Particles | peak 0 system(s), 0 burst(s) |
| Frame (metal) | step 2.53 ms median, 6.67 p95; render 1.24 median, 2.24 p95; frame p95 8.75, max 380.35 |
| Audio | 17.4 s, peak 0.16, RMS 0.0156, 0 clipped, 90.42 % silent ticks |

### skin-pixel

every screen in the pixel style pack matches its cell of kit-skin-pixel.png (1044 ticks)

| Measure | Value |
|---|---|
| Text boxes | 236; 195 at 4.5:1; 236 of 236 at 9 px ink |
| Lowest contrast | 2.05:1 `Price` (shop); 2.2:1 `Count` (inventory); 2.21:1 `Count` (inventory) |
| Smallest ink | 9 px `Up` (widgets); 9 px `Down` (widgets); 10 px `Pause` (platformer) |
| HUD | 5 gameplay frame(s), up to 4.23 % of the surface, 0.75 % of the centre |
| UI | 24 screen change(s), 24 with motion; 708 ticks of UI motion; 0 press(es), 0 with a ui-bus sound |
| Inputs | 0, 0 answered, median 0 ticks, max 0; 0 with a sound; 72.24 % of input-free ticks also change something |
| Damage | 0 moment(s): hit stop 0, shake 0, floating text 0, sound 0, tween 0, particles 0 |
| Idle | 26.79 % of ticks |
| Particles | peak 0 system(s), 0 burst(s) |
| Frame (metal) | step 2.66 ms median, 8.24 p95; render 1.25 median, 1.95 p95; frame p95 9.86, max 98.21 |
| Audio | 17.4 s, peak 0.16, RMS 0.0156, 0 clipped, 90.42 % silent ticks |

### skin-scifi-phone

every screen in the scifi style pack on a phone matches its cell of kit-skin-scifi-phone.png (1044 ticks)

| Measure | Value |
|---|---|
| Text boxes | 230; 198 at 4.5:1; 208 of 230 at 9 px ink |
| Lowest contrast | 1.59:1 `Armor` (shooter); 1.9:1 `Health` (widgets); 1.9:1 `Health` (shooter) |
| Smallest ink | 8 px `Unit` (shop); 8 px `Name` (shop); 8.33 px `Blurb` (gallery) |
| HUD | 5 gameplay frame(s), up to 14.4 % of the surface, 1.43 % of the centre |
| UI | 25 screen change(s), 23 with motion; 703 ticks of UI motion; 0 press(es), 0 with a ui-bus sound |
| Inputs | 0, 0 answered, median 0 ticks, max 0; 0 with a sound; 72.05 % of input-free ticks also change something |
| Damage | 0 moment(s): hit stop 0, shake 0, floating text 0, sound 0, tween 0, particles 0 |
| Idle | 27.95 % of ticks |
| Particles | peak 0 system(s), 0 burst(s) |
| Frame (metal) | step 1.66 ms median, 3.78 p95; render 1.54 median, 2.13 p95; frame p95 6.06, max 422.16 |
| Audio | 17.4 s, peak 0.16, RMS 0.0156, 0 clipped, 90.42 % silent ticks |

### skin-scifi

every screen in the scifi style pack matches its cell of kit-skin-scifi.png (1044 ticks)

| Measure | Value |
|---|---|
| Text boxes | 238; 200 at 4.5:1; 236 of 237 at 9 px ink |
| Lowest contrast | 1.55:1 `Gallery` (cutscene); 1.59:1 `Armor` (shooter); 1.9:1 `Health` (widgets) |
| Smallest ink | 8 px `Name` (shop); 9 px `Blurb` (gallery); 9 px `Blurb` (gallery) |
| HUD | 5 gameplay frame(s), up to 4.17 % of the surface, 0.75 % of the centre |
| UI | 28 screen change(s), 28 with motion; 743 ticks of UI motion; 0 press(es), 0 with a ui-bus sound |
| Inputs | 0, 0 answered, median 0 ticks, max 0; 0 with a sound; 76.02 % of input-free ticks also change something |
| Damage | 0 moment(s): hit stop 0, shake 0, floating text 0, sound 0, tween 0, particles 0 |
| Idle | 23.98 % of ticks |
| Particles | peak 0 system(s), 0 burst(s) |
| Frame (metal) | step 1.68 ms median, 3.93 p95; render 1.51 median, 2.68 p95; frame p95 6.66, max 136.78 |
| Audio | 17.4 s, peak 0.16, RMS 0.0156, 0 clipped, 90.42 % silent ticks |

### sounds

a press sounds on the ui bus by default, and not after sounds:off (90 ticks)

| Measure | Value |
|---|---|
| Text boxes | 0; 0 at 4.5:1; 0 of 0 at 9 px ink |
| HUD | 0 gameplay frame(s), up to 0 % of the surface, 0 % of the centre |
| UI | 5 screen change(s), 5 with motion; 89 ticks of UI motion; 2 press(es), 1 with a ui-bus sound |
| Inputs | 4, 4 answered, median 0 ticks, max 0; 2 with a sound; 100 % of input-free ticks also change something |
| Damage | 0 moment(s): hit stop 0, shake 0, floating text 0, sound 0, tween 0, particles 0 |
| Idle | 0 % of ticks |
| Particles | peak 0 system(s), 0 burst(s) |
| Frame (metal) | step 2.1 ms median, 2.47 p95; render 0.87 median, 1.13 p95; frame p95 3.61, max 4.84 |
| Audio | 1.5 s, peak 0.21, RMS 0.0229, 0 clipped, 75.56 % silent ticks |

### touch

touch_move_to and move_to_text drive the gallery, the shop, results, idle and the cutscene (450 ticks)

| Measure | Value |
|---|---|
| Text boxes | 0; 0 at 4.5:1; 0 of 0 at 9 px ink |
| HUD | 0 gameplay frame(s), up to 0 % of the surface, 0 % of the centre |
| UI | 8 screen change(s), 8 with motion; 384 ticks of UI motion; 9 press(es), 6 with a ui-bus sound |
| Inputs | 11, 11 answered, median 0 ticks, max 0; 8 with a sound; 98.28 % of input-free ticks also change something |
| Damage | 0 moment(s): hit stop 0, shake 0, floating text 0, sound 0, tween 0, particles 0 |
| Idle | 6.59 % of ticks |
| Particles | peak 0 system(s), 0 burst(s) |
| Frame (metal) | step 1.57 ms median, 2.44 p95; render 0.87 median, 1.4 p95; frame p95 3.71, max 12.62 |
| Audio | 7.5 s, peak 0.21, RMS 0.0219, 0 clipped, 82.22 % silent ticks |

### tutorial

the tutorial spotlights each target and Next walks to the end (150 ticks)

| Measure | Value |
|---|---|
| Text boxes | 0; 0 at 4.5:1; 0 of 0 at 9 px ink |
| HUD | 0 gameplay frame(s), up to 0 % of the surface, 0 % of the centre |
| UI | 3 screen change(s), 3 with motion; 149 ticks of UI motion; 4 press(es), 3 with a ui-bus sound |
| Inputs | 4, 4 answered, median 0 ticks, max 0; 3 with a sound; 100 % of input-free ticks also change something |
| Damage | 0 moment(s): hit stop 0, shake 0, floating text 0, sound 0, tween 0, particles 0 |
| Idle | 0 % of ticks |
| Particles | peak 0 system(s), 0 burst(s) |
| Frame (metal) | step 1.5 ms median, 2.38 p95; render 0.87 median, 1.2 p95; frame p95 3.55, max 7.06 |
| Audio | 2.5 s, peak 0.21, RMS 0.0193, 0 clipped, 88 % silent ticks |

