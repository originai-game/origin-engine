<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `video-to-game` guide: From gameplay video to a playable game

Use one or several gameplay videos as the design document for an Origin Engine game: study the shots, reproduce the art and core action, then prove the result with input-driven tests, side-by-side frames and a self-play recording. A moving scene is not yet a playable game.

The tools: `og video study --help`, `og compare --help`, `og render --help`; `og api show camera`, `og api show anim`, `og api show assert`, `og api show commands` and `og api show probe`. Asset requests follow `art-review §2`, looks `looks §1–3`, pose queries `animation §3` and `§7–8`, captures `captures §1–4`, tests `testing §2–3`, performance `testing §6` and `perf §2–4`, and scatters `world §3`.

<a id="guide-video-to-game-1"></a>
### `video-to-game §1` Keep the design evidence

Save the source videos, study frames and JSON, `SPEC.md`, generation prompts and seeds, the game seed, asset originals and their sources, licences and generation receipts. Keep them in a reference directory beside `assets/`, not inside it, and do not import them into the game. They are creation evidence, not player downloads. Inspect `og build --publish --json`'s file list and the final package to confirm none leaked in; packaging the project root is not packaging the game. Keep temporary builds outside the project rather than accumulating copies in the evidence directory.

Record each generation's id and the cost reported for that id, net of any refund, and total a run's cost from those records. An account-wide total is shared with other work and may lag a refund, so diffing it before and after is not evidence of one run's cost. A failed generation is not free until its own cost says so: read it before retrying.

For third-party footage, mark the gameplay ranges first. Exclude watermarks, badges, end cards and the original audio from the design and the generated assets; keep the untouched source as evidence. A cut candidate can locate an end card, but inspect it before excluding a range. Use the gameplay segment's palette, not one contaminated by a title card. Generate the game's own audio through the platform's generation services.

When there is no source yet, ask a video model through the platform's generation services for:

- One original action verb the player controls, not a list of genres.
- Its visible consequence: what changes when the action succeeds or fails.
- A stable, stated camera that keeps the actor and the consequence in view.
- An explicit art direction: shapes, materials, palette and light.
- Gameplay only, without a fake HUD, captions or promotional cuts.

For example: “A courier hooks a windline to swing between flying creatures; releasing over a back lands and delivers a parcel, missing it means falling. A stable three-quarter camera shows the courier, line and destination. Cool blue clouds, dark cloth, satin skin tones and warm gold action accents. No HUD or text.” Study what the video actually shows, not what this prompt intended: another generation of the same prompt is another design.

<a id="guide-video-to-game-2"></a>
### `video-to-game §2` Study every clip before building

Run `og video study` on each source. Read the contact sheet, then open full-size frames around the action, camera changes and cut candidates. Use `--at` for repeatable moments: for a ten-second gameplay clip, start with `--at 2,5,8`. For shorter clips or excluded tails, choose three valid times spanning the opening, core action and destination, and freeze that selection before tuning.

Use the real presentation timestamps (PTS) in the study output, not frame number divided by nominal fps. Record both the requested time and the selected frame's time when they differ. Keep that citation when cropping out an overlay or normalising a frame's size.

Write one spec, with a clip and timestamp on every observed design fact, such as “A 1.25 s” or “B 4.5–6.0 s”:

- Hero identity and silhouette, scale, materials, facing and action poses.
- Camera position relative to the hero, lens, framing and transitions.
- The core verb: input, trigger, consequence, feedback and recovery; key moments in order.
- Traversable surfaces, route landmarks, hazards, destination and depth layers.
- Palette, light direction, atmosphere and the details that carry the style.

Separate observations from inferences and additions. A clip cannot prove its control scheme, gravity or failure rules. Choose playable rules for what it omits, label them as author decisions and name the test that will check them. A short action needs a repeatable loop, a failure and a restart, not just a scripted camera ride.

For multiple clips, add a decision table before implementation. Each row names a conflict or connection, the evidence from each clip, the chosen rule and why. Resolve hero identity, geography, shot order, bends versus straights, light direction and time of day. Two clips are one world only once the spec says how their places join, such as “one road, ridden toward the sun in both clips”; check that join against the frames. A cast shadow on a study frame gives the sun's direction relative to travel: ride every stretch a clip shows the way that clip rides it, and route the stretches no clip shows where their light matters least. A ridge clip looking down on a village road can justify joining them into one route; a return climb unseen in either clip is an addition, not an observed fact. Do not average incompatible cameras or discard one clip silently.

<a id="guide-video-to-game-3"></a>
### `video-to-game §3` Measure the shot, then set the camera

Before tuning, measure the hero's pixel height, foot or vehicle-base y, and horizon y on each selected source frame, then what fixes the world around them: the path's width at the hero, the drop to what lies below and the angle a framing structure such as an arch or ring spans. Record the frame dimensions and the measurements in the spec; normalise by image height when resolutions differ. Mark a hidden foot or horizon as unknown rather than guessing. Also note facing, screen side and where the action's target lies.

