# og polish: lantern-vale

2026-10-25, 11 playtest(s) on metal. Mechanical measures of og-taste and og-feel (og-build §5); a score is 0 to 100, n/a when nothing in the playtests exercises it. The measures cannot judge style: read the frames.

## Scores

| | Score | Measure |
|---|---|---|
| **Taste** | **100** | mean of the taste rows |
| Contrast | 100 | text boxes whose ink and ground reach 4.5:1 |
| Text size | 100 | text boxes whose tallest ink line reaches 9 CSS px (a 13 px face's capitals) |
| HUD occupancy | 100 | gameplay frames: HUD at most 15 % of the surface and 5 % of the central third |
| Screen transitions | 100 | screen changes with UI motion around them |
| UI sounds | 100 | UI presses answered on the ui bus |
| Locale keys | 100 | string tables without missing keys (og cook) |
| **Feel** | **96** | mean of the feel rows |
| Input latency | 100 | inputs answered, by the median ticks to the answer (0–1: 100) |
| Hit feedback | 100 | damage moments with hit stop, shake, floating text or a sound (three of four: 100) |
| Idle frames | 100 | ticks where nothing new is seen or heard (10 %: 100, 60 %: 0) |
| Frame time | 82 | p95 step + render (8.3 ms: 100, 33 ms: 0), worst playtest |
| Sound on input | 96 | inputs answered by a sound within 6 ticks |

## Fixes

No finding crossed a threshold.

## Playtests

### flow

Set out, walk and run on the terrain, find a lantern by an overlap query, light it with E (342 ticks)

| Measure | Value |
|---|---|
| Text boxes | 0; 0 at 4.5:1; 0 of 0 at 9 px ink |
| HUD | 0 gameplay frame(s), up to 0 % of the surface, 0 % of the centre |
| UI | 3 screen change(s), 3 with motion; 158 ticks of UI motion; 1 press(es), 1 with a ui-bus sound |
| Inputs | 4, 4 answered, median 0 ticks, max 0; 4 with a sound; 74.32 % of input-free ticks also change something |
| Damage | 0 moment(s): hit stop 0, shake 0, floating text 0, sound 0, tween 0, particles 0 |
| Idle | 0 % of ticks |
| Particles | peak 1 system(s), 1 burst(s) |
| Frame (metal) | step 2.62 ms median, 6.17 p95; render 2.34 median, 2.69 p95; frame p95 8.47, max 13.16 |
| Audio | 5.7 s, peak 0.24, RMS 0.0354, 0 clipped, 0 % silent ticks |

### frames

the title, a run, a lantern, a slime, the far side, the Chinese settings and the dawn results match their reference frames (684 ticks)

| Measure | Value |
|---|---|
| Text boxes | 54; 54 at 4.5:1; 54 of 54 at 9 px ink |
| Lowest contrast | 4.65:1 `NewBest` (results); 5.51:1 `LanguageLabel` (settings-zh); 6.04:1 `Name` (settings-zh) |
| Smallest ink | 9 px `NewBest` (results); 11 px `play` (title); 11 px `Toast#2` (far) |
| HUD | 4 gameplay frame(s), up to 2.14 % of the surface, 0 % of the centre |
| UI | 9 screen change(s), 9 with motion; 572 ticks of UI motion; 1 press(es), 1 with a ui-bus sound |
| Inputs | 2, 2 answered, median 0 ticks, max 0; 2 with a sound; 86.56 % of input-free ticks also change something |
| Damage | 2 moment(s) from `hits`, `hp`: hit stop 2, shake 2, floating text 2, sound 2, tween 0, particles 1 |
| Idle | 0 % of ticks |
| Particles | peak 4 system(s), 4 burst(s) |
| Frame (metal) | step 2.87 ms median, 6.35 p95; render 2.56 median, 3.01 p95; frame p95 9.13, max 21.62 |
| Audio | 11.4 s, peak 0.62, RMS 0.0628, 0 clipped, 0 % silent ticks |

![frames title](frames/frames-title.png) 

title (tick 60, menu: HUD 5.83 %, 5 texts, 0 under 4.5:1, 0 small)

### locales

the title, the HUD, the pause menu and the results in Chinese, then English (402 ticks)

| Measure | Value |
|---|---|
| Text boxes | 5; 5 at 4.5:1; 5 of 5 at 9 px ink |
| Lowest contrast | 4.56:1 `settings` (title-zh); 7.48:1 `play` (title-zh); 8.13:1 `Footer` (title-zh) |
| Smallest ink | 12 px `Footer` (title-zh); 15 px `Subtitle` (title-zh); 15 px `play` (title-zh) |
| HUD | 0 gameplay frame(s), up to 0 % of the surface, 0 % of the centre |
| UI | 7 screen change(s), 7 with motion; 363 ticks of UI motion; 0 press(es), 0 with a ui-bus sound |
| Inputs | 2, 2 answered, median 1 ticks, max 1; 2 with a sound; 89.39 % of input-free ticks also change something |
| Damage | 0 moment(s): hit stop 0, shake 0, floating text 0, sound 0, tween 0, particles 0 |
| Idle | 0 % of ticks |
| Particles | peak 4 system(s), 4 burst(s) |
| Frame (metal) | step 2.98 ms median, 8.9 p95; render 2.38 median, 2.99 p95; frame p95 12.06, max 18.67 |
| Audio | 6.7 s, peak 0.44, RMS 0.0689, 0 clipped, 0 % silent ticks |

![locales title-zh](frames/locales-title-zh.png) 

title-zh (tick 54, menu: HUD 5.25 %, 5 texts, 0 under 4.5:1, 0 small)

### lose

no hearts left, night at dusk's end, Again and Title (462 ticks)

| Measure | Value |
|---|---|
| Text boxes | 0; 0 at 4.5:1; 0 of 0 at 9 px ink |
| HUD | 0 gameplay frame(s), up to 0 % of the surface, 0 % of the centre |
| UI | 7 screen change(s), 7 with motion; 231 ticks of UI motion; 2 press(es), 2 with a ui-bus sound |
| Inputs | 2, 2 answered, median 0 ticks, max 0; 2 with a sound; 50 % of input-free ticks also change something |
| Damage | 1 moment(s) from `hits`, `hp`: hit stop 1, shake 1, floating text 1, sound 1, tween 0, particles 0 |
| Idle | 0 % of ticks |
| Particles | peak 0 system(s), 0 burst(s) |
| Frame (metal) | step 2.71 ms median, 9.32 p95; render 2.27 median, 3.02 p95; frame p95 12.38, max 17.71 |
| Audio | 7.7 s, peak 0.6, RMS 0.0616, 0 clipped, 0 % silent ticks |

### music

the chunked music plays from the title, its chunks streamed (192 ticks)

| Measure | Value |
|---|---|
| Text boxes | 0; 0 at 4.5:1; 0 of 0 at 9 px ink |
| HUD | 0 gameplay frame(s), up to 0 % of the surface, 0 % of the centre |
| UI | 0 screen change(s), 0 with motion; 191 ticks of UI motion; 0 press(es), 0 with a ui-bus sound |
| Inputs | 0, 0 answered, median 0 ticks, max 0; 0 with a sound; 100 % of input-free ticks also change something |
| Damage | 0 moment(s): hit stop 0, shake 0, floating text 0, sound 0, tween 0, particles 0 |
| Idle | 0 % of ticks |
| Particles | peak 0 system(s), 0 burst(s) |
| Frame (metal) | step 2.92 ms median, 3.33 p95; render 2.51 median, 2.83 p95; frame p95 6.01, max 13.16 |
| Audio | 3.2 s, peak 0.04, RMS 0.0092, 0 clipped, 0 % silent ticks |

### physics

the character controller shoves the pile of rocks and is stopped by a trunk (270 ticks)

| Measure | Value |
|---|---|
| Text boxes | 0; 0 at 4.5:1; 0 of 0 at 9 px ink |
| HUD | 0 gameplay frame(s), up to 0 % of the surface, 0 % of the centre |
| UI | 1 screen change(s), 1 with motion; 34 ticks of UI motion; 0 press(es), 0 with a ui-bus sound |
| Inputs | 2, 2 answered, median 0 ticks, max 0; 2 with a sound; 88.38 % of input-free ticks also change something |
| Damage | 0 moment(s): hit stop 0, shake 0, floating text 0, sound 0, tween 0, particles 0 |
| Idle | 0 % of ticks |
| Particles | peak 0 system(s), 0 burst(s) |
| Frame (metal) | step 2.6 ms median, 7.97 p95; render 2.33 median, 2.85 p95; frame p95 10.81, max 14.58 |
| Audio | 4.5 s, peak 0.05, RMS 0.0072, 0 clipped, 0 % silent ticks |

### replay

a session with the pile, a lantern, a slime's touch, the far side and the results replays to the same hash (576 ticks)

| Measure | Value |
|---|---|
| Text boxes | 0; 0 at 4.5:1; 0 of 0 at 9 px ink |
| HUD | 0 gameplay frame(s), up to 0 % of the surface, 0 % of the centre |
| UI | 5 screen change(s), 5 with motion; 338 ticks of UI motion; 0 press(es), 0 with a ui-bus sound |
| Inputs | 3, 3 answered, median 0 ticks, max 0; 3 with a sound; 84.57 % of input-free ticks also change something |
| Damage | 2 moment(s) from `hits`, `hp`: hit stop 2, shake 2, floating text 2, sound 2, tween 0, particles 1 |
| Idle | 0 % of ticks |
| Particles | peak 4 system(s), 4 burst(s) |
| Frame (metal) | step 2.76 ms median, 7.57 p95; render 2.48 median, 3.02 p95; frame p95 10.65, max 18.53 |
| Audio | 9.6 s, peak 0.6, RMS 0.0655, 0 clipped, 0 % silent ticks |

### settings

pause, then the settings: a slider, a typed name with an input method, a mixed-script label, Chinese, saved (282 ticks)

| Measure | Value |
|---|---|
| Text boxes | 26; 26 at 4.5:1; 26 of 26 at 9 px ink |
| Lowest contrast | 5.51:1 `LanguageLabel` (settings-zh); 7.35:1 `Done` (settings-zh); 7.47:1 `Label` (settings-zh) |
| Smallest ink | 11 px `Value` (settings); 12 px `Label` (settings); 12 px `Value` (settings) |
| HUD | 0 gameplay frame(s), up to 0 % of the surface, 0 % of the centre |
| UI | 6 screen change(s), 6 with motion; 279 ticks of UI motion; 5 press(es), 5 with a ui-bus sound |
| Inputs | 7, 7 answered, median 0 ticks, max 1; 6 with a sound; 100 % of input-free ticks also change something |
| Damage | 0 moment(s): hit stop 0, shake 0, floating text 0, sound 0, tween 0, particles 0 |
| Idle | 0 % of ticks |
| Particles | peak 0 system(s), 0 burst(s) |
| Frame (metal) | step 3.11 ms median, 7.68 p95; render 2.15 median, 2.62 p95; frame p95 11.05, max 13.25 |
| Audio | 4.7 s, peak 0.51, RMS 0.0438, 0 clipped, 0 % silent ticks |

![settings settings](frames/settings-settings.png) 

settings (tick 198, menu: HUD 8.87 %, 13 texts, 0 under 4.5:1, 0 small)

### slimes

a slime sees the hero, turns alert and chases on twos, and its touch lands with weight (150 ticks)

| Measure | Value |
|---|---|
| Text boxes | 0; 0 at 4.5:1; 0 of 0 at 9 px ink |
| HUD | 0 gameplay frame(s), up to 0 % of the surface, 0 % of the centre |
| UI | 1 screen change(s), 1 with motion; 34 ticks of UI motion; 0 press(es), 0 with a ui-bus sound |
| Inputs | 0, 0 answered, median 0 ticks, max 0; 0 with a sound; 40 % of input-free ticks also change something |
| Damage | 1 moment(s) from `hits`, `hp`: hit stop 1, shake 1, floating text 1, sound 1, tween 0, particles 0 |
| Idle | 0 % of ticks |
| Particles | peak 0 system(s), 0 burst(s) |
| Frame (metal) | step 2.64 ms median, 8.84 p95; render 2.39 median, 3.04 p95; frame p95 11.91, max 13.88 |
| Audio | 2.5 s, peak 0.47, RMS 0.0326, 0 clipped, 0 % silent ticks |

### stream

the far side's streaming group loads as the camera nears it and unloads as it leaves, replaying the same (270 ticks)

| Measure | Value |
|---|---|
| Text boxes | 0; 0 at 4.5:1; 0 of 0 at 9 px ink |
| HUD | 0 gameplay frame(s), up to 0 % of the surface, 0 % of the centre |
| UI | 2 screen change(s), 2 with motion; 46 ticks of UI motion; 0 press(es), 0 with a ui-bus sound |
| Inputs | 2, 2 answered, median 0 ticks, max 0; 2 with a sound; 56.33 % of input-free ticks also change something |
| Damage | 0 moment(s): hit stop 0, shake 0, floating text 0, sound 0, tween 0, particles 0 |
| Idle | 0 % of ticks |
| Particles | peak 0 system(s), 0 burst(s) |
| Frame (metal) | step 2.58 ms median, 7.77 p95; render 2.3 median, 2.81 p95; frame p95 10.55, max 13.38 |
| Audio | 4.5 s, peak 0.06, RMS 0.0077, 0 clipped, 0 % silent ticks |

### win

four lanterns lit: the results under the dawn look, then Again under the dusk look (263 ticks)

| Measure | Value |
|---|---|
| Text boxes | 0; 0 at 4.5:1; 0 of 0 at 9 px ink |
| HUD | 0 gameplay frame(s), up to 0 % of the surface, 0 % of the centre |
| UI | 6 screen change(s), 6 with motion; 250 ticks of UI motion; 1 press(es), 1 with a ui-bus sound |
| Inputs | 1, 1 answered, median 0 ticks, max 0; 1 with a sound; 99.09 % of input-free ticks also change something |
| Damage | 0 moment(s): hit stop 0, shake 0, floating text 0, sound 0, tween 0, particles 0 |
| Idle | 0 % of ticks |
| Particles | peak 4 system(s), 4 burst(s) |
| Frame (metal) | step 3.06 ms median, 9.86 p95; render 2.38 median, 2.97 p95; frame p95 12.91, max 18.4 |
| Audio | 4.4 s, peak 0.43, RMS 0.0862, 0 clipped, 0 % silent ticks |

