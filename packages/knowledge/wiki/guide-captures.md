<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `captures` guide: Captures, marks, key frames and frame diffs

How to look at a running game in as few tokens as possible: frames fitted for a model, crops at full resolution, numbered boxes that point back to source (Set-of-Mark), the key frames of a run with a timeline synced to ticks, and perceptual diff heat maps with the worst regions enlarged. The engine reports measurements and regions; what matters is the reader's call.

The API of everything here: `og api show probe` (`capture`, `marks`, `timeline`, `pickUi`, `describe`), `og render --help`, `og compare --help`.

<a id="guide-captures-1"></a>
### `captures §1` A frame for a model

`capture` fits its PNG to a long side of 1280 (`CAPTURE_FIT`) by default: the whole frame a model reads at once. Each pixel of the fitted image is the mean of the area it covers, so thin text and edges stay legible.

| Want | Call |
|---|---|
| The whole frame, model-sized | `og probe capture` |
| The rendered size (a reference, a pixel count) | `og probe capture '{"fit": 0}'` |
| A detail at full resolution, as its own image | `og probe capture '{"region": {"x": 600, "y": 20, "width": 200, "height": 120}}'` |

A requested size renders with the same camera, its aspect following the size. On the web the readback is asynchronous: a capture requested on tick N is answered on N + 1, and a synchronous in-page call answers null on the first tick and the frame from then on.

`region` is a box of the play surface in CSS px (the units of `pickUi`, `describe` and `touch_move_to`). The answer's `scale` is the PNG's pixels per CSS pixel and `region` the box it shows: a PNG pixel (px, py) is the surface point (region.x + px / scale, region.y + py / scale). Look at the whole frame first, then cut the region you need instead of asking for a larger whole frame.

<a id="guide-captures-2"></a>
### `captures §2` Set-of-Mark: numbered boxes with their source

`marks` draws a numbered box over each UI element (amber) and each scene entity (cyan) the frame shows, and answers what each number is:

| Field | What |
|---|---|
| `n` | The number on the image |
| `kind` | `ui` or `entity` |
| `path` | The element's full path (as `dump_ui` and `touch_move_to` name it), or the entity's probe id |
| `what`, `text` | The element's kind and text; what draws the entity and its name |
| `rect` | The box in the PNG's pixels |
| `point` | A surface point on it, for `pick`, `pickUi` or `touch_move_to` |
| `source` | The document line that built the element, or the scene line that declares the entity |
| `part`, `rule` | The style part an element is drawn as, and the rule that dresses it, with its file and line |
| `depth` | An entity's distance from the camera |

UI marks come first in reading order, then entities nearest first. A layout container that draws nothing gets no mark, a label inside a button of the same box is marked once (as the button), and a backdrop covering the whole surface is left out. `max` (64) caps the marks; `omitted` counts the rest. `ui: false` or `entities: false` marks only one kind.

An entity's `what` names what draws it. A grass field is seen at field fidelity, not per blade: its coverage is the ground it grows on lifted half its tallest blade, so a pixel between blades answers the field and a tip above that height answers what is behind it.

A review then says "7 is cut off" and the map says `ui/shop.svelte:42`, `Button.base[primary]` from `styles/clean/style.ogt:210`: the line to change.

<a id="guide-captures-3"></a>
### `captures §3` Key frames of a run

`og render --keyframes` records a scripted run and keeps the frames where something happened:

- the first and the last frame (`start`, `end`), and the first of each `--jobs` part (`part`);
- each frame after an event of the probe's `timeline`: an input, a command step, a screen change, an assertion firing or violation, an expectation;
- each frame whose mean FLIP from the last key frame is at least `--keyframe-change` (0.02), when it is at least `--keyframe-gap` seconds (0.5) after it (`visual`, with the measured `change`).

It writes `keyframes/<index>.png` (fitted to 1280), `keyframes.png` (all of them on one labelled sheet) and `timeline.json`: each key frame with its tick, time, change and reasons, and every event with its tick and time. Read the sheet first, then open the key frames the sheet points to. A 4 s drive at 30 fps keeps about 12 of 120 frames.

The probe's `timeline` member answers the same events to any tool that steps a run itself: each with `tick`, `kind` (`input`, `step`, `screen`, `assertion`, `expect`), a one-line `what` and its `detail`. A read takes them.

<a id="guide-captures-4"></a>
### `captures §4` Perceptual diffs

`og compare <reference.png> <render.png>` measures NVIDIA's FLIP (the LDR-FLIP of the reference implementation) of the two besides the silhouette IoU and ΔE00: mean, p50, p95, p99 and max per pixel, in [0, 1]. It writes the heat map (`<render>.flip.png`, magma: black is no difference) and the `--regions` (4) worst regions, one per change, each grown to the change's extent and written as reference | render | heat enlarged to about 256 px a panel (`<render>.flip-<n>.png`). Read the strips, not the whole heat map.

`--ppd` sets the observer's pixels per degree (67.02: 0.7 m from a 0.7 m wide 4K monitor). Fewer pixels per degree forgives more fine detail.

A gfx reference frame that fails its test writes the same heat map and its three worst regions beside `.actual.png` and `.diff.png`, and prints FLIP's numbers with each region's box.

<a id="guide-captures-5"></a>
### `captures §5` Sizing for a model

| Image | Size |
|---|---|
| A whole frame | Long side 1280 (`capture`, `marks`, key frames) |
| A detail | A `region` at full resolution, as its own image |
| A diff | The worst-region strips, about 256 px a panel |
| A run | `keyframes.png`, four to a row |

Send the smallest image that answers the question. A crop beats a larger whole frame; one sheet beats many frames.
