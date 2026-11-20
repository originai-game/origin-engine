<meta name="component" content="RhythmTiming">
<meta name="strings" content="./rhythm.en.ogt">
<meta name="prop:counts" content="list" data-doc="The judgement tally so far in the game's order, keyed rows of { id, judgement, label, count }: judgement the kind (perfect, great, good, bad, miss; its colour, on data-judgement), label its word as the game names it, count how many.">
<meta name="prop:hits" content="list" data-doc="The latest hits on the timing scale, keyed rows of { id, judgement, offset, age }: offset from −1 (earliest the game judges) through 0 (on the beat) to 1 (latest); age from 0 (this hit) to 1 (about to drop), which fades it. The game keeps the list short (eight or so).">
<meta name="fixture:default" content='{"counts":[{"id":"perfect","judgement":"perfect","label":"Perfect","count":380},{"id":"great","judgement":"great","label":"Great","count":30},{"id":"good","judgement":"good","label":"Good","count":2},{"id":"miss","judgement":"miss","label":"Miss","count":0}],"hits":[{"id":"h4","judgement":"perfect","offset":0.12,"age":0},{"id":"h3","judgement":"perfect","offset":0.04,"age":0.3},{"id":"h2","judgement":"great","offset":-0.24,"age":0.6},{"id":"h1","judgement":"perfect","offset":0.08,"age":0.85}]}'>
<meta name="fixture:early" content='{"counts":[{"id":"perfect","judgement":"perfect","label":"Perfect","count":12},{"id":"great","judgement":"great","label":"Great","count":9},{"id":"good","judgement":"good","label":"Good","count":5},{"id":"bad","judgement":"bad","label":"Bad","count":3},{"id":"miss","judgement":"miss","label":"Miss","count":4}],"hits":[{"id":"e3","judgement":"good","offset":-0.7,"age":0},{"id":"e2","judgement":"bad","offset":-0.92,"age":0.4},{"id":"e1","judgement":"great","offset":-0.4,"age":0.7}]}'>
<meta name="fixture:empty" content='{"counts":[],"hits":[]}'>
<meta name="fixture:overflow" content='{"counts":[{"id":"critical","judgement":"perfect","label":"Transcendent Critical Perfect","count":98765},{"id":"perfect","judgement":"perfect","label":"Perfect","count":12345},{"id":"great","judgement":"great","label":"Great","count":0},{"id":"good","judgement":"good","label":"Good","count":0},{"id":"bad","judgement":"bad","label":"Bad","count":0},{"id":"miss","judgement":"miss","label":"Miss","count":0}],"hits":[{"id":"o1","judgement":"miss","offset":1.6,"age":0}]}'>

<!--
  How the run is being judged, at a corner of the field: the tally so far,
  each judgement in its colour, and under it the timing scale from Early
  to Late with the beat at its centre and the latest hits as ticks, the
  newest brightest. The game judges and counts; read only.
