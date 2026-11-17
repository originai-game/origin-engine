<meta name="component" content="RhythmResults">
<meta name="strings" content="./rhythm.en.ogt">
<meta name="parts" content="RhythmResults.frame RhythmResults.band RhythmResults.title RhythmResults.grade RhythmResults.badge RhythmResults.readout RhythmResults.figure RhythmResults.row RhythmResults.name RhythmResults.caption RhythmResults.well">
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
<section id="RhythmResults" part="RhythmResults.frame" data-layout="shell" data-gap="loose" aria-label={song != '' ? song : t("rhythm.results.title")} data-cleared={cleared} data-mark={mark}>
  <header part="RhythmResults.band" data-layout="row"><h2 part="RhythmResults.title">{song != '' ? song : t("rhythm.results.title")}</h2></header>
  <div class="body" data-layout="scroll" data-fill>
    <div class="content" data-layout="stack" data-gap="loose">
      <div class="top" data-layout="cluster" data-gap="wide">
      <div class="verdict" data-layout="row" data-gap="loose">
        {#if grade != ''}<span part="RhythmResults.grade RhythmResults.figure" class="grade" aria-label={t("rhythm.results.gradeName", { grade: grade })} data-cleared={cleared}>{grade}</span>{/if}
        <div class="words" data-layout="stack" data-gap="hair">
          <span part="RhythmResults.badge" data-tag={cleared ? 'cleared' : 'failed'}>{cleared ? t("rhythm.results.cleared") : t("rhythm.results.failed")}</span>
          {#if mark != ''}<span part="RhythmResults.badge" data-tag={mark}>{mark == 'allPerfect' ? t("rhythm.results.allPerfect") : t("rhythm.results.fullCombo")}</span>{/if}
        </div>
      </div>
      <div part="RhythmResults.readout RhythmResults.figure" class="score-cell" data-layout="stack" data-gap="none" role="group" aria-label={t("rhythm.results.score")}>
        <span class="caption" part="RhythmResults.caption">{t("rhythm.results.score")}</span>
        <span part="RhythmResults.figure" class="score">{formatNumber(score)}</span>
        <div class="best" data-layout="cluster" data-gap="tight">
          {#if newBest}<span part="RhythmResults.badge" data-tag="newBest">{t("rhythm.results.newBest")}</span>{/if}
          {#if previous > 0}<span class="previous" part="RhythmResults.caption">{t("rhythm.results.previous", { score: previous })}</span>{/if}
        </div>
      </div>
      </div>
      <div class="lower" data-layout="grid" data-gap="wide">
      <div class="stats" part="RhythmResults.well" data-layout="stack">
        <div part="RhythmResults.readout RhythmResults.figure" class="stat" data-layout="row" role="group" aria-label={t("rhythm.results.maxCombo")}><span class="caption" part="RhythmResults.caption">{t("rhythm.results.maxCombo")}</span><span part="RhythmResults.figure" class="value">{formatNumber(maxCombo)}</span></div>
        {#if accuracy != ''}<div part="RhythmResults.readout RhythmResults.figure" class="stat" data-layout="row" role="group" aria-label={t("rhythm.results.accuracy")}><span class="caption" part="RhythmResults.caption">{t("rhythm.results.accuracy")}</span><span part="RhythmResults.figure" class="value">{accuracy}</span></div>{/if}
      </div>
      {#if counts.length}
        <div class="counts" data-layout="stack" data-gap="hair" role="list" aria-label={t("rhythm.results.counts")}>
          {#each counts as c (c.id)}
            <div part="RhythmResults.row" data-layout="row" role="listitem" aria-label={c.label} data-judgement={c.judgement}><span part="RhythmResults.name" class="label" data-fill>{c.label}</span><span class="count" part="RhythmResults.figure">{formatNumber(c.count)}</span></div>
          {/each}
        </div>
      {:else}
        <p class="empty-text" part="RhythmResults.caption" role="status">{t("rhythm.results.empty")}</p>
      {/if}
      </div>
    </div>
  </div>
  <footer class="actions" data-layout="row">
    <button id="Retry" variant="ghost" disabled={!canRetry} onclick={retry}><span>{t("rhythm.results.retry")}</span></button>
    <button id="Next" variant="primary" onclick={next} onback={next}><kbd class="key" data-action="back"></kbd><span>{t("rhythm.results.next")}</span></button>
  </footer>
</section>

<style>
:host { inline-size: stretch; min-inline-size: 0; block-size: stretch; }
[part="RhythmResults.band"] { width: stretch; flex: none; }
[part="RhythmResults.title"] { margin: 0; white-space: nowrap; text-overflow: ellipsis; overflow: hidden; }
.body { overflow-y: auto; flex: 1; min-height: 0; }
.content { width: stretch; padding: var(--space-xs); }
/* The verdict and the score share a row and wrap rather than cut a figure. */
.top { width: stretch; justify-content: space-between; }
.lower { grid-template-columns: minmax(0, 1fr) minmax(0, 1fr); width: stretch; align-items: start; }
.verdict { flex: none; }
[part="RhythmResults.grade RhythmResults.figure"] { flex: none; min-width: var(--size-xl); --figure-size: var(--font-step-5xl); --figure-weight: var(--font-weight-bold); text-align: center; white-space: nowrap; }
.words { align-items: flex-start; min-width: 0; }
[part="RhythmResults.badge"] { white-space: nowrap; padding: 0 var(--space-xs); }
.score-cell { flex: none; max-width: stretch; align-items: flex-end; margin-inline-start: auto; }
.score { --figure-size: var(--font-step-3xl); --figure-weight: var(--font-weight-semibold); white-space: nowrap; }
.best { justify-content: flex-end; }
.previous { white-space: nowrap; }
.stats { min-width: 0; padding: var(--space-sm); }
.stat { align-items: baseline; justify-content: space-between; }
.value, .count { flex: none; white-space: nowrap; }
.counts { min-width: 0; }
[part="RhythmResults.row"] { width: stretch; padding: var(--space-xxs) var(--space-sm); }
.label { flex: 1; min-width: 0; text-align: start; white-space: nowrap; text-overflow: ellipsis; overflow: hidden; }
.empty-text { margin: 0; padding: var(--space-md); text-align: center; }
.actions { width: stretch; flex: none; justify-content: flex-end; }
.actions button { min-height: var(--size-touch); min-width: calc(var(--size-xxl) * 1.25); gap: var(--space-xs); }
@media (pointer: coarse) { .key { display: none; } }
/* A narrow panel (a phone, or a side column on a wide screen) stacks the
   verdict, score, figures and tally in one column; the actions share the width. */
@container (width < 560px) {
  .content { --layout-gap-loose: var(--layout-gap); }
  .lower { grid-template-columns: minmax(0, 1fr); --layout-gap-wide: var(--layout-gap); }
  .score-cell { align-items: flex-start; margin-inline-start: 0; }
  .best { justify-content: flex-start; }
  .actions button { flex: 1; min-width: 0; }
}
</style>
