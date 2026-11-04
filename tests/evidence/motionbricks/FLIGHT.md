# Flight operator: root alignment experiment

This is a CPU experiment, not a shipping operator or an accepted model.
`crates/og-runtime/examples/flight_plan.rs` restores the archived prototype and
adds the coordinator-approved root alignment. The shipping model pin, clamp,
certification set, contact bounds and pelvis slack are unchanged. There are
**zero original-library training jump windows**; the historical library/train
fixture predates ground-v2 and does not establish training eligibility.

## Inputs and reproduction

OVH bucket `origin-generated-assets`, prefix
`training/motion-trajectory/learning-curve-v2/` holds the immutable originals.
The receipt `archive.json` has SHA256
`31ac8f28cf3a4e5e03a9dc1a7f56832cc013def77919817e8f3822f0d9aed564`.
Verify each input against the receipt before running. The original
`handoff/flight_plan.rs` has SHA256
`93a923efde26fc1ae362c6e58b25a6b047c236e8a9d2a5de53fa115957a81ae0`;
`handoff/results-v6.json` and `handoff/results-main-ca0de12.json` retain the
raw-root operator results. Do not replace those baselines. Preserve both plan
files and their unsupported groups. This experiment uses `recordings-v2.json`;
its recorded source FK supplies the root stand-in without rerunning a model.

With the verified `handoff/` files in `$HANDOFF`:

```sh
cargo test -p og-runtime --example flight_plan
cargo run -p og-runtime --example flight_plan -- \
  "$HANDOFF/recordings-v2.json" "$HANDOFF/model.safetensors" \
  "$HANDOFF/traveller.glb" "$HANDOFF/meshy.ogt" > flight-aligned.json
```

This runs no GPU and does not use certification. The inherited spring port
(Daniel Holden's Spring-It-On, revision
`4d97b497a78e40c7b47d49e8a9da27e4aa5616d6`, MIT) is withdrawn with the
whole-clip solve below: nothing in the example calls it any more, and
`third_party/spring-it-on/` is removed with it.

## Root contract (approved 2026-11-04)

Production FlightPlan must carry **tick-owned controller position and yaw at
each 30 Hz sample**. Physics and world gravity own travel; no generated pose or
contact flag writes simulation state. The production path is not implemented
by this experiment.

The fixture stand-in is `[source hips.x, 0, source hips.z]` and source yaw
(the same upper-leg heading convention as the runtime). It is explicitly
ground-projected, not a source controller-elevation track. Generated pelvis Y
is retained. The traveller's root stand-in follows the existing retargeter's
rest-relative translation scale; its Y is also projected to zero.

Before torso/contact IK, one rigid transform re-roots the generated body onto
the planned horizontal position and yaw. Only the hips' local transform changes;
descendants retain their internal pose and lengths. Rotation corrections are
unwrapped, and translation/yaw offsets use cubic Hermite interpolation with
shared central-difference tangents at every sample, one-sided tangents at the
ends. Evaluation uses absolute sample age, never accumulated render time or a
phase reset. The correction is C1 across takeoff, apex and landing. This does
**not** prove a discrete joint-step bound or make the underlying generated pose
C1. A production streaming boundary must retain adjacent samples/tangents;
restarting this curve for each model batch is not the contract.

The only correction claimed here is horizontal/yaw drift. Pelvis slack is
still −25%/+10% of leg length, measured from the aligned pose. Unreachable
contacts remain errors. `base` and `landing_step_bound_m` retain the original
raw-root pose's measurements; `aligned_base` is diagnostic, not a replacement
acceptance bound. `corrected` measures the complete operator.

Each case also carries `model_root_drift_soma`: horizontal distance, signed
shortest yaw difference, and signed generated-minus-source hips Y at **every
sample**, plus the maximum absolute vertical/yaw error and maximum horizontal
error with sample/time. These are model-quality diagnostics for pilot 6, never
operator acceptance. SOMA-space model diagnostics are repeated on the traveller
case, not mislabelled as retargeting error. Source reference Y is retained in
these diagnostics even though the operator's root stand-in uses Y=0.

Integrated operator world-hash replay, gesture priorities, leg limits and
macrunner/art-director review remain open; CPU contact and landing acceptance
is in §Whole-clip solve below. The
example's inherited `world_replay` exercise does not integrate the palette and
must not be presented as integrated-operator acceptance. No new training or
generation spend is authorized.

