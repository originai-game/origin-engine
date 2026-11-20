<meta name="component" content="RhythmResults">
<meta name="strings" content="./rhythm.en.ogt">
<meta name="prop:song" content="string" data-doc="The song's title as the heading; empty takes the kit's (Results).">
<meta name="prop:grade" content="string" data-doc="The grade the game awarded (S, AA, C); empty hides it.">
<meta name="prop:cleared" content="bool" data-doc="The game's verdict: Cleared, or Failed (data-cleared).">
<meta name="prop:score" content="number" data-doc="The run's score as the game counts it, shown with grouping.">
<meta name="prop:previous" content="number" data-doc="The player's best before this run (Best …); 0 hides it.">
<meta name="prop:newBest" content="bool" data-doc="The game says this run beat the best: New best shows by the score.">
<meta name="prop:maxCombo" content="number" data-doc="The longest combo of the run.">
<meta name="prop:accuracy" content="string" data-doc="The accuracy as the game words it (97.42%); empty hides it.">
<meta name="prop:mark" content="string" data-doc="A run mark the game awarded: fullCombo (Full combo), allPerfect (All perfect), or empty for none; on data-mark.">
<meta name="prop:counts" content="list" data-doc="The judgement tally in the game's order, keyed rows of { id, judgement, label, count }: judgement the kind (perfect, great, good, bad, miss; on data-judgement), label its word as the game names it, count how many.">
<meta name="prop:canRetry" content="bool" data-doc="Retry is offered (false: disabled).">
<meta name="action:back" content="Next">
<meta name="fixture:default" content='{"song":"Skyline Drive","grade":"S","cleared":true,"score":962410,"previous":948220,"newBest":true,"maxCombo":412,"accuracy":"97.42%","mark":"","counts":[{"id":"perfect","judgement":"perfect","label":"Perfect","count":388},{"id":"great","judgement":"great","label":"Great","count":21},{"id":"good","judgement":"good","label":"Good","count":3},{"id":"miss","judgement":"miss","label":"Miss","count":2}],"canRetry":true}'>
<meta name="fixture:failed" content='{"song":"Static Bloom","grade":"D","cleared":false,"score":198400,"previous":0,"newBest":false,"maxCombo":57,"accuracy":"76.30%","mark":"","counts":[{"id":"perfect","judgement":"perfect","label":"Perfect","count":120},{"id":"great","judgement":"great","label":"Great","count":64},{"id":"good","judgement":"good","label":"Good","count":31},{"id":"miss","judgement":"miss","label":"Miss","count":88}],"canRetry":true}'>
<meta name="fixture:fullCombo" content='{"song":"Ember Waltz","grade":"SS","cleared":true,"score":991870,"previous":995120,"newBest":false,"maxCombo":356,"accuracy":"99.12%","mark":"fullCombo","counts":[{"id":"perfect","judgement":"perfect","label":"Perfect","count":340},{"id":"great","judgement":"great","label":"Great","count":16},{"id":"good","judgement":"good","label":"Good","count":0},{"id":"miss","judgement":"miss","label":"Miss","count":0}],"canRetry":true}'>
<meta name="fixture:allPerfect" content='{"song":"Ember Waltz","grade":"SSS","cleared":true,"score":1000000,"previous":991870,"newBest":true,"maxCombo":356,"accuracy":"100.00%","mark":"allPerfect","counts":[{"id":"perfect","judgement":"perfect","label":"Perfect","count":356},{"id":"miss","judgement":"miss","label":"Miss","count":0}],"canRetry":true}'>
<meta name="fixture:disabled" content='{"song":"Vault of Echoes","grade":"A","cleared":true,"score":901330,"previous":0,"newBest":false,"maxCombo":201,"accuracy":"93.80%","mark":"","counts":[{"id":"perfect","judgement":"perfect","label":"Perfect","count":240},{"id":"miss","judgement":"miss","label":"Miss","count":9}],"canRetry":false}'>
<meta name="fixture:empty" content='{"song":"","grade":"","cleared":false,"score":0,"previous":0,"newBest":false,"maxCombo":0,"accuracy":"","mark":"","counts":[],"canRetry":true}'>
<meta name="fixture:overflow" content='{"song":"The Unbearably Long Extended Twelve-Minute Midnight Orchestral Remix of a Song","grade":"SSS+","cleared":true,"score":123456789,"previous":123450000,"newBest":true,"maxCombo":98765,"accuracy":"99.9999%","mark":"allPerfect","counts":[{"id":"critical","judgement":"perfect","label":"Transcendent Critical Perfect","count":98765},{"id":"perfect","judgement":"perfect","label":"Perfect","count":0},{"id":"great","judgement":"great","label":"Great","count":0},{"id":"good","judgement":"good","label":"Good","count":0},{"id":"bad","judgement":"bad","label":"Bad","count":0},{"id":"miss","judgement":"miss","label":"Miss","count":0},{"id":"early","judgement":"great","label":"Early","count":0},{"id":"late","judgement":"great","label":"Late","count":0}],"canRetry":true}'>

