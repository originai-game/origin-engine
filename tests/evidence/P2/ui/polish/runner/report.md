# og polish: runner

2026-10-19, 8 playtest(s) on metal. Mechanical measures of og-taste and og-feel (og-build §5); a score is 0 to 100, n/a when nothing in the playtests exercises it. The measures cannot judge style: read the frames.

## Scores

| | Score | Measure |
|---|---|---|
| **Taste** | **n/a** | mean of the taste rows |
| Contrast | n/a | text boxes whose ink and ground reach 4.5:1 |
| Text size | n/a | text boxes whose tallest ink line reaches 9 CSS px (a 13 px face's capitals) |
| HUD occupancy | n/a | gameplay frames: HUD at most 15 % of the surface and 5 % of the central third |
| Screen transitions | n/a | screen changes with UI motion around them |
| UI sounds | n/a | UI presses answered on the ui bus |
| Locale keys | n/a | string tables without missing keys (og cook) |
| **Feel** | **95** | mean of the feel rows |
| Input latency | 100 | inputs answered, by the median ticks to the answer (0–1: 100) |
| Hit feedback | n/a | damage moments with hit stop, shake, floating text or a sound (three of four: 100) |
| Idle frames | 100 | ticks where nothing new is seen or heard (10 %: 100, 60 %: 0) |
| Frame time | 80 | p95 step + render (8.3 ms: 100, 33 ms: 0), worst playtest |
| Sound on input | 100 | inputs answered by a sound within 6 ticks |

## Fixes

No finding crossed a threshold.

## Playtests

### coins

a coin in reach is collected and scores; one out of reach is not (180 ticks)

| Measure | Value |
|---|---|
| Text boxes | 0; 0 at 4.5:1; 0 of 0 at 9 px ink |
| HUD | 0 gameplay frame(s), up to 0 % of the surface, 0 % of the centre |
| UI | 0 screen change(s), 0 with motion; 0 ticks of UI motion; 0 press(es), 0 with a ui-bus sound |
| Inputs | 1, 1 answered, median 0 ticks, max 0; 1 with a sound; 82.73 % of input-free ticks also change something |
| Damage | 0 moment(s): hit stop 0, shake 0, floating text 0, sound 0, tween 0, particles 0 |
| Idle | 0 % of ticks |
| Particles | peak 0 system(s), 0 burst(s) |
| Frame (metal) | step 2.4 ms median, 6.54 p95; render 2.4 median, 7.45 p95; frame p95 12.69, max 41.71 |
| Audio | 3 s, peak 0.14, RMS 0.0171, 0 clipped, 77.78 % silent ticks |

### flow

title, run, crash on a crate, game over, run again (306 ticks)

| Measure | Value |
|---|---|
| Text boxes | 0; 0 at 4.5:1; 0 of 0 at 9 px ink |
| HUD | 0 gameplay frame(s), up to 0 % of the surface, 0 % of the centre |
| UI | 0 screen change(s), 0 with motion; 0 ticks of UI motion; 0 press(es), 0 with a ui-bus sound |
| Inputs | 3, 3 answered, median 0 ticks, max 0; 3 with a sound; 72.41 % of input-free ticks also change something |
| Damage | 0 moment(s): hit stop 0, shake 0, floating text 0, sound 0, tween 0, particles 0 |
| Idle | 5.07 % of ticks |
| Particles | peak 0 system(s), 0 burst(s) |
| Frame (metal) | step 2.38 ms median, 7.51 p95; render 2.22 median, 6.6 p95; frame p95 13.34, max 180.02 |
| Audio | 5.1 s, peak 0.43, RMS 0.0269, 0 clipped, 76.14 % silent ticks |

![flow title](frames/flow-title.png) 

title (tick 18: HUD 0 %, 0 texts, 0 under 4.5:1, 0 small)

### frames

reference frames: title, a coin's pickup, the crash (180 ticks)

| Measure | Value |
|---|---|
| Text boxes | 0; 0 at 4.5:1; 0 of 0 at 9 px ink |
| HUD | 0 gameplay frame(s), up to 0 % of the surface, 0 % of the centre |
| UI | 0 screen change(s), 0 with motion; 0 ticks of UI motion; 0 press(es), 0 with a ui-bus sound |
| Inputs | 1, 1 answered, median 0 ticks, max 0; 1 with a sound; 78.42 % of input-free ticks also change something |
| Damage | 0 moment(s): hit stop 0, shake 0, floating text 0, sound 0, tween 0, particles 0 |
| Idle | 0 % of ticks |
| Particles | peak 0 system(s), 0 burst(s) |
| Frame (metal) | step 2.05 ms median, 6.11 p95; render 2.16 median, 5.58 p95; frame p95 11.01, max 151.6 |
| Audio | 3 s, peak 0.43, RMS 0.03, 0 clipped, 73.33 % silent ticks |

![frames title](frames/frames-title.png) 

title (tick 18: HUD 0 %, 0 texts, 0 under 4.5:1, 0 small)

### jump

a full jump clears a crate and a fence; a hop clears a crate, not a fence (522 ticks)

| Measure | Value |
|---|---|
| Text boxes | 0; 0 at 4.5:1; 0 of 0 at 9 px ink |
| HUD | 0 gameplay frame(s), up to 0 % of the surface, 0 % of the centre |
| UI | 0 screen change(s), 0 with motion; 0 ticks of UI motion; 0 press(es), 0 with a ui-bus sound |
| Inputs | 5, 5 answered, median 0 ticks, max 0; 5 with a sound; 84.87 % of input-free ticks also change something |
| Damage | 0 moment(s): hit stop 0, shake 0, floating text 0, sound 0, tween 0, particles 0 |
| Idle | 0.98 % of ticks |
| Particles | peak 0 system(s), 0 burst(s) |
| Frame (metal) | step 1.96 ms median, 6.83 p95; render 2.17 median, 5.82 p95; frame p95 12.4, max 210.2 |
| Audio | 8.7 s, peak 0.43, RMS 0.0197, 0 clipped, 81.99 % silent ticks |

![jump jump](frames/jump-jump.png) 

jump (tick 114: HUD 0 %, 0 texts, 0 under 4.5:1, 0 small)

### records

the best score is read from the store, and a crash submits the score and saves the new best (228 ticks)

| Measure | Value |
|---|---|
| Text boxes | 0; 0 at 4.5:1; 0 of 0 at 9 px ink |
| HUD | 0 gameplay frame(s), up to 0 % of the surface, 0 % of the centre |
| UI | 0 screen change(s), 0 with motion; 0 ticks of UI motion; 0 press(es), 0 with a ui-bus sound |
| Inputs | 1, 1 answered, median 0 ticks, max 0; 1 with a sound; 83.96 % of input-free ticks also change something |
| Damage | 0 moment(s): hit stop 0, shake 0, floating text 0, sound 0, tween 0, particles 0 |
| Idle | 0 % of ticks |
| Particles | peak 0 system(s), 0 burst(s) |
| Frame (metal) | step 2.02 ms median, 5.55 p95; render 2.2 median, 5.36 p95; frame p95 9.49, max 22.06 |
| Audio | 3.8 s, peak 0.43, RMS 0.0278, 0 clipped, 78.95 % silent ticks |

### replay

replay: two runs of jumps, coins and a crash end on one hash (546 ticks)

| Measure | Value |
|---|---|
| Text boxes | 0; 0 at 4.5:1; 0 of 0 at 9 px ink |
| HUD | 0 gameplay frame(s), up to 0 % of the surface, 0 % of the centre |
| UI | 0 screen change(s), 0 with motion; 0 ticks of UI motion; 0 press(es), 0 with a ui-bus sound |
| Inputs | 5, 5 answered, median 0 ticks, max 0; 5 with a sound; 83.46 % of input-free ticks also change something |
| Damage | 0 moment(s): hit stop 0, shake 0, floating text 0, sound 0, tween 0, particles 0 |
| Idle | 1.31 % of ticks |
| Particles | peak 0 system(s), 0 burst(s) |
| Frame (metal) | step 2.04 ms median, 5 p95; render 2.18 median, 4.95 p95; frame p95 9.06, max 17.54 |
| Audio | 9.1 s, peak 0.43, RMS 0.0213, 0 clipped, 77.47 % silent ticks |

### sounds

a round's start, a jump, a coin and a crash each play their sound (180 ticks)

| Measure | Value |
|---|---|
| Text boxes | 0; 0 at 4.5:1; 0 of 0 at 9 px ink |
| HUD | 0 gameplay frame(s), up to 0 % of the surface, 0 % of the centre |
| UI | 0 screen change(s), 0 with motion; 0 ticks of UI motion; 0 press(es), 0 with a ui-bus sound |
| Inputs | 2, 2 answered, median 0 ticks, max 0; 2 with a sound; 75.93 % of input-free ticks also change something |
| Damage | 0 moment(s): hit stop 0, shake 0, floating text 0, sound 0, tween 0, particles 0 |
| Idle | 0 % of ticks |
| Particles | peak 0 system(s), 0 burst(s) |
| Frame (metal) | step 2.27 ms median, 5.38 p95; render 2.21 median, 5.19 p95; frame p95 9.26, max 16.68 |
| Audio | 3 s, peak 0.43, RMS 0.0326, 0 clipped, 58.33 % silent ticks |

### tune

set writes Tuning: start speed and jump height change the run (150 ticks)

| Measure | Value |
|---|---|
| Text boxes | 0; 0 at 4.5:1; 0 of 0 at 9 px ink |
| HUD | 0 gameplay frame(s), up to 0 % of the surface, 0 % of the centre |
| UI | 0 screen change(s), 0 with motion; 0 ticks of UI motion; 0 press(es), 0 with a ui-bus sound |
| Inputs | 2, 2 answered, median 0 ticks, max 0; 2 with a sound; 75.64 % of input-free ticks also change something |
| Damage | 0 moment(s): hit stop 0, shake 0, floating text 0, sound 0, tween 0, particles 0 |
| Idle | 0 % of ticks |
| Particles | peak 0 system(s), 0 burst(s) |
| Frame (metal) | step 2.21 ms median, 5.25 p95; render 2.15 median, 4.78 p95; frame p95 10.07, max 19.41 |
| Audio | 2.5 s, peak 0.14, RMS 0.0184, 0 clipped, 77.33 % silent ticks |

