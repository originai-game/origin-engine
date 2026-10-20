# og polish: third-person

2026-10-20, 10 playtest(s) on metal. Mechanical measures of og-taste and og-feel (og-build §5); a score is 0 to 100, n/a when nothing in the playtests exercises it. The measures cannot judge style: read the frames.

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
| **Feel** | **69** | mean of the feel rows |
| Input latency | 100 | inputs answered, by the median ticks to the answer (0–1: 100) |
| Hit feedback | n/a | damage moments with hit stop, shake, floating text or a sound (three of four: 100) |
| Idle frames | 100 | ticks where nothing new is seen or heard (10 %: 100, 60 %: 0) |
| Frame time | 0 | p95 step + render (8.3 ms: 100, 33 ms: 0), worst playtest |
| Sound on input | 76 | inputs answered by a sound within 6 ticks |

## Fixes

| Measure | Where | Found | Fix |
|---|---|---|---|
| frame time | flow | p95 step + render 155.28 ms on metal (the frame is 16.7 ms) | og-game §14 (performance): fewer draws and particles, cheaper systems; og run --json measures |
| frame time | frames | p95 step + render 134.08 ms on metal (the frame is 16.7 ms) | og-game §14 (performance): fewer draws and particles, cheaper systems; og run --json measures |
| frame time | hitstop | p95 step + render 50.29 ms on metal (the frame is 16.7 ms) | og-game §14 (performance): fewer draws and particles, cheaper systems; og run --json measures |
| frame time | lose | p95 step + render 26.19 ms on metal (the frame is 16.7 ms) | og-game §14 (performance): fewer draws and particles, cheaper systems; og run --json measures |
| frame time | pause | p95 step + render 61.16 ms on metal (the frame is 16.7 ms) | og-game §14 (performance): fewer draws and particles, cheaper systems; og run --json measures |
| frame time | restart | p95 step + render 19.65 ms on metal (the frame is 16.7 ms) | og-game §14 (performance): fewer draws and particles, cheaper systems; og run --json measures |
| frame time | win | p95 step + render 19.33 ms on metal (the frame is 16.7 ms) | og-game §14 (performance): fewer draws and particles, cheaper systems; og run --json measures |

## Playtests

### flow

Set out, walk, run and wade on the terrain, kindle a beacon with the hand in the bowl (636 ticks)

| Measure | Value |
|---|---|
| Text boxes | 0; 0 at 4.5:1; 0 of 0 at 9 px ink |
| HUD | 0 gameplay frame(s), up to 0 % of the surface, 0 % of the centre |
| UI | 3 screen change(s), 3 with motion; 182 ticks of UI motion; 1 press(es), 1 with a ui-bus sound |
| Inputs | 4, 4 answered, median 0 ticks, max 0; 2 with a sound; 33.91 % of input-free ticks also change something |
| Damage | 0 moment(s): hit stop 0, shake 0, floating text 0, sound 0, tween 0, particles 0 |
| Idle | 0 % of ticks |
| Particles | peak 12 system(s), 2 burst(s) |
| Frame (metal) | step 10.51 ms median, 111.26 p95; render 10.66 median, 105.89 p95; frame p95 155.28, max 357.2 |
| Audio | 10.6 s, peak 0.32, RMS 0.023, 0 clipped, 0 % silent ticks |

### frames

the title, a beacon catching and the dawn summary match their reference frames (438 ticks)

| Measure | Value |
|---|---|
| Text boxes | 22; 22 at 4.5:1; 22 of 22 at 9 px ink |
| Lowest contrast | 4.65:1 `settings` (title); 5.17:1 `Menu` (dawn); 5.51:1 `Footer` (title) |
| Smallest ink | 11 px `play` (title); 11 px `Toast#1` (beacon); 11 px `Text` (beacon) |
| HUD | 1 gameplay frame(s), up to 1.44 % of the surface, 0 % of the centre |
| UI | 4 screen change(s), 4 with motion; 407 ticks of UI motion; 0 press(es), 0 with a ui-bus sound |
| Inputs | 1, 1 answered, median 0 ticks, max 0; 0 with a sound; 93.45 % of input-free ticks also change something |
| Damage | 0 moment(s): hit stop 0, shake 0, floating text 0, sound 0, tween 0, particles 0 |
| Idle | 0 % of ticks |
| Particles | peak 12 system(s), 4 burst(s) |
| Frame (metal) | step 7.86 ms median, 77.98 p95; render 10.4 median, 56.71 p95; frame p95 134.08, max 3503.95 |
| Audio | 7.3 s, peak 0.44, RMS 0.0493, 0 clipped, 0 % silent ticks |

![frames title](frames/frames-title.png) 

title (tick 54, menu: HUD 5.83 %, 5 texts, 0 under 4.5:1, 0 small)

### hitstop

Escape pressed during a grab's hit-stop pauses the run (66 ticks)