Match these on rendered game frames at the same aspect ratio. As an initial framing tolerance, keep hero height within 10% of the source height and base and horizon within 3% of frame height; record a reason for any deliberate departure. These are author review targets, not engine pass scores. A matching height with a clipped head or foot fails. Review the whole action and camera transitions as well as the three stills: a camera can match them and lose the hero between them.

Measure the camera the shot script actually uses, not one remembered or guessed; a structure placed for an earlier camera frames nothing. When hero size and horizon already match and the frame still reads wrong, measure the land before moving the camera again: a path too wide or a landmark too far changes the shot as much as the lens. A camera pitched down sees little above the horizon, so a ring, arch or canopy meant to frame the view has to sit inside that vertical reach.

Fix framing before fine grading or asset detail. Check `facing §1–2` before correcting a model with a half-turn. Keep the hero, interaction point and next landing or threat readable together. A hook is not necessarily the eventual landing point; aim and feedback must show where the player can really arrive. Move the camera around obstacles, not through the actor, and do not hide broken geometry with a convenient angle.

<a id="guide-video-to-game-4"></a>
### `video-to-game §4` Transfer the art, preflight the assets

Choose the nearest look and put its overrides in the game's look file (`looks §1–3`). Request assets through `artBrief` with the study frames as references (`art-review §2`). Put hero identity in the subject, not just the style reference. Accept a concept image against the source before using it for a model request through the platform's generation services; inspect the model again, since a good concept does not guarantee its proportions or colours survive.

Before building the level around an asset, preflight it in a small rendered scene:

- **Pose and proportions:** inspect the evaluated rest pose and action poses, with the rig and instance scales applied. Raw mesh bounds alone do not establish a skinned character's world size.
- **Facing:** inspect front, side and back. Identify the actual head, tail and travel direction; an axis convention cannot identify a generated model's anatomy.
- **Root motion:** play each clip through its full cycle and measure travel. Decide whether the controller or the clip owns movement; do not let both move the actor.
- **Contact:** test the real standable surface, foot placement and hand or wheel contacts. Check on moving platforms and through blends; use `animation §3` and `§7–8` for pose timing and coordinate frames, not guessed joint indices.
- **Surface quality:** inspect the turntable and action in the intended lighting for seams, flipped faces, broken highlights and deformation. Fix the asset and re-capture; a colour tint cannot repair geometry.
- **Size, levels and cost:** read its bounds, origin, triangles and levels of detail. `og export gltf <asset> --lod N` writes one cooked level to inspect, and the `debug:lod` view shows which level draws. Fit a replacement to the asset it replaces, the same height with its base at the origin, so the swap changes one thing. A model's levels are not an impostor: a wood belongs in og.TerrainScatter with an impostor, whose `scatterIssue` reports a model that does not fit (`world §3`).
- **Texture colour:** measure the texture's own colour before tinting. A colour that seems to come from the light may be the base colour; change the texels rather than stacking tints.

Judge shape before colour. A generated model is a closed shell: light does not pass through a generated crown, so it casts a solid shadow and lets no dapple or shaft through. Compare its silhouette and scale with the clip first; from a low camera a copy's size and outline show more than the number of copies, and no colour makes the wrong shape read as the clip's.

Grade with numbers, not by eye, which drifts after a few passes. Pick a few named surfaces, such as the path in shade and in sun, the hero and the ground, check each sample region as a crop before trusting its number, and compare Oklab L, C and h between source and game frames at the mapped times. To measure a material change, render the same frame before and after and read only the pixels that differ. Albedo moves lightness; a shaded-to-lit ratio that is off is lighting, so record it as a lighting gap rather than darkening the material until the shade matches.

Use `og api show og.load.gltfInfo`, `og api show og.render.geometryInfo` and `og api show og.render.raycast` for the measurements and their limits. Follow `art-review §3–5` for conformance and model acceptance. If generated art exceeds project budgets, reduce it or justify a measured budget change in the spec, rather than disabling checks. Preserve originals and make derived assets reproducible; label an adapted action as adapted, not as a newly authored clip.

<a id="guide-video-to-game-5"></a>
### `video-to-game §5` One playable route, reused

Write a lap or complete action route in TypeScript that produces an ordered real-input command script. A planner may use the game's movement rules to choose inputs; the acceptance run must still play those inputs through the engine. Export the script as a module the playtests can import. Reuse it, with the same seed, for `og test`, camera captures and `og render`; do not maintain three slightly different routes.

Record the event ticks and route landmarks, then map each source shot to a game time and tick on that route. For a synchronous recreation, align the action's start and take the source and game frames at the same elapsed times. If several clips show different route sections, state each section's offset. Label action-phase comparisons when timings differ; never present them as time-synchronised evidence.