## CPU rerun on engine base 9bb81cb41 (per-sample pipeline, superseded)

This section records the per-sample pipeline (FABRIK torso solve, spring
inertialization, torso reprojection, contact plant) that the whole-clip solve
of the next section replaces. Its numbers are the "before" column there.

All 20 cases were rerun after alignment. Raw-root `base` measurements and
landing-step bounds match the archived results exactly. Running/adaptation's
contact maximum drops from 23.225/14.366 cm (SOMA/traveller) to below 0.0001 cm
on both rigs. Hopping/train remains unreachable at 3.596/3.966 cm (previously
3.445/3.785 cm); it is not waived. The other 18 cases have contact error below
0.0001 cm. Ten of twenty cases still fail the original landing-step bound;
running/adaptation SOMA worsens from 33.710 to 35.857 cm against its unchanged
30.659 cm bound. All twenty still satisfy the 5° lean bound (maximum 1.48934°)
and exact reverse-seek/sparse-sample equality. **The operator is not accepted.**

The maximum horizontal model-root drift is 70.249 cm in running/adaptation.
The maximum absolute vertical drift is +44.573 cm in hopping/train, at sample
24 (0.8 s from the start of its recording). These remain model-quality metrics,
not exclusions from operator acceptance. The generated/source hips comparison
retains all samples, including contacts and source-invalid historical fixtures.

The root-placement regression fails when alignment is disabled; with alignment
enabled, placement/preserved height, C1/yaw-wrap/seek behavior and signed vertical
drift peak timing pass. Those CPU checks establish the correction contract,
not art acceptance or integrated-world replay.

## Whole-clip solve (branch `flight-acceptance`)

The per-sample pipeline could not meet the continuity bound because each
stage (torso IK, inertialization, reprojection, plant) re-moved joints the
previous stage had already placed, and a plant at the final sample had no
knowledge of the samples before it. `correct` in
`crates/og-runtime/examples/flight_plan.rs` replaces those stages with one
solve over every sample of the clip, in which the continuity bound is a hard
per-joint inequality on the step between consecutive samples, the same
priority order as the approved contract: root, bone lengths, planned
contacts, the upright cone, the step bound are projected; the generated pose
is a decaying pull. Nothing is relaxed silently: a sample where the hard
constraints cannot all hold is listed in `conflict_report` with its kind,
sample and deficit, and the solve lands at the deficit-minimising pose.

The bound into sample `t` is the raw-root landing-window bound
(`landing_step_bound_m`, samples landing±2) inside that window and
`max(bound, raw_step[t])` outside it, both scaled by 0.99 so a projected step
never sits on the bound exactly. Stages, each over the whole clip:

1. **Pelvis height.** Per-sample feasible intervals for hips Y (the slack box
   ∩ reach of every hard contact, with the knee's perpendicular offset as a
   proxy of 0.7 × bound), propagated forward and backward through the step
   relation of the hips cluster. A sample whose interval is empty is a
   conflict: its Y is the deficit minimiser (ternary search), chosen first and
   frozen; the others are picked by Gauss–Seidel toward the generated height.
2. **Legs.** Foot and knee particles per leg under exact bone lengths, the
   pins of hard contacts, the step bound and, for a free foot, a pull toward
   its planned path (generated foot, approaching the anchor before a hard
   contact, leaving it after). A free foot is never a world anchor.
3. **Rest of the body.** Position-based sweeps around the fixed hips cluster,
   knees and feet: a decaying pull to the generated pose shifted by the root
   alignment, the upright cone (4.9°, hips→head and pelvis→chest, its
   allowance ramped by the flight weight), step projection and bone-length
   projection in both orders, then a rotation refit per joint.

The output changes only the hips translation and the rotations below the
hips; every bone length is the rig's, and the example asserts that forward
kinematics of the written local poses matches the solved particles within
0.1 mm. Evaluation is at integer samples in fixed order: a second run of the
solve, read in reverse and at stride 7, is asserted bitwise equal, so seek and
sparse sampling are exact. All 20 cases run in about 8 s on one CPU core.

