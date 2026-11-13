<meta name="component" content="RhythmScoreHud">
<meta name="strings" content="./rhythm.en.ogt">
<meta name="parts" content="RhythmScoreHud.frame RhythmScoreHud.cell RhythmScoreHud.title RhythmScoreHud.figure RhythmScoreHud.row RhythmScoreHud.badge">
<meta name="prop:score" content="number" data-doc="The score as the game counts it, shown with grouping.">
<meta name="prop:accuracy" content="string" data-doc="The accuracy as the game words it (98.73%); empty hides it.">
<meta name="prop:life" content="number" data-doc="The life gauge, 0 to 1; negative hides it (a mode without one).">
<meta name="prop:danger" content="bool" data-doc="The game says life is low: the gauge turns to its danger tone and Danger shows beside it.">
<meta name="prop:failed" content="bool" data-doc="The game says the run is failed (life gone): Failed shows and the frame takes data-failed.">
<meta name="prop:progress" content="number" data-doc="How far through the song, 0 to 1 (a ProgressBar).">
<meta name="fixture:default" content='{"score":482310,"accuracy":"97.42%","life":0.78,"danger":false,"failed":false,"progress":0.46}'>
<meta name="fixture:danger" content='{"score":215880,"accuracy":"81.05%","life":0.14,"danger":true,"failed":false,"progress":0.62}'>
<meta name="fixture:failed" content='{"score":198400,"accuracy":"76.30%","life":0,"danger":true,"failed":true,"progress":0.71}'>
<meta name="fixture:nolife" content='{"score":0,"accuracy":"","life":-1,"danger":false,"failed":false,"progress":0}'>
<meta name="fixture:overflow" content='{"score":123456789,"accuracy":"100.0000%","life":1,"danger":false,"failed":false,"progress":1}'>

<!--
  The run's figures over the play field: score, accuracy, the life gauge
  with Danger and Failed in words, and the song's progress. The game owns
  every figure; read only.
-->
<section id="RhythmScoreHud" part="RhythmScoreHud.frame" class="hud" aria-label={t("rhythm.hud.name")} data-danger={danger} data-failed={failed}>
  <div class="cells">
    <div part="RhythmScoreHud.cell" class="score" role="group" aria-label={t("rhythm.hud.score")}>
      <span part="RhythmScoreHud.title" class="caption">{t("rhythm.hud.score")}</span>
      <span part="RhythmScoreHud.figure" class="figure">{formatNumber(score)}</span>
    </div>
    <div part="RhythmScoreHud.cell" class="accuracy" role="group" aria-label={t("rhythm.hud.accuracy")} hidden={accuracy == ''}>
      <span part="RhythmScoreHud.title" class="caption">{t("rhythm.hud.accuracy")}</span>
      <span part="RhythmScoreHud.figure" class="figure">{accuracy}</span>
    </div>
  </div>
  <div part="RhythmScoreHud.row" class="life" hidden={life < 0} data-danger={danger} data-failed={failed}>
    <div class="gauge"><ProgressBar value={life} buffer={0} label={t("rhythm.hud.life")} valueText="" showValue={false} indeterminate={false} tone={danger || failed ? 'danger' : 'success'} /></div>
    <span part="RhythmScoreHud.badge" data-tag="danger" hidden={!danger || failed}>{t("rhythm.hud.danger")}</span>
    <span part="RhythmScoreHud.badge" data-tag="failed" hidden={!failed}>{t("rhythm.hud.failed")}</span>
  </div>
  <div part="RhythmScoreHud.row" class="progress"><ProgressBar value={progress} buffer={0} label={t("rhythm.hud.progress")} valueText="" showValue={false} indeterminate={false} tone="info" /></div>
</section>

<style>
[part="RhythmScoreHud.frame"][hidden], [part="RhythmScoreHud.frame"] [hidden] { display: none; }
[part="RhythmScoreHud.frame"] { display: flex; flex-direction: column; min-width: calc(var(--size-xxl) * 3.2); max-width: stretch; gap: var(--space-xs); padding: var(--space-sm) var(--space-md); }
.cells { display: flex; flex-wrap: wrap; width: stretch; gap: var(--space-xxs) var(--space-lg); align-items: flex-end; }
[part="RhythmScoreHud.cell"] { display: flex; flex-direction: column; align-items: flex-start; }
[part="RhythmScoreHud.cell"].score { flex: 1; }
[part="RhythmScoreHud.cell"].accuracy { flex: none; align-items: flex-end; }
/* A figure is never cut: the frame grows with it (a score is read whole). */
[part="RhythmScoreHud.figure"] { white-space: nowrap; font-variant-numeric: tabular-nums; }
[part="RhythmScoreHud.row"] { display: flex; width: stretch; align-items: flex-end; gap: var(--space-xs); }
.gauge { flex: 1; min-width: 0; }
[part="RhythmScoreHud.badge"] { flex: none; white-space: nowrap; padding: 0 var(--space-xs); }
@media (width < 600px) {
  [part="RhythmScoreHud.frame"] { width: stretch; min-width: 0; padding: var(--space-xs) var(--space-sm); }
  .cells { gap: var(--space-sm); }
}
</style>
