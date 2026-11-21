<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `perf` guide: Perf: what one run says about a frame's cost

How to find out what makes a game slow from a single run, without a second run to compare against: the per-frame record, its report, the budgets that flag a line, and how to read a hitch. Read before tuning a scene's cost or chasing a stutter.

Run options and their help: `og run --help` (`--perf`, `--budget`).

<a id="guide-perf-1"></a>
### `perf §1` Recording a run

```sh
og run . --headless --size 2560x1440 --ticks 1200 --perf og-out/perf
```

`--perf <dir>` records one row per frame and, when the run ends,
writes `perf.txt` (for a person) and `perf.json` (for a tool) to `<dir>`
and prints the text. Each frame draws once per tick, as a traced run's do,
so the record holds every frame of the run. A run of a few thousand
frames keeps them all; a longer one keeps the last 8192.

Hold the camera where the cost is (a `--commands` step that holds it, or
the game's own debug switch) and let the first seconds after loading
pass before the frames you care about: they are in the record too, and
the report names them as hitches with their first-use cause.

The record costs a few counters per frame, so the numbers are the
game's. It is the engine's only record of frame times: `og run --json`'s
frame figures, `og run --device`'s, a desktop package's exit report and
field telemetry's `frames` are read from it, so they count the same
frames and the same hitches. A published page with field telemetry on
records its play through it.

<a id="guide-perf-2"></a>
### `perf §2` The report's lines

Each line is one cost of the frame, in milliseconds, with its mean,
median, 95th and 99th percentiles and maximum over the run, and its share of the median
frame interval:

| line | what it measures |
|---|---|
| `frame interval` | time from one drawn frame to the next: a presenting host's display frames (the web page's, the desktop window's), else the engine's clock between frames |
| `frame work` | the engine's time on the frame: its ticks and its render submission |
| `gpu frame` | the GPU's time on the frame, first pass to last |
| `gpu <pass>` | one render pass's time alone on the GPU, when no other pass ran |
| `cpu ticks` | the ticks run before the frame, all their systems |
| `cpu physics` | physics systems inside them |
| `cpu ui` | the UI's systems inside them |
| `cpu game systems` | the game's own systems inside them |
| `cpu engine systems` | the engine's other systems inside them |
| `cpu render build` | building the frame: extracting the world, preparing, submitting |
| `cpu render encode` | encoding the render passes' commands |
| `cpu blocking gpu waits` | time the frame stood still waiting for the GPU |
| `cpu pipelines made` | time spent making pipelines on the frame's thread |
| `upload kb`, `uploads` | data written to the GPU in the frame, and the writes |
| `buffers made`, `buffer kb made` | GPU buffers created in the frame |
| `readbacks` | reads of GPU data back to the CPU started in the frame |
| `blocking gpu waits`, `pipelines made` | how many of each the frame did |

After the lines, the report lists where that data moved, each place with
its calls, total MB, KB per frame and the frames it moved in: uploads in
steady play and uploads only at load, buffers made, every readback, every
blocking wait, and the pipelines made after the first frames. In steady
play a frame should make no buffers and no pipelines; an upload that
repeats every frame with the same size is data that could stay on the
GPU.

GPU passes that ran together each count only their time alone, so the
pass lines plus the overlap add up to `gpu frame`. A pass the GPU could
not time is left out of a frame, never counted as 0.

The time passes spend overlapped belongs to no one pass, so one frame in
30 runs serially: each pass waits for the one before it, and the main and
prepass passes are timed per draw class (opaque, meshlets, skinned,
terrain, scatter, grass, sky, water and blended, as "main: terrain"). The
table `gpu cost per pass, alone` lists those frames' costs, largest first,
with their sum next to the real frame's span (JSON `costs`). That is what
each pass costs by itself; the sum is usually less than the real frame,
since passes that overlap slow each other down. The serial frames feed only
that table, never the other lines. `og run --gpu-serial` runs every frame
serially.

A frame is GPU-bound when `gpu frame` is near `frame interval`, and
CPU-bound when `cpu ticks` plus the render lines are. Start with the
larger side's largest line.

<a id="guide-perf-3"></a>
### `perf §3` Budgets

A budget is the most milliseconds a line's median may take. Put them in
the project, so every run is flagged the same way:

```
perf {
  budget "frame interval" 16.6
  budget "gpu frame" 14
  budget "cpu ticks" 4
}
```

`--budget "<line>=<ms>"` overrides one for a run. A line over its budget
reads `OVER` in the text and `"over": true` in the JSON. Lines without a
budget are reported, never flagged.

Starting values for a 60 Hz desktop game: `frame interval` 16.6,
`gpu frame` 14, `cpu ticks` 4, `cpu render build` 4,
`cpu render encode` 2. For a 30 Hz phone game, double them. A single
pass rarely needs more than a quarter of `gpu frame`; the main lit pass
may take half.

<a id="guide-perf-4"></a>
### `perf §4` Hitches

A hitch is a frame whose interval is more than twice the median of the
64 frames before it (none in the first 8), so a game drawn at a steady 30
has no hitch at 50 ms and one at 60 Hz does. One of 250 ms or more is a
long hitch (field telemetry counts both). Each has a cause:

- `pipeline compile N ms`: the frame made a pipeline it met for the first
  time. Warm it while loading.
- `<line> +N ms`: the line that grew most over its own median in that
  frame. For the first frames after loading or Play the medians are the
  steady frames', so a first build or a first use reads as the line it
  landed on.

Either cause is followed by the first-use work the frame did, when it
did any: pipelines made, buffers made, blocking waits, and an upload of
1 MB or more. The first frame after loading or Play reads, for example,
`pipeline compile 36 ms; 102 pipelines made, 385 buffers made (562003 kb),
257509 kb uploaded`.

A hitch whose cause is `cpu ticks` with several ticks is the simulation
catching up after a slow frame: fix the slow frame before it.