The rotation refit is the method of Müller, Bender, Chentanez and Macklin,
"A Robust Method to Extract the Rotational Part of Deformations" (MIG 2016);
the sweeps are position-based dynamics projections (Müller et al. 2007). Both
are algorithms reimplemented from the papers, not ported code, so no
third-party licence is carried.

### Results, engine base 9bb81cb41 on branch `flight-acceptance`

Steps in cm against the unchanged raw-root bound; "before" is the per-sample
pipeline of the previous section. Contact, lean and deviation (distance of a
corrected joint from the aligned generated pose, maximum and mean over the
clip) are after the solve.

| case | rig | bound | before | after | at | contact cm | lean ° | deviation max / mean cm |
|---|---|---|---|---|---|---|---|---|
| hopping/adaptation | SOMA | 5.155 | 9.164 FAIL | 5.103 pass | RightShin s17 | 0.0000 | 4.900 | 9.71 / 0.32 |
| hopping/adaptation | traveller | 4.633 | 8.540 FAIL | 4.586 pass | RightLeg s18 | 0.0000 | 4.900 | 12.72 / 0.80 |
| hopping/old | SOMA | 5.006 | 7.796 FAIL | 4.956 pass | RightHand s11 | 0.0000 | 4.900 | 10.31 / 0.93 |
| hopping/old | traveller | 4.034 | 6.717 FAIL | 3.994 pass | RightLeg s11 | 0.0024 | 4.900 | 17.25 / 1.67 |
| hopping/train | SOMA | 5.196 | 7.218 FAIL | 5.162 pass | LeftFoot s20 | 3.4028 | 4.919 | 27.82 / 6.29 |
| hopping/train | traveller | 4.992 | 5.308 FAIL | 4.959 pass | LeftFoot s23 | 3.9606 | 4.906 | 28.53 / 5.28 |
| library/train | SOMA | 16.529 | 16.147 pass | 16.364 pass | RightToeBase s16 | 0.0000 | 4.900 | 18.63 / 0.73 |
| library/train | traveller | 15.812 | 15.525 pass | 15.654 pass | RightToeBase s16 | 0.0000 | 4.900 | 19.40 / 0.68 |
| running/adaptation | SOMA | 30.659 | 35.857 FAIL | 30.352 pass | RightHandMiddleEnd s12 | 0.0000 | 4.900 | 43.74 / 2.25 |
| running/adaptation | traveller | 21.646 | 26.954 FAIL | 21.430 pass | LeftHand s10 | 0.0001 | 4.900 | 37.72 / 2.03 |
| running/old | SOMA | 6.853 | 9.623 FAIL | 6.784 pass | LeftShin s16 | 0.0000 | 4.900 | 8.48 / 0.96 |
| running/old | traveller | 5.636 | 8.770 FAIL | 5.579 pass | LeftForeArm s15 | 0.0000 | 4.900 | 12.87 / 1.07 |
| running/train | SOMA | 17.630 | 12.823 pass | 17.322 pass | RightHandMiddleEnd s21 | 0.0000 | 4.950 | 27.42 / 3.19 |
| running/train | traveller | 12.303 | 9.378 pass | 12.180 pass | RightLeg s22 | 0.0000 | 4.900 | 25.17 / 2.39 |
| standing/adaptation | SOMA | 23.455 | 18.823 pass | 23.221 pass | RightHandMiddleEnd s21 | 0.0000 | 4.900 | 12.79 / 0.27 |
| standing/adaptation | traveller | 14.458 | 14.256 pass | 14.314 pass | RightHand s18 | 0.0000 | 4.900 | 13.43 / 0.69 |
| standing/old | SOMA | 32.808 | 25.435 pass | 32.480 pass | LeftHandMiddleEnd s23 | 0.0000 | 4.900 | 17.20 / 1.18 |
| standing/old | traveller | 23.567 | 21.189 pass | 23.331 pass | RightHand s22 | 0.0000 | 4.900 | 17.40 / 1.15 |
| standing/train | SOMA | 25.105 | 19.285 pass | 24.853 pass | LeftHandMiddleEnd s23 | 0.0000 | 4.902 | 20.72 / 1.53 |
| standing/train | traveller | 19.747 | 15.584 pass | 19.550 pass | LeftHand s23 | 0.0000 | 4.900 | 20.25 / 1.16 |