| Measure | Value |
|---|---|
| Text boxes | 0; 0 at 4.5:1; 0 of 0 at 9 px ink |
| HUD | 0 gameplay frame(s), up to 0 % of the surface, 0 % of the centre |
| UI | 2 screen change(s), 2 with motion; 60 ticks of UI motion; 0 press(es), 0 with a ui-bus sound |
| Inputs | 1, 1 answered, median 1 ticks, max 1; 1 with a sound; 92.86 % of input-free ticks also change something |
| Damage | 0 moment(s): hit stop 0, shake 0, floating text 0, sound 0, tween 0, particles 0 |
| Idle | 0 % of ticks |
| Particles | peak 12 system(s), 0 burst(s) |
| Frame (metal) | step 12.4 ms median, 28.94 p95; render 12.95 median, 24.87 p95; frame p95 50.29, max 63.76 |
| Audio | 1.1 s, peak 0.31, RMS 0.042, 0 clipped, 0 % silent ticks |

### lose

caught with no hearts left, night at dusk's end, Again and Title (522 ticks)

| Measure | Value |
|---|---|
| Text boxes | 0; 0 at 4.5:1; 0 of 0 at 9 px ink |
| HUD | 0 gameplay frame(s), up to 0 % of the surface, 0 % of the centre |
| UI | 5 screen change(s), 5 with motion; 288 ticks of UI motion; 2 press(es), 2 with a ui-bus sound |
| Inputs | 2, 2 answered, median 0 ticks, max 0; 2 with a sound; 54.26 % of input-free ticks also change something |
| Damage | 0 moment(s): hit stop 0, shake 0, floating text 0, sound 0, tween 0, particles 0 |
| Idle | 0 % of ticks |
| Particles | peak 12 system(s), 1 burst(s) |
| Frame (metal) | step 6.45 ms median, 14.85 p95; render 9.28 median, 12.65 p95; frame p95 26.19, max 34.98 |
| Audio | 8.7 s, peak 0.23, RMS 0.0371, 0 clipped, 0 % silent ticks |

### pause

Escape pauses, settings switch the language and save, Resume gives the run back (312 ticks)

| Measure | Value |
|---|---|
| Text boxes | 20; 20 at 4.5:1; 20 of 20 at 9 px ink |
| Lowest contrast | 5.04:1 `settings` (pause); 5.21:1 `Label` (settings-zh); 5.33:1 `Label` (settings-zh) |
| Smallest ink | 10 px `resume` (pause); 11 px `Value` (settings-zh); 12 px `quit` (pause) |
| HUD | 1 gameplay frame(s), up to 1.42 % of the surface, 0 % of the centre |
| UI | 8 screen change(s), 8 with motion; 297 ticks of UI motion; 5 press(es), 5 with a ui-bus sound |
| Inputs | 7, 7 answered, median 0 ticks, max 1; 6 with a sound; 89.62 % of input-free ticks also change something |
| Damage | 0 moment(s): hit stop 0, shake 0, floating text 0, sound 0, tween 0, particles 0 |
| Idle | 0 % of ticks |
| Particles | peak 12 system(s), 0 burst(s) |
| Frame (metal) | step 6.15 ms median, 36.69 p95; render 8.82 median, 36.48 p95; frame p95 61.16, max 1801.23 |
| Audio | 5.2 s, peak 0.42, RMS 0.0363, 0 clipped, 0 % silent ticks |

![pause pause](frames/pause-pause.png) 

pause (tick 105, menu: HUD 5.81 %, 4 texts, 0 under 4.5:1, 0 small)

### phone

the phone layout: a HUD inside the safe area, a joystick that walks, Jump and Kindle (408 ticks)

| Measure | Value |
|---|---|
| Text boxes | 7; 7 at 4.5:1; 7 of 7 at 9 px ink |
| Lowest contrast | 10.67:1 `Count` (play); 10.67:1 `Hour` (play); 10.67:1 `Toast#1` (play) |
| Smallest ink | 11 px `Toast#1` (play); 11 px `Text` (play); 12 px `Hour` (play) |
| HUD | 1 gameplay frame(s), up to 3.7 % of the surface, 0 % of the centre |
| UI | 2 screen change(s), 2 with motion; 201 ticks of UI motion; 3 press(es), 3 with a ui-bus sound |
| Inputs | 4, 4 answered, median 0 ticks, max 0; 4 with a sound; 78.99 % of input-free ticks also change something |
| Damage | 0 moment(s): hit stop 0, shake 0, floating text 0, sound 0, tween 0, particles 0 |
| Idle | 0 % of ticks |
| Particles | peak 12 system(s), 1 burst(s) |
| Frame (metal) | step 4.86 ms median, 6.31 p95; render 6.17 median, 7.91 p95; frame p95 13.34, max 2974.39 |
| Audio | 6.8 s, peak 0.32, RMS 0.0242, 0 clipped, 0 % silent ticks |

