<meta name="component" content="RhythmJudgement">
<meta name="strings" content="./rhythm.en.ogt">
<meta name="parts" content="RhythmJudgement.frame RhythmJudgement.combo RhythmJudgement.count RhythmJudgement.legend RhythmJudgement.mark RhythmJudgement.call RhythmJudgement.word RhythmJudgement.timing RhythmJudgement.offset">
<meta name="prop:judgement" content="string" data-default="none" data-doc="The last hit's judgement as the game judged it: perfect, great, good, bad, miss, or none (nothing shown); on data-judgement for the pack, and always a word.">
<meta name="prop:label" content="string" data-doc="The judgement's word as the game names it (Critical, Marvelous); empty takes the kit's word for judgement.">
<meta name="prop:timing" content="string" data-doc="early or late under the word (Early, Late); empty hides it.">
<meta name="prop:offset" content="string" data-doc="The hit's offset as the game words it (+4 ms, −12 ms), beside the timing; empty hides it.">
<meta name="prop:hit" content="number" data-doc="The hit's id: a new value draws the word afresh (each note judged), so the word and the combo pop again.">
<meta name="prop:combo" content="number" data-doc="The running combo as the game counts it; 0 hides it.">
<meta name="prop:broken" content="bool" data-doc="The combo just broke: Combo broken shows in its place.">
<meta name="prop:mark" content="string" data-doc="The run is still on course for a mark: fullCombo (Full combo), allPerfect (All perfect), or empty for none; on data-mark.">
<meta name="fixture:default" content='{"judgement":"perfect","label":"","timing":"late","offset":"+4 ms","hit":1,"combo":412,"broken":false,"mark":"fullCombo"}'>
<meta name="fixture:great" content='{"judgement":"great","label":"","timing":"early","offset":"−21 ms","hit":2,"combo":129,"broken":false,"mark":""}'>
<meta name="fixture:good" content='{"judgement":"good","label":"","timing":"late","offset":"+48 ms","hit":3,"combo":130,"broken":false,"mark":""}'>
<meta name="fixture:miss" content='{"judgement":"miss","label":"","timing":"","offset":"","hit":4,"combo":0,"broken":true,"mark":""}'>
<meta name="fixture:none" content='{"judgement":"none","label":"","timing":"","offset":"","hit":0,"combo":0,"broken":false,"mark":""}'>
<meta name="fixture:overflow" content='{"judgement":"perfect","label":"Transcendent Critical Perfect","timing":"late","offset":"+1 ms","hit":5,"combo":12345,"broken":false,"mark":"allPerfect"}'>

<!--
  The play field's centre: the running combo as the hero figure with its
  word (Combo broken when it breaks, Full combo or All perfect while the
  run holds one), and under it the last hit's judgement, each in its own
  lit colour, with Early or Late and the offset. A new hit id pops the word
  and the count again. The game owns timing windows, judging and counting;
  this only shows what it was told. Read only; not a live region, since a
  hit's word changes many times a second.
