# og polish: fx-showcase

2026-10-18, 9 playtest(s) on metal. Mechanical measures of og-taste and og-feel (og-build §5); a score is 0 to 100, n/a when nothing in the playtests exercises it. The measures cannot judge style: read the frames.

## Scores

| | Score | Measure |
|---|---|---|
| **Taste** | **100** | mean of the taste rows |
| Contrast | 100 | text boxes whose ink and ground reach 4.5:1 |
| Text size | 100 | text boxes whose tallest ink line reaches 9 CSS px (a 13 px face's capitals) |
| HUD occupancy | 100 | gameplay frames: HUD at most 15 % of the surface and 5 % of the central third |
| Screen transitions | n/a | screen changes with UI motion around them |
| UI sounds | n/a | UI presses answered on the ui bus |
| Locale keys | n/a | string tables without missing keys (og cook) |
| **Feel** | **97** | mean of the feel rows |
| Input latency | n/a | inputs answered, by the median ticks to the answer (0–1: 100) |
| Hit feedback | n/a | damage moments with hit stop, shake, floating text or a sound (three of four: 100) |
| Idle frames | 100 | ticks where nothing new is seen or heard (10 %: 100, 60 %: 0) |
| Frame time | 93 | p95 step + render (8.3 ms: 100, 33 ms: 0), worst playtest |
| Sound on input | n/a | inputs answered by a sound within 6 ticks |

## Fixes

No finding crossed a threshold.

## Playtests

### controller

the controller station matches its reference and behaves (342 ticks)

| Measure | Value |
|---|---|
| Text boxes | 2; 2 at 4.5:1; 2 of 2 at 9 px ink |
| Lowest contrast | 15.41:1 `BadgeText` (controller-before); 15.41:1 `BadgeText` (controller) |
| Smallest ink | 16 px `BadgeText` (controller-before); 16 px `BadgeText` (controller) |
| HUD | 2 gameplay frame(s), up to 0.35 % of the surface, 0 % of the centre |
| UI | 0 screen change(s), 0 with motion; 0 ticks of UI motion; 0 press(es), 0 with a ui-bus sound |
| Inputs | 0, 0 answered, median 0 ticks, max 0; 0 with a sound; 100 % of input-free ticks also change something |
| Damage | 0 moment(s): hit stop 0, shake 0, floating text 0, sound 0, tween 0, particles 0 |
| Idle | 0 % of ticks |
| Particles | peak 4 system(s), 0 burst(s) |
| Frame (metal) | step 0.9 ms median, 1.18 p95; render 4.44 median, 4.97 p95; frame p95 6.13, max 126.57 |
| Audio | 5.7 s, peak 0, RMS 0, 0 clipped, 100 % silent ticks |

![controller controller-before](frames/controller-controller-before.png) 

controller-before (tick 300: HUD 0.35 %, 1 texts, 0 under 4.5:1, 0 small)

### fluid

the fluid station matches its reference and behaves (222 ticks)

| Measure | Value |
|---|---|
| Text boxes | 2; 2 at 4.5:1; 2 of 2 at 9 px ink |
| Lowest contrast | 15.41:1 `BadgeText` (fluid-before); 15.41:1 `BadgeText` (fluid) |
| Smallest ink | 16 px `BadgeText` (fluid-before); 16 px `BadgeText` (fluid) |
| HUD | 2 gameplay frame(s), up to 0.35 % of the surface, 0 % of the centre |
| UI | 0 screen change(s), 0 with motion; 0 ticks of UI motion; 0 press(es), 0 with a ui-bus sound |
| Inputs | 0, 0 answered, median 0 ticks, max 0; 0 with a sound; 100 % of input-free ticks also change something |
| Damage | 0 moment(s): hit stop 0, shake 0, floating text 0, sound 0, tween 0, particles 0 |
| Idle | 0 % of ticks |
| Particles | peak 4 system(s), 0 burst(s) |
| Frame (metal) | step 1.98 ms median, 2.3 p95; render 3.51 median, 4.92 p95; frame p95 6.22, max 113.27 |
| Audio | 3.7 s, peak 0, RMS 0, 0 clipped, 100 % silent ticks |

![fluid fluid-before](frames/fluid-fluid-before.png) 

fluid-before (tick 180: HUD 0.35 %, 1 texts, 0 under 4.5:1, 0 small)

### frames

every station matches its reference frame (432 ticks)

| Measure | Value |
|---|---|
| Text boxes | 5; 5 at 4.5:1; 5 of 5 at 9 px ink |
| Lowest contrast | 15.41:1 `BadgeText` (post); 15.41:1 `BadgeText` (lens); 15.41:1 `BadgeText` (fx) |
| Smallest ink | 16 px `BadgeText` (post); 16 px `BadgeText` (lens); 16 px `BadgeText` (fx) |
| HUD | 5 gameplay frame(s), up to 0.35 % of the surface, 0 % of the centre |
| UI | 0 screen change(s), 0 with motion; 0 ticks of UI motion; 0 press(es), 0 with a ui-bus sound |
| Inputs | 0, 0 answered, median 0 ticks, max 0; 0 with a sound; 100 % of input-free ticks also change something |
| Damage | 0 moment(s): hit stop 0, shake 0, floating text 0, sound 0, tween 0, particles 0 |
| Idle | 0 % of ticks |
| Particles | peak 4 system(s), 0 burst(s) |
| Frame (metal) | step 0.86 ms median, 2.04 p95; render 4.59 median, 5.14 p95; frame p95 6.48, max 922.06 |
| Audio | 7.2 s, peak 0, RMS 0, 0 clipped, 100 % silent ticks |

### joints

the joints station matches its reference and behaves (222 ticks)

| Measure | Value |
|---|---|
| Text boxes | 2; 2 at 4.5:1; 2 of 2 at 9 px ink |
| Lowest contrast | 15.41:1 `BadgeText` (joints-before); 15.41:1 `BadgeText` (joints) |
| Smallest ink | 16 px `BadgeText` (joints-before); 16 px `BadgeText` (joints) |
| HUD | 2 gameplay frame(s), up to 0.35 % of the surface, 0 % of the centre |
| UI | 0 screen change(s), 0 with motion; 0 ticks of UI motion; 0 press(es), 0 with a ui-bus sound |
| Inputs | 0, 0 answered, median 0 ticks, max 0; 0 with a sound; 100 % of input-free ticks also change something |
| Damage | 0 moment(s): hit stop 0, shake 0, floating text 0, sound 0, tween 0, particles 0 |
| Idle | 0 % of ticks |
| Particles | peak 4 system(s), 0 burst(s) |
| Frame (metal) | step 0.94 ms median, 1.23 p95; render 4.6 median, 5.1 p95; frame p95 6.38, max 129.8 |
| Audio | 3.7 s, peak 0, RMS 0, 0 clipped, 100 % silent ticks |

### rope

the rope station matches its reference and behaves (192 ticks)

| Measure | Value |
|---|---|
| Text boxes | 2; 2 at 4.5:1; 2 of 2 at 9 px ink |
| Lowest contrast | 15.41:1 `BadgeText` (rope-before); 15.41:1 `BadgeText` (rope) |
| Smallest ink | 16 px `BadgeText` (rope-before); 16 px `BadgeText` (rope) |
| HUD | 2 gameplay frame(s), up to 0.35 % of the surface, 0 % of the centre |
| UI | 0 screen change(s), 0 with motion; 0 ticks of UI motion; 0 press(es), 0 with a ui-bus sound |
| Inputs | 0, 0 answered, median 0 ticks, max 0; 0 with a sound; 100 % of input-free ticks also change something |
| Damage | 0 moment(s): hit stop 0, shake 0, floating text 0, sound 0, tween 0, particles 0 |
| Idle | 0 % of ticks |
| Particles | peak 4 system(s), 0 burst(s) |
| Frame (metal) | step 0.91 ms median, 1.18 p95; render 4.88 median, 5.25 p95; frame p95 6.35, max 143.44 |
| Audio | 3.2 s, peak 0, RMS 0, 0 clipped, 100 % silent ticks |

### shaders

the shaders station matches its reference and behaves (132 ticks)

| Measure | Value |
|---|---|
| Text boxes | 2; 2 at 4.5:1; 2 of 2 at 9 px ink |
| Lowest contrast | 15.41:1 `BadgeText` (shaders-before); 15.41:1 `BadgeText` (shaders) |
| Smallest ink | 16 px `BadgeText` (shaders-before); 16 px `BadgeText` (shaders) |
| HUD | 2 gameplay frame(s), up to 0.35 % of the surface, 0 % of the centre |
| UI | 0 screen change(s), 0 with motion; 0 ticks of UI motion; 0 press(es), 0 with a ui-bus sound |
| Inputs | 0, 0 answered, median 0 ticks, max 0; 0 with a sound; 100 % of input-free ticks also change something |
| Damage | 0 moment(s): hit stop 0, shake 0, floating text 0, sound 0, tween 0, particles 0 |
| Idle | 0 % of ticks |
| Particles | peak 4 system(s), 0 burst(s) |
| Frame (metal) | step 1.67 ms median, 2.08 p95; render 7.69 median, 8.11 p95; frame p95 10.03, max 415.72 |
| Audio | 2.2 s, peak 0, RMS 0, 0 clipped, 100 % silent ticks |

### sky

the sky station matches its reference and behaves (132 ticks)

| Measure | Value |
|---|---|
| Text boxes | 2; 2 at 4.5:1; 2 of 2 at 9 px ink |
| Lowest contrast | 15.41:1 `BadgeText` (sky-before); 15.41:1 `BadgeText` (sky) |
| Smallest ink | 16 px `BadgeText` (sky-before); 16 px `BadgeText` (sky) |
| HUD | 2 gameplay frame(s), up to 0.35 % of the surface, 0 % of the centre |
| UI | 0 screen change(s), 0 with motion; 0 ticks of UI motion; 0 press(es), 0 with a ui-bus sound |
| Inputs | 0, 0 answered, median 0 ticks, max 0; 0 with a sound; 100 % of input-free ticks also change something |
| Damage | 0 moment(s): hit stop 0, shake 0, floating text 0, sound 0, tween 0, particles 0 |
| Idle | 0 % of ticks |
| Particles | peak 4 system(s), 0 burst(s) |
| Frame (metal) | step 0.84 ms median, 1.07 p95; render 6.01 median, 6.38 p95; frame p95 7.69, max 333.3 |
| Audio | 2.2 s, peak 0, RMS 0, 0 clipped, 100 % silent ticks |

### water

the water station matches its reference and behaves (222 ticks)

| Measure | Value |
|---|---|
| Text boxes | 2; 2 at 4.5:1; 2 of 2 at 9 px ink |
| Lowest contrast | 15.41:1 `BadgeText` (water-before); 15.41:1 `BadgeText` (water) |
| Smallest ink | 16 px `BadgeText` (water-before); 16 px `BadgeText` (water) |
| HUD | 2 gameplay frame(s), up to 0.35 % of the surface, 0 % of the centre |
| UI | 0 screen change(s), 0 with motion; 0 ticks of UI motion; 0 press(es), 0 with a ui-bus sound |
| Inputs | 0, 0 answered, median 0 ticks, max 0; 0 with a sound; 100 % of input-free ticks also change something |
| Damage | 0 moment(s): hit stop 0, shake 0, floating text 0, sound 0, tween 0, particles 0 |
| Idle | 0 % of ticks |
| Particles | peak 4 system(s), 0 burst(s) |
| Frame (metal) | step 0.85 ms median, 1.08 p95; render 4.64 median, 5.03 p95; frame p95 6.09, max 548.71 |
| Audio | 3.7 s, peak 0, RMS 0, 0 clipped, 100 % silent ticks |

### weather

the weather station matches its reference and behaves (312 ticks)

| Measure | Value |
|---|---|
| Text boxes | 2; 2 at 4.5:1; 2 of 2 at 9 px ink |
| Lowest contrast | 15.41:1 `BadgeText` (weather-before); 15.41:1 `BadgeText` (weather) |
| Smallest ink | 16 px `BadgeText` (weather-before); 16 px `BadgeText` (weather) |
| HUD | 2 gameplay frame(s), up to 0.35 % of the surface, 0 % of the centre |
| UI | 0 screen change(s), 0 with motion; 0 ticks of UI motion; 0 press(es), 0 with a ui-bus sound |
| Inputs | 0, 0 answered, median 0 ticks, max 0; 0 with a sound; 100 % of input-free ticks also change something |
| Damage | 0 moment(s): hit stop 0, shake 0, floating text 0, sound 0, tween 0, particles 0 |
| Idle | 0 % of ticks |
| Particles | peak 5 system(s), 1 burst(s) |
| Frame (metal) | step 0.75 ms median, 0.99 p95; render 4.35 median, 4.82 p95; frame p95 5.79, max 193.91 |
| Audio | 5.2 s, peak 0, RMS 0, 0 clipped, 100 % silent ticks |

