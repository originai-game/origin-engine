# og polish: idle

2026-10-19, 10 playtest(s) on metal. Mechanical measures of og-taste and og-feel (og-build §5); a score is 0 to 100, n/a when nothing in the playtests exercises it. The measures cannot judge style: read the frames.

## Scores

| | Score | Measure |
|---|---|---|
| **Taste** | **97** | mean of the taste rows |
| Contrast | 91 | text boxes whose ink and ground reach 4.5:1 |
| Text size | 100 | text boxes whose tallest ink line reaches 9 CSS px (a 13 px face's capitals) |
| HUD occupancy | 100 | gameplay frames: HUD at most 15 % of the surface and 5 % of the central third |
| Screen transitions | 100 | screen changes with UI motion around them |
| UI sounds | 92 | UI presses answered on the ui bus |
| Locale keys | 100 | string tables without missing keys (og cook) |
| **Feel** | **74** | mean of the feel rows |
| Input latency | 100 | inputs answered, by the median ticks to the answer (0–1: 100) |
| Hit feedback | n/a | damage moments with hit stop, shake, floating text or a sound (three of four: 100) |
| Idle frames | 100 | ticks where nothing new is seen or heard (10 %: 100, 60 %: 0) |
| Frame time | 0 | p95 step + render (8.3 ms: 100, 33 ms: 0), worst playtest |
| Sound on input | 95 | inputs answered by a sound within 6 ticks |

## Fixes

| Measure | Where | Found | Fix |
|---|---|---|---|
| frame time | away | p95 step + render 91.78 ms on metal (the frame is 16.7 ms) | og-game §14 (performance): fewer draws and particles, cheaper systems; og run --json measures |
| UI sounds | flow | 2 of 13 UI presses played no sound on the ui bus | og-taste §6 (feedback) and og-audio §3: give the skin or og.ui.configure its press and open sounds |
| frame time | flow | p95 step + render 57.89 ms on metal (the frame is 16.7 ms) | og-game §14 (performance): fewer draws and particles, cheaper systems; og run --json measures |
| contrast | frames orbit: `Cost` "81" | 3.98:1 between ink and ground (body text wants 4.5:1) | og-taste §3 (value before hue): darken the ground or lighten the ink in the skin's palette, never per call |
| contrast | frames orbit: `Cost` "19.8K" | 4.44:1 between ink and ground (body text wants 4.5:1) | og-taste §3 (value before hue): darken the ground or lighten the ink in the skin's palette, never per call |
| frame time | frames | p95 step + render 23.33 ms on metal (the frame is 16.7 ms) | og-game §14 (performance): fewer draws and particles, cheaper systems; og run --json measures |
| contrast | milestone milestone: `Cost` "1,200" | 3.68:1 between ink and ground (body text wants 4.5:1) | og-taste §3 (value before hue): darken the ground or lighten the ink in the skin's palette, never per call |
| contrast | milestone milestone: `Cost` "15" | 4.1:1 between ink and ground (body text wants 4.5:1) | og-taste §3 (value before hue): darken the ground or lighten the ink in the skin's palette, never per call |
| frame time | milestone | p95 step + render 76.33 ms on metal (the frame is 16.7 ms) | og-game §14 (performance): fewer draws and particles, cheaper systems; og run --json measures |
| frame time | nova | p95 step + render 46.6 ms on metal (the frame is 16.7 ms) | og-game §14 (performance): fewer draws and particles, cheaper systems; og run --json measures |
| contrast | offline welcome: `Collect` "Collect" | 3.05:1 between ink and ground (body text wants 4.5:1) | og-taste §3 (value before hue): darken the ground or lighten the ink in the skin's palette, never per call |
| frame time | offline | p95 step + render 33.34 ms on metal (the frame is 16.7 ms) | og-game §14 (performance): fewer draws and particles, cheaper systems; og run --json measures |
| frame time | pad | p95 step + render 20.59 ms on metal (the frame is 16.7 ms) | og-game §14 (performance): fewer draws and particles, cheaper systems; og run --json measures |
| contrast | pause settings-zh: `Done` "完成" | 2.97:1 between ink and ground (body text wants 4.5:1) | og-taste §3 (value before hue): darken the ground or lighten the ink in the skin's palette, never per call |
| contrast | pause play-zh: `Name` "晶簇" | 3.1:1 between ink and ground (body text wants 4.5:1) | og-taste §3 (value before hue): darken the ground or lighten the ink in the skin's palette, never per call |
| contrast | pause play-zh: `Cost` "1,200" | 3.68:1 between ink and ground (body text wants 4.5:1) | og-taste §3 (value before hue): darken the ground or lighten the ink in the skin's palette, never per call |
| frame time | pause | p95 step + render 68.19 ms on metal (the frame is 16.7 ms) | og-game §14 (performance): fewer draws and particles, cheaper systems; og run --json measures |
| contrast | phone play: `Cost` "18" | 4.13:1 between ink and ground (body text wants 4.5:1) | og-taste §3 (value before hue): darken the ground or lighten the ink in the skin's palette, never per call |
| contrast | phone play: `Amount` "85" | 4.3:1 between ink and ground (body text wants 4.5:1) | og-taste §3 (value before hue): darken the ground or lighten the ink in the skin's palette, never per call |
| UI sounds | phone | 1 of 4 UI presses played no sound on the ui bus | og-taste §6 (feedback) and og-audio §3: give the skin or og.ui.configure its press and open sounds |
| frame time | phone | p95 step + render 24.26 ms on metal (the frame is 16.7 ms) | og-game §14 (performance): fewer draws and particles, cheaper systems; og run --json measures |
| frame time | replay | p95 step + render 24.44 ms on metal (the frame is 16.7 ms) | og-game §14 (performance): fewer draws and particles, cheaper systems; og run --json measures |

## Playtests

### away

two hours away by the host's clock, across processes, credited at half the rate (96 ticks)

| Measure | Value |
|---|---|
| Text boxes | 0; 0 at 4.5:1; 0 of 0 at 9 px ink |
| HUD | 0 gameplay frame(s), up to 0 % of the surface, 0 % of the centre |
| UI | 2 screen change(s), 2 with motion; 94 ticks of UI motion; 0 press(es), 0 with a ui-bus sound |
| Inputs | 1, 1 answered, median 0 ticks, max 0; 1 with a sound; 100 % of input-free ticks also change something |
| Damage | 0 moment(s): hit stop 0, shake 0, floating text 0, sound 0, tween 0, particles 0 |
| Idle | 0 % of ticks |
| Particles | peak 2 system(s), 0 burst(s) |
| Frame (metal) | step 12.53 ms median, 49.05 p95; render 13.64 median, 68.06 p95; frame p95 91.78, max 154.88 |
| Audio | 1.6 s, peak 0.24, RMS 0.028, 0 clipped, 0 % silent ticks |

### flow

taps make sparks, sparks buy embers, embers make sparks, a purchase too dear is refused (390 ticks)

| Measure | Value |
|---|---|
| Text boxes | 20; 20 at 4.5:1; 20 of 20 at 9 px ink |
| Lowest contrast | 4.61:1 `Cost` (tapped); 4.73:1 `Amount` (tapped); 4.9:1 `Owned` (tapped) |
| Smallest ink | 11 px `Rate` (tapped); 11 px `Name` (tapped); 11 px `Name` (tapped) |
| HUD | 1 gameplay frame(s), up to 2.08 % of the surface, 0 % of the centre |
| UI | 1 screen change(s), 1 with motion; 388 ticks of UI motion; 13 press(es), 11 with a ui-bus sound |
| Inputs | 17, 17 answered, median 0 ticks, max 0; 16 with a sound; 100 % of input-free ticks also change something |
| Damage | 0 moment(s): hit stop 0, shake 0, floating text 0, sound 0, tween 0, particles 0 |
| Idle | 0 % of ticks |
| Particles | peak 8 system(s), 6 burst(s) |
| Frame (metal) | step 8.73 ms median, 32.7 p95; render 10.17 median, 30.48 p95; frame p95 57.89, max 345.48 |
| Audio | 6.5 s, peak 0.21, RMS 0.035, 0 clipped, 0 % silent ticks |

![flow tapped](frames/flow-tapped.png) 

tapped (tick 213: HUD 2.08 %, 20 texts, 0 under 4.5:1, 0 small)

### frames

the title, the orb with its motes and the supernova's card match their reference frames (246 ticks)

| Measure | Value |
|---|---|
| Text boxes | 38; 36 at 4.5:1; 38 of 38 at 9 px ink |
| Lowest contrast | 3.98:1 `Cost` (orbit); 4.44:1 `Cost` (orbit); 4.71:1 `Cost` (orbit) |
| Smallest ink | 11 px `Footer` (title); 11 px `Rate` (orbit); 11 px `Name` (orbit) |
| HUD | 1 gameplay frame(s), up to 2.04 % of the surface, 0 % of the centre |
| UI | 3 screen change(s), 3 with motion; 244 ticks of UI motion; 0 press(es), 0 with a ui-bus sound |
| Inputs | 0, 0 answered, median 0 ticks, max 0; 0 with a sound; 100 % of input-free ticks also change something |
| Damage | 0 moment(s): hit stop 0, shake 0, floating text 0, sound 0, tween 0, particles 0 |
| Idle | 0 % of ticks |
| Particles | peak 3 system(s), 1 burst(s) |
| Frame (metal) | step 5.99 ms median, 13.59 p95; render 7.92 median, 13.71 p95; frame p95 23.33, max 396.14 |
| Audio | 4.1 s, peak 0.53, RMS 0.0638, 0 clipped, 0 % silent ticks |

![frames title](frames/frames-title.png) 

title (tick 54, menu: HUD 5.25 %, 5 texts, 0 under 4.5:1, 0 small)

### milestone

the tenth of a kind doubles it; big numbers read 1.23M (90 ticks)

| Measure | Value |
|---|---|
| Text boxes | 21; 19 at 4.5:1; 21 of 21 at 9 px ink |
| Lowest contrast | 3.68:1 `Cost` (milestone); 4.1:1 `Cost` (milestone); 4.5:1 `Cost` (milestone) |
| Smallest ink | 11 px `Rate` (milestone); 11 px `Name` (milestone); 11 px `Name` (milestone) |
| HUD | 1 gameplay frame(s), up to 1.99 % of the surface, 0 % of the centre |
| UI | 2 screen change(s), 2 with motion; 88 ticks of UI motion; 0 press(es), 0 with a ui-bus sound |
| Inputs | 1, 1 answered, median 0 ticks, max 0; 1 with a sound; 100 % of input-free ticks also change something |
| Damage | 0 moment(s): hit stop 0, shake 0, floating text 0, sound 0, tween 0, particles 0 |
| Idle | 0 % of ticks |
| Particles | peak 3 system(s), 1 burst(s) |
| Frame (metal) | step 9.18 ms median, 47.35 p95; render 11.24 median, 44.51 p95; frame p95 76.33, max 559.98 |
| Audio | 1.5 s, peak 0.25, RMS 0.0291, 0 clipped, 0 % silent ticks |

![milestone milestone](frames/milestone-milestone.png) 

milestone (tick 87: HUD 1.99 %, 21 texts, 2 under 4.5:1, 0 small)

### nova

a supernova: asked, declined, collapsed, the run's card, the next run stronger (282 ticks)

| Measure | Value |
|---|---|
| Text boxes | 12; 12 at 4.5:1; 12 of 12 at 9 px ink |
| Lowest contrast | 5.89:1 `Value` (card); 5.89:1 `Again` (card); 7.13:1 `Value` (card) |
| Smallest ink | 11 px `Key` (card); 11 px `Key` (card); 12 px `Value` (card) |
| HUD | 0 gameplay frame(s), up to 0 % of the surface, 0 % of the centre |
| UI | 10 screen change(s), 10 with motion; 275 ticks of UI motion; 4 press(es), 4 with a ui-bus sound |
| Inputs | 6, 6 answered, median 0 ticks, max 0; 6 with a sound; 100 % of input-free ticks also change something |
| Damage | 0 moment(s): hit stop 0, shake 0, floating text 0, sound 0, tween 0, particles 0 |
| Idle | 0 % of ticks |
| Particles | peak 3 system(s), 1 burst(s) |
| Frame (metal) | step 7.73 ms median, 25.3 p95; render 8.21 median, 27.54 p95; frame p95 46.6, max 264.57 |
| Audio | 4.7 s, peak 0.74, RMS 0.068, 0 clipped, 0 % silent ticks |

![nova card](frames/nova-card.png) 

card (tick 201, menu: HUD 4.28 %, 12 texts, 0 under 4.5:1, 0 small)

### offline

the save, two hours away credited at half the rate, a stale write merged (264 ticks)

| Measure | Value |
|---|---|
| Text boxes | 3; 2 at 4.5:1; 3 of 3 at 9 px ink |
| Lowest contrast | 3.05:1 `Collect` (welcome); 14.39:1 `Title` (welcome); 14.81:1 `Body` (welcome) |
| Smallest ink | 13 px `Body` (welcome); 16 px `Title` (welcome); 26 px `Collect` (welcome) |
| HUD | 0 gameplay frame(s), up to 0 % of the surface, 0 % of the centre |
| UI | 8 screen change(s), 8 with motion; 262 ticks of UI motion; 2 press(es), 2 with a ui-bus sound |
| Inputs | 4, 4 answered, median 0 ticks, max 0; 4 with a sound; 100 % of input-free ticks also change something |
| Damage | 0 moment(s): hit stop 0, shake 0, floating text 0, sound 0, tween 0, particles 0 |
| Idle | 0 % of ticks |
| Particles | peak 2 system(s), 0 burst(s) |
| Frame (metal) | step 8.35 ms median, 18.29 p95; render 8.91 median, 16.9 p95; frame p95 33.34, max 295.14 |
| Audio | 4.4 s, peak 0.36, RMS 0.0475, 0 clipped, 0 % silent ticks |

![offline welcome](frames/offline-welcome.png) 

welcome (tick 195, menu: HUD 5.4 %, 3 texts, 1 under 4.5:1, 0 small)

### pad

A, the d-pad, X, Y, Start and B play the orb (294 ticks)

| Measure | Value |
|---|---|
| Text boxes | 0; 0 at 4.5:1; 0 of 0 at 9 px ink |
| HUD | 0 gameplay frame(s), up to 0 % of the surface, 0 % of the centre |
| UI | 6 screen change(s), 6 with motion; 292 ticks of UI motion; 0 press(es), 0 with a ui-bus sound |
| Inputs | 14, 14 answered, median 0 ticks, max 0; 12 with a sound; 100 % of input-free ticks also change something |
| Damage | 0 moment(s): hit stop 0, shake 0, floating text 0, sound 0, tween 0, particles 0 |
| Idle | 0 % of ticks |
| Particles | peak 5 system(s), 3 burst(s) |
| Frame (metal) | step 6.5 ms median, 10.98 p95; render 7.42 median, 11.48 p95; frame p95 20.59, max 76.58 |
| Audio | 4.9 s, peak 0.23, RMS 0.0462, 0 clipped, 0 % silent ticks |

### pause

Escape pauses, settings switch to 简体中文 and save, numbers read in 万 and 亿 (330 ticks)

| Measure | Value |
|---|---|
| Text boxes | 37; 30 at 4.5:1; 37 of 37 at 9 px ink |
| Lowest contrast | 2.97:1 `Done` (settings-zh); 3.1:1 `Name` (play-zh); 3.68:1 `Cost` (play-zh) |
| Smallest ink | 11 px `resume` (pause); 12 px `quit` (pause); 12 px `Value` (settings-zh) |
| HUD | 1 gameplay frame(s), up to 1.83 % of the surface, 0 % of the centre |
| UI | 8 screen change(s), 8 with motion; 328 ticks of UI motion; 5 press(es), 5 with a ui-bus sound |
| Inputs | 6, 6 answered, median 0 ticks, max 0; 6 with a sound; 100 % of input-free ticks also change something |
| Damage | 0 moment(s): hit stop 0, shake 0, floating text 0, sound 0, tween 0, particles 0 |
| Idle | 0 % of ticks |
| Particles | peak 2 system(s), 0 burst(s) |
| Frame (metal) | step 10.33 ms median, 38.76 p95; render 11.24 median, 31.21 p95; frame p95 68.19, max 458.08 |
| Audio | 5.5 s, peak 0.41, RMS 0.0367, 0 clipped, 0 % silent ticks |

![pause pause](frames/pause-pause.png) 

pause (tick 120, menu: HUD 6.8 %, 4 texts, 0 under 4.5:1, 0 small)

### phone

the phone layout: a HUD inside the safe area, the orb tapped, a card bought (168 ticks)

| Measure | Value |
|---|---|
| Text boxes | 20; 18 at 4.5:1; 20 of 20 at 9 px ink |
| Lowest contrast | 4.13:1 `Cost` (play); 4.3:1 `Amount` (play); 5.11:1 `Owned` (play) |
| Smallest ink | 10.33 px `Rate` (play); 10.33 px `Name` (play); 10.33 px `Name` (play) |
| HUD | 1 gameplay frame(s), up to 6.62 % of the surface, 0 % of the centre |
| UI | 1 screen change(s), 1 with motion; 166 ticks of UI motion; 4 press(es), 3 with a ui-bus sound |
| Inputs | 4, 4 answered, median 0 ticks, max 0; 4 with a sound; 100 % of input-free ticks also change something |
| Damage | 0 moment(s): hit stop 0, shake 0, floating text 0, sound 0, tween 0, particles 0 |
| Idle | 0 % of ticks |
| Particles | peak 4 system(s), 2 burst(s) |
| Frame (metal) | step 6.92 ms median, 11.87 p95; render 8.42 median, 14.39 p95; frame p95 24.26, max 901.16 |
| Audio | 2.8 s, peak 0.23, RMS 0.0239, 0 clipped, 0 % silent ticks |

![phone play](frames/phone-play.png) 

play (tick 165: HUD 6.62 %, 20 texts, 2 under 4.5:1, 0 small)

### replay

a run with taps, purchases and a milestone replays to the same hash (216 ticks)

| Measure | Value |
|---|---|
| Text boxes | 0; 0 at 4.5:1; 0 of 0 at 9 px ink |
| HUD | 0 gameplay frame(s), up to 0 % of the surface, 0 % of the centre |
| UI | 1 screen change(s), 1 with motion; 214 ticks of UI motion; 9 press(es), 9 with a ui-bus sound |
| Inputs | 13, 13 answered, median 0 ticks, max 0; 13 with a sound; 100 % of input-free ticks also change something |
| Damage | 0 moment(s): hit stop 0, shake 0, floating text 0, sound 0, tween 0, particles 0 |
| Idle | 0 % of ticks |
| Particles | peak 9 system(s), 7 burst(s) |
| Frame (metal) | step 7.98 ms median, 13.57 p95; render 8.44 median, 13.47 p95; frame p95 24.44, max 46.46 |
| Audio | 3.6 s, peak 0.22, RMS 0.0376, 0 clipped, 0 % silent ticks |

