<meta name="component" content="RhythmJudgement">
<meta name="strings" content="./rhythm.en.ogt">
<meta name="parts" content="RhythmJudgement.frame RhythmJudgement.plate RhythmJudgement.title RhythmJudgement.badge RhythmJudgement.combo RhythmJudgement.figure">
<meta name="prop:judgement" content="string" data-default="none" data-doc="The last hit's judgement as the game judged it: perfect, great, good, bad, miss, or none (nothing shown); on data-judgement for the pack, and always a word.">
<meta name="prop:label" content="string" data-doc="The judgement's word as the game names it (Critical, Marvelous); empty takes the kit's word for judgement.">
<meta name="prop:timing" content="string" data-doc="early or late beside the word (Early, Late); empty hides it.">
<meta name="prop:hit" content="number" data-doc="The hit's id: a new value draws the word afresh (each note judged), so a pack's entrance animation on the plate replays.">
<meta name="prop:combo" content="number" data-doc="The running combo as the game counts it; 0 hides it.">
<meta name="prop:broken" content="bool" data-doc="The combo just broke: Combo broken shows in its place.">
<meta name="fixture:default" content='{"judgement":"perfect","label":"","timing":"","hit":1,"combo":128,"broken":false}'>
<meta name="fixture:great" content='{"judgement":"great","label":"","timing":"early","hit":2,"combo":129,"broken":false}'>
<meta name="fixture:good" content='{"judgement":"good","label":"","timing":"late","hit":3,"combo":130,"broken":false}'>
<meta name="fixture:miss" content='{"judgement":"miss","label":"","timing":"","hit":4,"combo":0,"broken":true}'>
<meta name="fixture:none" content='{"judgement":"none","label":"","timing":"","hit":0,"combo":0,"broken":false}'>
<meta name="fixture:overflow" content='{"judgement":"perfect","label":"Transcendent Critical Perfect","timing":"late","hit":5,"combo":12345,"broken":false}'>

<!--
  The judgement of the last hit, the word drawn afresh for each new hit id
  (a pack animates its entrance on the plate, with a reduced-motion path),
  Early or Late beside it, and the combo under it (Combo broken when it
  breaks). The game owns timing windows, judging and counting; this only
  shows what it was told. Read only; not a live region, since a hit's
  word changes many times a second.
-->
<div id="RhythmJudgement" part="RhythmJudgement.frame" class="bare" role="group" aria-label={t("rhythm.judge.name")} data-judgement={judgement}>
  {#if judgement != 'none'}
    {#key hit}
    <div part="RhythmJudgement.plate" data-judgement={judgement} data-timing={timing}>
      <span part="RhythmJudgement.title" class="word">{label != '' ? label : (judgement == 'perfect' ? t("rhythm.judge.perfect") : (judgement == 'great' ? t("rhythm.judge.great") : (judgement == 'good' ? t("rhythm.judge.good") : (judgement == 'bad' ? t("rhythm.judge.bad") : t("rhythm.judge.miss")))))}</span>
      {#if timing != ''}<span part="RhythmJudgement.badge" class="timing" data-timing={timing}>{timing == 'early' ? t("rhythm.judge.early") : t("rhythm.judge.late")}</span>{/if}
    </div>
    {/key}
  {/if}
  {#if combo > 0 || broken}
  <div part="RhythmJudgement.combo RhythmJudgement.figure" class="combo" data-broken={broken}>
    {#if !broken}<span class="count">{formatNumber(combo)}</span>{/if}
    <span class="combo-word">{broken ? t("rhythm.judge.broken") : t("rhythm.judge.combo")}</span>
  </div>
  {/if}
</div>

<style>
/* Drawn over the scene: the style's tag halo holds its text there. */
:host { text-shadow: var(--tag-halo); }
:host { display: flex; flex-direction: column; inline-size: stretch; align-items: center; gap: var(--space-sm); pointer-events: none; }
[part="RhythmJudgement.plate"] { display: flex; max-width: stretch; align-items: baseline; justify-content: center; gap: var(--space-xs); padding: var(--space-xxs) var(--space-lg); }
.word { min-width: 0; font: var(--text-xl3); font-weight: var(--font-weight-bold); text-align: center; white-space: nowrap; text-overflow: ellipsis; overflow: hidden; }
.timing { flex: none; font: var(--text-sm); white-space: nowrap; padding: var(--space-xxs) var(--space-xs); }
[part="RhythmJudgement.combo RhythmJudgement.figure"] { display: flex; flex-direction: column; align-items: center; }
.count { font: var(--text-xl4); font-weight: var(--font-weight-bold); white-space: nowrap; font-variant-numeric: tabular-nums; }
.combo-word { font: var(--text-sm); white-space: nowrap; color: var(--ink-muted); }
</style>
