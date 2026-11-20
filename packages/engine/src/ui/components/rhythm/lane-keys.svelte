<meta name="component" content="RhythmLaneKeys">
<meta name="strings" content="./rhythm.en.ogt">
<meta name="parts" content="RhythmLaneKeys.frame RhythmLaneKeys.lane RhythmLaneKeys.key">
<meta name="prop:lanes" content="list" data-doc="The lanes left to right, keyed rows of { id, key, held }: key the input that plays the lane as the game names it (S, LB); held the player holds it now (the cap lights).">
<meta name="fixture:default" content='{"lanes":[{"id":"l1","key":"S","held":true},{"id":"l2","key":"D","held":false},{"id":"l3","key":"F","held":false},{"id":"l4","key":"J","held":true},{"id":"l5","key":"K","held":false},{"id":"l6","key":"L","held":false}]}'>
<meta name="fixture:four" content='{"lanes":[{"id":"a","key":"D","held":false},{"id":"b","key":"F","held":false},{"id":"c","key":"J","held":false},{"id":"d","key":"K","held":false}]}'>
<meta name="fixture:empty" content='{"lanes":[]}'>
<meta name="fixture:overflow" content='{"lanes":[{"id":"p1","key":"Left Shoulder","held":true},{"id":"p2","key":"Left Trigger","held":false},{"id":"p3","key":"Dpad Up","held":false},{"id":"p4","key":"Y","held":false},{"id":"p5","key":"Right Trigger","held":false},{"id":"p6","key":"Right Shoulder","held":true},{"id":"p7","key":"Space","held":false},{"id":"p8","key":"Enter","held":false}]}'>

<!--
  The lanes' key caps under the judgement line, spread across the field's
  width, one per lane, each lit while the player holds it. The game reads
  input and names the keys; read only.
-->
<ol id="RhythmLaneKeys" part="RhythmLaneKeys.frame" class="hud bare" data-layout="row" data-gap="tight" aria-label={t("rhythm.lanes.name")}>
  {#each lanes as l (l.id)}<li part="RhythmLaneKeys.lane" class="lane" data-layout="center" data-fill>
    <div part="RhythmLaneKeys.key" class="key" data-layout="center" data-held={l.held} aria-label={l.held ? t("rhythm.lanes.held", { key: l.key }) : l.key}>{l.key}</div>
  </li>{/each}
</ol>

<style>
:host { flex: 1 1 auto; inline-size: 100%; margin: 0; padding: 0; list-style: none; align-items: center; }
.lane { flex: 1 1 0; min-inline-size: 0; }
.key { min-inline-size: calc(44 * var(--font-step-px)); max-inline-size: stretch; min-block-size: calc(34 * var(--font-step-px)); padding: 0 var(--space-xs); font-size: var(--hud-size-sm); font-weight: var(--hud-figure-weight); text-align: center; color: var(--hud-key-ink); background: var(--hud-key-fill); border-radius: var(--hud-key-radius); box-shadow: var(--hud-key-shadow); overflow-wrap: anywhere; }
.key[data-held=true] { background: var(--hud-key-lit-fill); box-shadow: var(--hud-key-lit-shadow); }
@media (width < 600px), (height < 500px) {
  .key { min-inline-size: calc(30 * var(--font-step-px)); min-block-size: calc(28 * var(--font-step-px)); font-size: var(--hud-size-xs); }
}
</style>