-->
<div id="RhythmJudgement" part="RhythmJudgement.frame" class="bare" data-layout="stack" data-gap="none" role="group" aria-label={t("rhythm.judge.name")} data-judgement={judgement} data-mark={mark}>
  {#if combo > 0 || broken}
  <div part="RhythmJudgement.combo" class="combo" data-layout="stack" data-gap="none" data-broken={broken}>
    {#if !broken}{#key hit}<span part="RhythmJudgement.count" class="count" data-motion="beat">{formatNumber(combo)}</span>{/key}{/if}
    <span part="RhythmJudgement.legend" class="combo-word">{broken ? t("rhythm.judge.broken") : t("rhythm.judge.combo")}</span>
    {#if !broken && mark != ''}<span part="RhythmJudgement.mark" class="mark" data-mark={mark}>{mark == 'allPerfect' ? t("rhythm.judge.allPerfect") : t("rhythm.judge.fullCombo")}</span>{/if}
  </div>
  {/if}
  {#if judgement != 'none'}
    {#key hit}
    <div part="RhythmJudgement.call" class="plate" data-layout="stack" data-gap="none" data-judgement={judgement} data-motion="stamp">
      <span part="RhythmJudgement.word" class="word" data-judgement={judgement}>{label != '' ? label : (judgement == 'perfect' ? t("rhythm.judge.perfect") : (judgement == 'great' ? t("rhythm.judge.great") : (judgement == 'good' ? t("rhythm.judge.good") : (judgement == 'bad' ? t("rhythm.judge.bad") : t("rhythm.judge.miss")))))}</span>
      {#if timing != '' || offset != ''}
      <div class="when" data-layout="row" data-gap="tight">
        {#if timing != ''}<span part="RhythmJudgement.timing" class="timing" data-timing={timing}>{timing == 'early' ? t("rhythm.judge.early") : t("rhythm.judge.late")}</span>{/if}
        {#if offset != ''}<span part="RhythmJudgement.offset" class="offset" data-timing={timing}>{offset}</span>{/if}
      </div>
      {/if}
    </div>
    {/key}
  {/if}
</div>

<style>
:host { inline-size: stretch; align-items: center; pointer-events: none; color: var(--hud-ink); text-shadow: var(--hud-shadow); -webkit-text-stroke-width: var(--hud-stroke-width); -webkit-text-stroke-color: var(--hud-stroke-color); paint-order: stroke fill; }
/* The combo: a slanted hero figure with its tracked word under it. */
.combo { align-items: center; max-inline-size: stretch; }
.count { font-size: var(--hud-size-hero); line-height: 0.85; font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); font-style: var(--judge-slant); font-variant-numeric: var(--hud-figure-numeric); white-space: nowrap; filter: var(--combo-glow); }
.combo-word { font-size: var(--hud-size-sm); font-weight: var(--hud-label-weight); letter-spacing: var(--combo-tracking); text-transform: var(--hud-label-transform); color: var(--hud-accent); white-space: nowrap; }
.combo[data-broken=true] .combo-word { color: var(--judge-miss-ink); letter-spacing: var(--hud-label-tracking); }
/* A small word over the scene sits on a tag plate. */
.mark { margin-block-start: var(--space-xs); padding: 0 var(--space-sm); font-size: var(--hud-label-size); font-weight: var(--hud-label-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); color: var(--hud-gold); white-space: nowrap; background: var(--hud-tag-fill); box-shadow: var(--hud-tag-rim); -webkit-text-stroke-width: 0; text-shadow: none; clip-path: polygon(var(--hud-tag-cut) 0, 100% 0, 100% calc(100% - var(--hud-tag-cut)), calc(100% - var(--hud-tag-cut)) 100%, 0 100%, 0 var(--hud-tag-cut)); }
/* The judgement: a gradient through slanted, tracked capitals over a glow. */
.plate { align-items: center; max-inline-size: stretch; margin-block-start: var(--space-xxl); }
.word { max-inline-size: stretch; min-inline-size: 0; font-size: var(--hud-size-judge); line-height: 1.05; font-weight: var(--hud-figure-weight); font-stretch: var(--bar-figure-stretch); font-style: var(--judge-slant); letter-spacing: var(--judge-tracking); text-transform: var(--hud-label-transform); text-align: center; white-space: nowrap; text-overflow: ellipsis; overflow: hidden; color: transparent; -webkit-text-stroke-width: 0; text-shadow: none; background-clip: var(--hud-text-clip); }
.word[data-judgement=perfect] { background-image: var(--judge-perfect-fill); filter: var(--judge-perfect-glow); }
.word[data-judgement=great] { background-image: var(--judge-great-fill); filter: var(--judge-great-glow); }
.word[data-judgement=good] { background-image: var(--judge-good-fill); filter: var(--judge-good-glow); }
.word[data-judgement=bad] { background-image: var(--judge-bad-fill); filter: var(--judge-bad-glow); }
.word[data-judgement=miss] { background-image: var(--judge-miss-fill); filter: var(--judge-miss-glow); }
.when { align-items: baseline; justify-content: center; }
.timing { font-size: var(--hud-label-size); font-weight: var(--hud-label-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); white-space: nowrap; }
.offset { font-size: var(--hud-size-sm); font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); font-variant-numeric: var(--hud-figure-numeric); color: var(--hud-gold); white-space: nowrap; }
.timing[data-timing=early], .offset[data-timing=early] { color: var(--judge-early-ink); }
.timing[data-timing=late] { color: var(--judge-late-ink); }
@media (width < 600px) {
  .count { font-size: var(--hud-size-hero-compact); }
  .word { font-size: var(--hud-size-judge-compact); }
  .plate { margin-block-start: var(--space-md); }
}
</style>
