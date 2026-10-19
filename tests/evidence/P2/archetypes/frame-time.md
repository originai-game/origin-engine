# Archetypes round: frame time and the machine's load

`og polish` scores frame time by the worst playtest's p95 of step + render
(8.3 ms is 100, 33 ms is 0). This round ran on an M4 Pro shared with other
agents' builds and gates, at load averages of 15 to 50, so a p95 measures
the scheduler as much as the game. Two things were done about it:

1. **og built release.** The gate builds og and the native addon in the
   dev profile (`packages/engine/native/build-profile` says `dev`), where
   serde_json and the ECS run unoptimised. The polish reports in this
   directory were made with og and `libog_host_node` built from this
   checkout with `cargo build --release -p og-host-node -p og-cli` (into a
   scratch target directory, nothing in the tree changed), passed as
   `OG_ENGINE_PACKAGE=packages/engine OG_NATIVE_ADDON=<release addon>
   <release og> polish .`, which is what a Maker's installed og is.
2. **A per-tick series under load**, to see where the p95 comes from.
   `roguelike-pulse-under-load.json` is `dump_pulse` after every tick of
   Lamp Dive walking up seed 7's first corridor (a step every 15 ticks
   with the foes' turns, 240 ticks), release build, load average 29–35.

   | | value |
   |---|---|
   | median step + render | 2.41 ms (step 1.49, render 0.64) |
   | p95 step + render | 50.3 ms |
   | ticks over 16.7 ms | 26 of 240 |

   The 26 slow ticks (72, 73, 82, 93, 98, 105, 119, …) do not follow the
   turns (a step, the foes' commands and the fog's update land every 15
   ticks); several are render spikes of 75–200 ms on ticks whose step took
   under 1 ms, which is the process waiting for a core, not the game's
   work.

The same release build gave, minutes apart:

| Archetype | Load average | Frame p95, worst playtest | Feel |
|---|---|---|---|
| roguelike | 30–40 | 43 ms | 80 |
| roguelike | 15–18 | 6.8 ms | 100 |
| tower-defense | 16–23 | 2.8 ms (first five playtests), 93 ms (phone, as the load rose to 23) | 80 |
| tower-defense | 25–26 | 33 ms (frame time 48) | 89 |

Each archetype's README states the scores of its committed report and,
where frame time held feel under 90, that this is why.