![phone play](frames/phone-play.png) 

play (tick 405: HUD 3.7 %, 7 texts, 0 under 4.5:1, 0 small)

### replay

a run with movement, a jump, kindling and a sentinel's grab replays to the same hash (486 ticks)

| Measure | Value |
|---|---|
| Text boxes | 0; 0 at 4.5:1; 0 of 0 at 9 px ink |
| HUD | 0 gameplay frame(s), up to 0 % of the surface, 0 % of the centre |
| UI | 4 screen change(s), 4 with motion; 193 ticks of UI motion; 1 press(es), 1 with a ui-bus sound |
| Inputs | 5, 5 answered, median 0 ticks, max 0; 3 with a sound; 60.94 % of input-free ticks also change something |
| Damage | 0 moment(s): hit stop 0, shake 0, floating text 0, sound 0, tween 0, particles 0 |
| Idle | 0 % of ticks |
| Particles | peak 12 system(s), 1 burst(s) |
| Frame (metal) | step 4.52 ms median, 6.81 p95; render 6.45 median, 7.84 p95; frame p95 14.62, max 20.73 |
| Audio | 8.1 s, peak 0.32, RMS 0.0277, 0 clipped, 0 % silent ticks |

### restart

a sentinel chasing when the run restarts is back on patrol (222 ticks)

| Measure | Value |
|---|---|
| Text boxes | 0; 0 at 4.5:1; 0 of 0 at 9 px ink |
| HUD | 0 gameplay frame(s), up to 0 % of the surface, 0 % of the centre |
| UI | 1 screen change(s), 1 with motion; 34 ticks of UI motion; 0 press(es), 0 with a ui-bus sound |
| Inputs | 0, 0 answered, median 0 ticks, max 0; 0 with a sound; 31.13 % of input-free ticks also change something |
| Damage | 0 moment(s): hit stop 0, shake 0, floating text 0, sound 0, tween 0, particles 0 |
| Idle | 0 % of ticks |
| Particles | peak 12 system(s), 0 burst(s) |
| Frame (metal) | step 4.47 ms median, 11.91 p95; render 6.35 median, 7.91 p95; frame p95 19.65, max 23.79 |
| Audio | 3.7 s, peak 0.07, RMS 0.0123, 0 clipped, 0 % silent ticks |

### sentinels

a sentinel on its round sees the hero, raises the alarm, chases and grabs a heart (462 ticks)

| Measure | Value |
|---|---|
| Text boxes | 3; 3 at 4.5:1; 3 of 3 at 9 px ink |
| Lowest contrast | 7.54:1 `Pause` (alarm); 10.14:1 `Count` (alarm); 10.53:1 `Hour` (alarm) |
| Smallest ink | 12 px `Hour` (alarm); 15 px `Count` (alarm); 38 px `Pause` (alarm) |
| HUD | 1 gameplay frame(s), up to 0.97 % of the surface, 0 % of the centre |
| UI | 1 screen change(s), 1 with motion; 34 ticks of UI motion; 0 press(es), 0 with a ui-bus sound |
| Inputs | 0, 0 answered, median 0 ticks, max 0; 0 with a sound; 29.2 % of input-free ticks also change something |
| Damage | 0 moment(s): hit stop 0, shake 0, floating text 0, sound 0, tween 0, particles 0 |
| Idle | 0 % of ticks |
| Particles | peak 12 system(s), 0 burst(s) |
| Frame (metal) | step 4.35 ms median, 6.98 p95; render 6.2 median, 7.42 p95; frame p95 14.11, max 1091.57 |
| Audio | 7.7 s, peak 0.26, RMS 0.0184, 0 clipped, 0 % silent ticks |

![sentinels alarm](frames/sentinels-alarm.png) 

alarm (tick 195: HUD 0.97 %, 3 texts, 0 under 4.5:1, 0 small)

### win

four beacons lit: dawn, a merged record, the summary, then Again from nothing (263 ticks)

| Measure | Value |
|---|---|
| Text boxes | 0; 0 at 4.5:1; 0 of 0 at 9 px ink |
| HUD | 0 gameplay frame(s), up to 0 % of the surface, 0 % of the centre |
| UI | 5 screen change(s), 5 with motion; 250 ticks of UI motion; 1 press(es), 1 with a ui-bus sound |
| Inputs | 1, 1 answered, median 0 ticks, max 0; 1 with a sound; 97.32 % of input-free ticks also change something |
| Damage | 0 moment(s): hit stop 0, shake 0, floating text 0, sound 0, tween 0, particles 0 |
| Idle | 0 % of ticks |
| Particles | peak 12 system(s), 4 burst(s) |
| Frame (metal) | step 4.82 ms median, 11.57 p95; render 7.23 median, 8.97 p95; frame p95 19.33, max 25.3 |
| Audio | 4.4 s, peak 0.44, RMS 0.0615, 0 clipped, 0 % silent ticks |