-->
<section id="RhythmTiming" part="frame" variant="hud" layout="stack" gap="tight" role="group" aria-label={t("rhythm.timing.name")}>
  {#if counts.length}
  <ul part="tally" class="tally" layout="stack" gap="none" aria-label={t("rhythm.results.counts")}>
    {#each counts as c (c.id)}<li part="entry" class="row" layout="row" gap="wide" class:judgement-bad={(c.judgement) == 'bad'} class:judgement-good={(c.judgement) == 'good'} class:judgement-great={(c.judgement) == 'great'} class:judgement-miss={(c.judgement) == 'miss'} class:judgement-perfect={(c.judgement) == 'perfect'}>
      <span part="kind" class="name" fill>{c.label}</span>
      <span part="count" class="count">{formatNumber(c.count)}</span>
    </li>{/each}
  </ul>
  {/if}
  <div class="meter" layout="stack" gap="hair">
    <span part="legend" class="label">{t("rhythm.timing.label")}</span>
    <div class="scale-row" layout="row" gap="tight">
    <div id="Scale" part="scale" layout="overlay">
      <div class="anchor-top" id="Centre" part="centre" anchor="top-edge"></div>
      {#each hits as h (h.id)}<div part="hit" class="hit" anchor="top-start" style:--at={h.offset <= -1 ? 0 : h.offset >= 1 ? 1 : (h.offset + 1) / 2} style:--age={h.age <= 0 ? 0 : h.age >= 1 ? 1 : h.age} class:judgement-bad={(h.judgement) == 'bad'} class:judgement-good={(h.judgement) == 'good'} class:judgement-great={(h.judgement) == 'great'} class:judgement-miss={(h.judgement) == 'miss'} class:judgement-perfect={(h.judgement) == 'perfect'}></div>{/each}
    </div>
      <span part="end" class="end early">{t("rhythm.judge.early")}</span>
      <span part="end" class="end late">{t("rhythm.judge.late")}</span>
    </div>
  </div>
</section>

<style>
/* On a smoked plate with two corners cut, so its coloured words read over
   any lane's glow. */
#RhythmTiming { inline-size: calc(196 * var(--font-step-px)); max-inline-size: stretch; padding: var(--space-sm) var(--space-md); color: var(--hud-ink); background: var(--hud-plate-fill); box-shadow: var(--hud-plate-rim); border-radius: var(--hud-plate-cut) 0; corner-shape: var(--cut-shape); clip-path: border-box; --layout-anchor-inset: 0; }
.tally { margin: 0; padding: 0; list-style: none; }
.row { align-items: baseline; }
.name { min-inline-size: 0; font-size: var(--hud-size-sm); font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); white-space: nowrap; text-overflow: ellipsis; overflow: hidden; }
.count { flex: none; font-size: var(--hud-size-sm); font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); font-variant-numeric: var(--hud-figure-numeric); white-space: nowrap; }
.row.judgement-perfect { color: var(--judge-perfect-ink); }
.row.judgement-great { color: var(--judge-great-ink); }
.row.judgement-good { color: var(--judge-good-ink); }
.row.judgement-bad { color: var(--judge-bad-ink); }
.row.judgement-miss { color: var(--judge-miss-ink); }
.label, .end { font-size: var(--hud-label-size); font-weight: var(--hud-label-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); color: var(--hud-label-ink); white-space: nowrap; }
.end { color: var(--hud-ink-dim); }
/* The scale: a thin well, the beat a white tick at its centre, each hit a
   tick in its judgement's colour that fades with its age. */
#Scale { block-size: calc(6 * var(--font-step-px)); overflow: visible; background: var(--bar-well); box-shadow: var(--bar-rim); }
#Centre { inline-size: calc(2 * var(--font-step-px)); block-size: calc(14 * var(--font-step-px)); margin-block-start: calc(-4 * var(--font-step-px)); background: var(--hud-tick); }
.hit { left: calc(var(--at) * 100%); inline-size: calc(3 * var(--font-step-px)); block-size: calc(12 * var(--font-step-px)); margin-block-start: calc(-3 * var(--font-step-px)); translate: -50% 0; opacity: calc(1 - var(--age) * 0.8); background: var(--judge-perfect-ink); }
.hit.judgement-great { background: var(--judge-great-ink); }
.hit.judgement-good { background: var(--judge-good-ink); }
.hit.judgement-bad { background: var(--judge-bad-ink); }
.hit.judgement-miss { background: var(--judge-miss-ink); }
/* The scale over Early and Late at its ends; on a phone they flank it on
   one line. */
.scale-row { flex-wrap: wrap; row-gap: var(--space-xxs); justify-content: space-between; }
#Scale { order: -1; flex: 1 1 100%; }
/* On a phone the tally runs as one line of pairs over a full-width scale. */
#RhythmTiming:size(compact) { inline-size: stretch; }
.tally:size(compact) { flex-direction: row; flex-wrap: wrap; column-gap: var(--space-md); }
.row:size(compact) { --layout-gap-wide: var(--space-xs); }
.name:size(compact), .count:size(compact) { font-size: var(--hud-size-xs); }
.label:size(compact) { display: none; }
.scale-row:size(compact) { flex-wrap: nowrap; }
#Scale:size(compact) { order: 0; flex: 1 1 0; min-inline-size: 0; }
.early:size(compact) { order: -1; }
.anchor-top { display: flex; justify-content: center; }
.anchor-top { display: flex; justify-content: center; }
</style>