<!--
  After the song: the grade, Cleared or Failed, the score with the old best
  and New best, Full combo or All perfect, max combo and accuracy, and the
  judgement tally. Retry (retry; disabled when the game refuses it) and
  Next (next; also Escape, pad B) report once. The game computes every
  figure, grade and verdict.
-->
<section id="RhythmResults" part="frame" variant="bare" layout="stack" fill gap="loose" aria-label={song != '' ? song : t("rhythm.results.title")} data-mark={mark} class:not-cleared={!(cleared)}>
  <header part="band" class="plate band" layout="row"><h2 class="title" part="title">{song != '' ? song : t("rhythm.results.title")}</h2></header>
  <div class="body" layout="scroll" fill>
    <div class="content" layout="stack" gap="loose">
      <div class="top" layout="wrap" gap="wide">
      <div class="verdict" layout="row" gap="loose">
        {#if grade != ''}<span part="figure" class="grade" aria-label={t("rhythm.results.gradeName", { grade: grade })} class:not-cleared={!(cleared)}>{grade}</span>{/if}
        <div class="words" layout="stack" gap="hair">
          <span class="badge" part="badge" class:tag-allPerfect={(cleared ? 'cleared' : 'failed') == 'allPerfect'} class:tag-failed={(cleared ? 'cleared' : 'failed') == 'failed'} class:tag-fullCombo={(cleared ? 'cleared' : 'failed') == 'fullCombo'} class:tag-newBest={(cleared ? 'cleared' : 'failed') == 'newBest'}>{cleared ? t("rhythm.results.cleared") : t("rhythm.results.failed")}</span>
          {#if mark != ''}<span class="badge" part="badge" class:tag-allPerfect={(mark) == 'allPerfect'} class:tag-failed={(mark) == 'failed'} class:tag-fullCombo={(mark) == 'fullCombo'} class:tag-newBest={(mark) == 'newBest'}>{mark == 'allPerfect' ? t("rhythm.results.allPerfect") : t("rhythm.results.fullCombo")}</span>{/if}
        </div>
      </div>
      <div part="figure" class="score-cell readout" layout="stack" gap="none" role="group" aria-label={t("rhythm.results.score")}>
        <span class="caption" part="caption">{t("rhythm.results.score")}</span>
        <span part="figure" class="score">{formatNumber(score)}</span>
        <div class="best" layout="wrap" gap="tight">
          {#if newBest}<span class="badge" part="badge" class:tag-newBest={true}>{t("rhythm.results.newBest")}</span>{/if}
          {#if previous > 0}<span class="previous caption" part="caption">{t("rhythm.results.previous", { score: previous })}</span>{/if}
        </div>
      </div>
      </div>
      <div class="lower grid" gap="wide">
      <div class="stats" part="well" layout="stack">
        <div part="figure" class="stat readout" layout="stack" role="group" aria-label={t("rhythm.results.maxCombo")}><span class="caption" part="caption">{t("rhythm.results.maxCombo")}</span><span part="figure" class="value">{formatNumber(maxCombo)}</span></div>
        {#if accuracy != ''}<div part="figure" class="stat readout" layout="stack" role="group" aria-label={t("rhythm.results.accuracy")}><span class="caption" part="caption">{t("rhythm.results.accuracy")}</span><span part="figure" class="value">{accuracy}</span></div>{/if}
      </div>
      {#if counts.length}
        <div class="counts" layout="stack" gap="hair" role="list" aria-label={t("rhythm.results.counts")}>
          {#each counts as c (c.id)}
            <div class="row" part="row" layout="row" role="listitem" aria-label={c.label} class:judgement-bad={(c.judgement) == 'bad'} class:judgement-good={(c.judgement) == 'good'} class:judgement-great={(c.judgement) == 'great'} class:judgement-miss={(c.judgement) == 'miss'} class:judgement-perfect={(c.judgement) == 'perfect'}><span part="name" class="label" fill>{c.label}</span><span class="count" part="figure">{formatNumber(c.count)}</span></div>
          {/each}
        </div>
      {:else}
        <p class="empty-text caption" part="caption" role="status">{t("rhythm.results.empty")}</p>
      {/if}
      </div>
    </div>
  </div>
  <footer class="actions" layout="row">
    <button id="Retry" variant="ghost" disabled={!canRetry} onclick={retry}><span>{t("rhythm.results.retry")}</span></button>
    <button id="Next" class="go" variant="primary" aria-label={t("rhythm.results.next")} onclick={next} onback={next}><kbd class="key" data-action="back"></kbd><span>{t("rhythm.results.next")}</span></button>
  </footer>
</section>

<style>
.grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(min(var(--layout-tile), 100%), 1fr)); }
/* After the song, over the blurred stage: the song's name, the grade set
   huge and slanted in the result's lit fill beside its marks, the score on
   a cut HUD plate, the max combo and accuracy on plates, and the tally,
   each judgement in its own ink with its count. Bare: no panel, the plates
   hold the figures over any stage. */
#RhythmResults { inline-size: stretch; min-inline-size: 0; block-size: stretch; color: var(--hud-ink); }
.band { width: stretch; flex: none; border: 0; padding: 0; min-height: 0; background: none; box-shadow: none; }
.title { margin: 0; font-size: var(--result-size-title); line-height: 1.05; font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); color: var(--hud-ink); white-space: nowrap; text-overflow: -og-scroll var(--motion-duration-orbit) var(--motion-duration-turn); overflow: hidden; }
.body { overflow-y: auto; flex: 1; min-height: 0; -og-scroll-fade: var(--space-xl); }
.content { width: stretch; padding: var(--space-xs); }
.top { width: stretch; justify-content: space-between; align-items: flex-end; }
.lower { grid-template-columns: minmax(0, 1fr) minmax(0, 1fr); width: stretch; align-items: start; }
.verdict { flex: none; align-items: center; }
/* The grade: condensed, slanted, in the win's gold or the loss's red. */
.grade { flex: none; min-width: var(--size-xl); padding-inline-end: var(--space-sm); font-size: var(--result-size-grade); line-height: 0.9; font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); font-style: var(--judge-slant); text-align: center; white-space: nowrap; color: transparent; background: var(--result-won-fill); background-clip: var(--hud-text-clip); filter: var(--result-won-glow); }
.grade.not-cleared { background: var(--result-lost-fill); background-clip: var(--hud-text-clip); filter: var(--result-lost-glow); }
.words { align-items: flex-start; min-width: 0; }
/* The marks: cut tags, Cleared in the accent, Failed in danger, a full
   combo and all perfect in the chart's special ramp, a new best in gold. */
.badge { white-space: nowrap; padding: 0 var(--space-sm); font-size: var(--hud-size-sm); font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); color: var(--hud-accent-ink); background: var(--hud-key-lit-fill); box-shadow: none; border-radius: var(--hud-tag-cut) 0; corner-shape: var(--cut-shape); clip-path: border-box; }
.badge.tag-failed { color: var(--hud-ink); background: var(--bar-danger); }
.badge.tag-fullCombo, .badge.tag-allPerfect { color: var(--chart-ink); background: var(--chart-special); }
.badge.tag-newBest { font-size: var(--hud-tag-size); color: var(--result-medal-ink); background: var(--hud-key-gold-fill); }
/* Plates: the score, the max combo, the accuracy and the tally. */
.score-cell, .stat, .counts { background: var(--hud-plate-fill); box-shadow: var(--hud-plate-rim); border-radius: var(--hud-plate-cut) 0; corner-shape: var(--cut-shape); clip-path: border-box; }
.score-cell { flex: 1 1 auto; max-width: stretch; min-width: 0; align-items: flex-start; padding: var(--space-md) var(--space-lg); }
.score { font-size: var(--result-size-score); line-height: 1; padding-block-end: var(--result-score-descender-space); font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); font-variant-numeric: var(--hud-figure-numeric); white-space: nowrap; }
.caption { font-size: var(--hud-label-size); font-weight: var(--hud-label-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); color: var(--hud-label-ink); }
.best { justify-content: flex-start; align-items: center; }
.previous { white-space: nowrap; letter-spacing: 0; text-transform: none; }
.stats { min-width: 0; padding: 0; background: none; box-shadow: none; border-radius: 0; display: grid; grid-template-columns: repeat(auto-fit, minmax(calc(150 * var(--font-step-px)), 1fr)); gap: var(--space-sm); }
/* A stat: its name small above its figure, the same in every plate at
   any width and text scale, so the name never runs into the figure. */
.stat { flex-direction: column; align-items: flex-start; justify-content: space-between; row-gap: var(--space-xxs); min-width: 0; padding: var(--space-sm) var(--space-md); }
.value { flex: none; white-space: nowrap; font-size: var(--hud-size-lg); font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); font-variant-numeric: var(--hud-figure-numeric); }
.counts { min-width: 0; padding: var(--space-sm) var(--space-md); }
.row { width: stretch; column-gap: var(--space-md); padding: var(--space-xxs) 0; align-items: baseline; background: none; border-radius: 0; }
.row:hover { background: none; }
.label { flex: 1; min-width: 0; text-align: start; overflow-wrap: anywhere; font-size: var(--hud-size-sm); font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); font-style: var(--judge-slant); letter-spacing: var(--judge-tracking); text-transform: var(--hud-label-transform); }
.judgement-perfect .label { color: var(--judge-perfect-ink); }
.judgement-great .label { color: var(--judge-great-ink); }
.judgement-good .label { color: var(--judge-good-ink); }
.judgement-bad .label { color: var(--judge-bad-ink); }
.judgement-miss .label { color: var(--judge-miss-ink); }
.count { flex: none; white-space: nowrap; font-size: var(--hud-size-md); font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); font-variant-numeric: var(--hud-figure-numeric); }
.empty-text { margin: 0; padding: var(--space-md); text-align: center; }
.actions { width: stretch; flex: none; justify-content: flex-end; }
.actions button { min-height: var(--size-touch); min-width: calc(var(--size-xxl) * 1.25); gap: var(--space-xs); }
/* A phone has no Escape: its key caps go. */
@media (pointer: coarse) { .key { display: none; } }
/* A narrow panel (a phone, or a side column on a wide screen) stacks the
   verdict, score, figures and tally in one column; the actions share the width. */
@container (width < 560px) {
  .content { --layout-gap-loose: var(--layout-gap); }
  .lower { grid-template-columns: minmax(0, 1fr); --layout-gap-wide: var(--layout-gap); }
  .title { font-size: var(--hud-size-md); }
  .grade { font-size: var(--result-size-grade-compact); }
  .badge { font-size: var(--hud-tag-size); }
  .score { font-size: var(--result-size-score-compact); }
  .value { font-size: var(--hud-size-md); }
  .actions button { flex: 1; min-width: 0; }
}
</style>
