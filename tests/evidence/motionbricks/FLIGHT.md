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

This runs no GPU and does not use certification. The inherited spring port is
Daniel Holden's Spring-It-On, revision
`4d97b497a78e40c7b47d49e8a9da27e4aa5616d6`; its MIT licence is in
`third_party/spring-it-on/LICENSE`.

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

Integrated operator world-hash replay, gesture priorities, leg limits, contact
and landing acceptance, and macrunner/art-director review remain open. The
example's inherited `world_replay` exercise does not integrate the palette and
must not be presented as integrated-operator acceptance. No new training or
generation spend is authorized.

## CPU rerun on engine base 9bb81cb41

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