Continuity: 0 of 20 fail (10 before). Contacts: 17 cases below 0.0001 cm,
one hopping/old traveller case at 0.002359 cm (solver residual, not a reach
deficit); hopping/train 3.403/3.961 cm, see below. Lean ≤ 4.9502° in every
case, within the 5° bound. Reverse seek and stride-7 sampling are bitwise
equal to forward evaluation in all 20. Raw-root `base` measurements and the
bounds are unchanged from the archived results.

Two things to read in the table, both consequences of the priority order
rather than defects: a joint step at the bound after landing (running/train,
standing/*) is the recovery toward the generated pose running as fast as the
hard continuity bound allows, since the generated pose is the lowest
priority; and a lean of 4.9° is the generated torso resting on the upright
cone, since the generated poses lean more than 5° through flight. Both are
tunable in one place (`MARGIN`, `CONE_DEGREES`) if art prefers a slower
recovery or a tighter cone. The maximum deviation from the generated pose is
43.74 cm (running/adaptation SOMA, LeftFoot s11, the free foot drawn toward
its planned anchor); the mean deviation is below 3.2 cm except hopping/train
(6.29/5.28 cm), whose pelvis sits at the bottom of the slack box.

### hopping/train: planned contact infeasible under the hard root

Both rigs report two `contact reach` conflicts, at samples 24 and 25:
deficit 3.3961 cm and 1.1678 cm (SOMA), 3.9547 cm and 1.8513 cm (traveller).
The solve lands at those deficits (contact maximum 3.4028/3.9606 cm; the
difference from the deficit is the knee proxy). This is a conflict between
two hard constraints, the root and the planned contact, and is reported, not
waived.

Proof (`reachbound.py` in the handoff scratch, SOMA rig, sample 24): with
the pelvis orientation fixed by the hard root (planned yaw, generated pitch
and roll) and hips Y anywhere in the slack box (−25%/+10% of leg length), a
straight leg reaches at best to 3.3961 cm from the anchor (minimum over
2001 Y values of |hip − anchor| − leg length); at sample 25, 1.1678 cm. The
deficit is not the leg's length: letting the pelvis pitch and roll ±60° about
the planned yaw brings the best deficit to 0.7462 cm, and the sphere bound
(any pelvis orientation, |root − anchor| − |hip offset| − leg length) is
−1.29 cm, feasible. The anchor is therefore reachable only by rotating the
pelvis away from its generated orientation, which the root contract forbids.
The fix is in the plan or the model (the anchor sits 3.4 cm beyond the
straight leg at the hard root), not in the operator.

### Tests

`cargo test -p og-runtime --example flight_plan` (5 tests). Two were added
with the solve and fail on the per-sample pipeline:

- `landing_on_a_straight_leg_keeps_every_joint_step_under_the_raw_bound`: a
  14-node SOMA-named rig lands on a straight leg whose foot is pinned at full
  reach while the pelvis sinks 2 cm per sample; every joint step must stay
  under the raw bound (2.88 cm), the contact under 0.01 cm, lean under 5°,
  no conflict. On the old code the LeftShin steps 8.63 cm at sample 14 (the
  knee pop of attempt 3 in the brief, reproduced on a synthetic clip).
- `unreachable_contact_reports_its_triangle_bound_and_lands_at_it`: an anchor
  1 m ahead must produce exactly one `contact reach` conflict at sample 14
  with deficit √(1² + 0.63²) − 0.84 m (±1 mm), the contact error within 2 mm
  of it and the pelvis at the bottom of the box. The old code reports no
  conflict.

Checks on the author's orb: `cargo check`, `cargo fmt --check`,
`cargo clippy -p og-runtime --example flight_plan -- -D warnings`,
`og api check`, the 5 example tests. No GPU, no wasm, no reference frame:
this is a CPU experiment and its acceptance is numeric.

### Limitations

The solve holds every sample of the clip: a production streaming boundary
must hold all of the clip's samples; a finite-window replacement has not
been shown equivalent. Restarting the solve per model batch is not the
contract. Gesture and gaze
priorities and leg joint limits are still not integrated (the leg reach is
the two-bone triangle, with no knee hyperextension check beyond the
straight-leg bound). The deviation from the generated pose is reported, not
bounded.

### Art handoff on motion-takeover

The integrator independently reran all 20 cases: zero landing failures,
bitwise reverse/stride-7 replay, maximum lean 4.950173°, contacts as above.
`MARGIN = 0.01` and `CONE_DEGREES = 4.9` are unchanged. Art must judge the
near-bound recovery speed and the cone-resting torso before either is tuned
or motion.jump is unblocked. Hopping/train's four sample/rig reach deficits
remain plan/model work, not an operator waiver. Integrated palette world-hash
replay, authored gesture/gaze constraints and leg limits remain separate open
items; numeric continuity is not a claim that those paths are implemented.

The optional fifth argument `--review` exports raw, root-aligned and corrected
FK positions without changing the solve or nominal measurements. The generator
below turns that export into a disposable diagnostic skeleton viewer with a
metre grid, contact anchors and phase/sample-labelled captures. It draws actual
exported positions, not a second implementation of the solve. It is not a
skinned-character or runtime-integration acceptance test. Only macrunner draws
it; the author checks data, types and CPU logic, not pixels.

```sh
target/debug/examples/flight_plan \
  "$HANDOFF/recordings-v2.json" "$HANDOFF/model.safetensors" \
  "$HANDOFF/traveller.glb" "$HANDOFF/meshy.ogt" --review > flight-review.json
python3 tests/evidence/motionbricks/flight-review.py flight-review.json /tmp/flight-review
# On macrunner, with its built og:
og run /tmp/flight-review --commands "$(cat /tmp/flight-review/case-08.commands)"
```

`index.json` maps all 20 case indices. Prioritize cases 00/01
(hopping/adaptation), 08/09 (running/adaptation), 12/13 (running/train), and
18/19 (standing/train), both rigs: takeoff/apex for CONE_DEGREES, landing−2
through landing+5 and recovery for MARGIN. Cases 04/05 expose the unwaived
hopping/train anchor conflicts. Each script shows raw/aligned/corrected side
by side, then the corrected apex in detail. `view:corrected;play` supports
interactive 30 Hz review. Source-ground-projected root provenance and zero
original-library training jump windows remain unchanged.

## motion.jump: vertical ownership must be settled before IDL

The controller will lift the entity through its ballistic arc. Today's runtime
adds the entity elevation to generated hips Y. If the generated Y already
contains the jump arc, this counts the arc twice. The ground-projected fixture
deliberately does not solve that production problem. The controller must remain
the only owner of the ballistic elevation, including ceiling hits, ledges and
actual contact. No IDL change should precede a decision on pose-local height.

Options to evaluate in that design:

1. **Explicit generated-root elevation plus local pelvis height (preferred
   contract).** Separate the generator's vertical locomotion track from its
   pose-local pelvis height. Subtract that locomotion track from the generated
   hips, then add the controller elevation. Crouch/compression remains in the
   local pose; launch/apex/landing travel comes only from the controller. This
   needs a trustworthy reference trajectory or a model-output contract: hips Y
   alone cannot identify the two quantities. Never infer simulation events
   from the generated motion. Source/library metadata and training supervision
   would need to establish how the tracks are separated.
2. **Remove a declared nominal generation arc.** Store the vertical arc the
   generator was conditioned on, subtract that arc at absolute sample age, and
   add actual controller elevation. This preserves the residual pelvis motion
   without changing model outputs, but is safe only if that nominal arc really
   describes the generated root. A model error in the nominal arc remains a
   presentation error; subtracting actual controller Y from unrelated generated
   hips does not establish the separation. Interrupted flight and recovery
   need a continuous residual policy, not phase-based switches.
3. **A planned local pelvis-height envelope during flight.** Replace generated
   hips Y with a rig-calibrated local pelvis-height envelope (with continuous
   preparation/recovery transitions), then add controller Y. This unambiguously
   avoids double elevation but discards generated pelvis compression and may
   alter contacts/gesture intent. It is a different presentation policy, not
   the current approved fixture correction, and needs explicit acceptance.

Do not simply add both arcs, remove all pelvis Y, or broaden IK slack to conceal
the discrepancy. Whichever option is selected must later be tested for the
apex height, ledge-up/down landings, ceiling/collision truncation, retained
walkTo, contact-started recovery and snapshot/seek behavior. No test of those
unimplemented semantics is claimed here.