Change presentation without changing simulation to make a shot. If physics, contacts or the route change, rerun the full route and its failure tests before reusing old evidence. Initialise the hero's visible pose and placement before the first tick as well as updating them during play: inspect frame zero. A successful lap is not a test of every branch; check at least one alternative separately and state what remains unplayed.

<a id="guide-video-to-game-6"></a>
### `video-to-game §6` Three independent acceptance checks

All three must pass; one cannot stand in for another.

- **Mechanics:** run `og test` with real key, pointer or gamepad inputs (`testing §2, §7`). Put every core phase and the loop's completion in `require`; inspect the probe's `timeline` for their order and ticks (`captures §3`). Check observable progress and contacts, deliberate failure, recovery and restart, and identical hashes for identical seed and input. A required event must represent what happened, not merely the press that requested it. No teleporting to the goal or setting a success flag. Read uncovered and unchecked results; a renderer-free run proves no frame.
- **Frames:** extract the frozen source times with `og video study --at` and capture the mapped game times from the shared route. Run `og compare` on every pair and inspect every side-by-side sheet and relevant detail (`captures §4`, `art-review §4–5`). Review all clips, not just the easiest one. Judge hero size and identity, camera, contact, target visibility, world structure, light and palette against the spec. Whole-frame difference numbers are not an aesthetic pass threshold. A familiar theme with the wrong shot or missing landmark is still a mismatch.
- **Performance:** measure timings on the target GPU at the target resolution over the route (`testing §6`), separately from screenshot readbacks. Judge GPU numbers only on a hardware GPU; a CPU renderer's timings are a rough relative guide, read by their medians. Run the same build twice first to learn the run-to-run spread and treat a difference inside it as none. Hold the frame to `frame interval`, at its median and p95, not to `gpu frame` (`perf §2–3`): a frame can be cheap on the GPU and bound on the CPU, so read `cpu render build` and each hitch's cause (`perf §4`). Find a cost by removing one content group a run over the same route, such as a wood, a band of trees or one group's shadows, not by guessing; a scatter's density map does not reduce its placement (`world §3`). Record hardware, backend, engine and game identity, CPU tick and GPU p50/p95, frame intervals, cold start and worst stalls. Include the first use of each core effect. A fast GPU percentile alone does not prove stable real-time fps; a recording encoded at 30 or 60 fps proves none of it. Remeasure after changing the scene or look.

Exercise the intended player build and host too. A successful build, a Play button or input script exit code is not proof that a browser displayed a playable game. Inspect actual output, play the loop, fail and restart there. Report a blocked host separately from a passed native run, and report an engine defect at its source rather than patching around it in the game.

When GPU time is shared or scarce, prepare everything before taking it: the build, the route, every shot time and every perf variant, run as one batch. Copy each output off the host as its job ends, since a stopped host can take its files with it; encode a recording on the host before moving it, and release the host when the batch is done.

<a id="guide-video-to-game-7"></a>
### `video-to-game §7` Record, audit and hand over

Record the shared input route with `og render`, including the game's own audio. Use the timeline to find a segment containing the complete verb and its consequence. Study the recording for candidate windows; motion amount and best-window activity only nominate candidates. Inspect for loading, falls, walls, occlusion, clipped limbs and losing the hero. The most active window may be the worst demonstration.

Use `--keyframes` for a sparse event script (`captures §3`). Dense steering pulses can retain nearly every frame, so sample the encoded movie instead of writing thousands of keyframe PNGs. A contact sheet is an overview, not an audit of the frames it omits: inspect every frame of the final short proof clip, plus watch and listen to it in motion.

Check the actual delivered file, not just the render report. Run `og video study` on it and use its decoded frame count, real PTS and packet-end duration to check for truncation against the requested span and fps. Use ffmpeg only for what study does not measure: audio duration, silence intervals (`silencedetect`) and per-frame brightness. Compare the audio duration with the video's; inspect frames flagged as black or unexpectedly dim, since a deliberately dark shot is not automatically broken. Listen to flagged silence intervals, especially action and landing cues: an audio stream can exist yet be silent. After trimming, repeat these checks on the delivered file.

Hand over the playable build, spec, seed and reusable input route, test results and event timeline, labelled side-by-side frames, audited self-play clip and target-GPU timings. Tie every result to the same final game and asset version; verify transferred source and cooked files agree so an old source cannot silently rebuild over new cooked art. Keep creation evidence separate from the game package (§1). List remaining fidelity gaps, untested branches and blocked hosts instead of calling them complete.

This workflow adapts the process of [video-to-game](https://github.com/lovisdotio/video-to-game) (MIT); the game itself uses Origin Engine's ordinary project and tools, not a gameplay template.
