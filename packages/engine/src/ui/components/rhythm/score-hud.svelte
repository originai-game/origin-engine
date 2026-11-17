<meta name="component" content="RhythmScoreHud">
<meta name="strings" content="./rhythm.en.ogt">
<meta name="parts" content="RhythmScoreHud.frame RhythmScoreHud.legend RhythmScoreHud.score RhythmScoreHud.accuracy RhythmScoreHud.grade RhythmScoreHud.best RhythmScoreHud.delta">
<meta name="prop:score" content="number" data-doc="The score as the game counts it, shown with grouping.">
<meta name="prop:accuracy" content="string" data-doc="The accuracy as the game words it (98.42%); empty hides it.">
<meta name="prop:grade" content="string" data-doc="The grade the run is on course for as the game names it (S, AA); empty hides it.">
<meta name="prop:best" content="number" data-default="0" data-doc="The player's best on this chart; 0 hides it. Ahead or behind it shows as a signed gap beside it (on data-trend).">
<meta name="fixture:default" content='{"score":987450,"accuracy":"98.42%","grade":"S","best":991208}'>
<meta name="fixture:ahead" content='{"score":482310,"accuracy":"97.42%","grade":"AA","best":455000}'>
<meta name="fixture:first" content='{"score":215880,"accuracy":"81.05%","grade":"B","best":0}'>
<meta name="fixture:start" content='{"score":0,"accuracy":"","grade":"","best":0}'>
<meta name="fixture:overflow" content='{"score":123456789,"accuracy":"100.0000%","grade":"SSS+","best":123450000}'>

<!--
  The run's figures over the play field, set against the screen's end:
  the score as the hero figure, the accuracy with the grade it is on course
  for, and the player's best with the gap to it. The game owns every
  figure; read only. Pair it with RhythmGauge for life and fever and
  RhythmSongProgress for the song's progress.
-->
<section id="RhythmScoreHud" part="RhythmScoreHud.frame" class="hud bare" data-layout="stack" data-gap="none" role="group" aria-label={t("rhythm.hud.name")}>
  <span part="RhythmScoreHud.legend" class="caption">{t("rhythm.hud.score")}</span>
  <span part="RhythmScoreHud.score" class="score">{formatNumber(score)}</span>
  {#if accuracy != '' || grade != ''}
  <div class="line" data-layout="row" data-gap="tight">
    {#if accuracy != ''}<span part="RhythmScoreHud.accuracy" class="accuracy" aria-label={t("rhythm.hud.accuracyName", { accuracy: accuracy })}>{accuracy}</span>{/if}
    {#if grade != ''}<span part="RhythmScoreHud.grade" class="grade" aria-label={t("rhythm.hud.gradeName", { grade: grade })}>{grade}</span>{/if}
  </div>
  {/if}
  {#if best > 0}
  <div class="line best" data-layout="row" data-gap="tight">
    <span part="RhythmScoreHud.legend" class="caption">{t("rhythm.hud.best")}</span>
    <span part="RhythmScoreHud.best" class="best-figure">{formatNumber(best)}</span>
    <span part="RhythmScoreHud.delta" class="delta" data-trend={score >= best ? 'up' : 'down'}>{score >= best ? t("rhythm.hud.ahead", { gap: score - best }) : t("rhythm.hud.behind", { gap: best - score })}</span>
  </div>
  {/if}
</section>

<style>
/* Text that stands straight over the scene: one stroke under the fill and
   one soft shadow, never a plate. */
:host { align-items: flex-end; max-inline-size: stretch; color: var(--hud-ink); text-shadow: var(--hud-shadow); -webkit-text-stroke-width: var(--hud-stroke-width); -webkit-text-stroke-color: var(--hud-stroke-color); paint-order: stroke fill; }
.caption { font-size: var(--hud-label-size); font-weight: var(--hud-label-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); color: var(--hud-label-ink); white-space: nowrap; }
.score { font-size: var(--hud-size-score); line-height: 0.9; font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); font-variant-numeric: var(--hud-figure-numeric); white-space: nowrap; }
.line { align-items: baseline; justify-content: flex-end; margin-block-start: var(--space-xs); }
.accuracy { font-size: var(--hud-size-md); font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); font-variant-numeric: var(--hud-figure-numeric); white-space: nowrap; }
.grade { font-size: var(--hud-size-lg); line-height: 1; font-weight: var(--hud-figure-weight); font-style: var(--judge-slant); color: var(--hud-gold); filter: var(--judge-perfect-glow); white-space: nowrap; }
.best { margin-block-start: var(--space-xxs); }
.best-figure { font-size: var(--hud-size-sm); font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); font-variant-numeric: var(--hud-figure-numeric); color: var(--hud-ink-dim); white-space: nowrap; }
/* The gap to the best on a tag plate: small figures never take a stroke. */
.delta { flex: none; padding: 0 var(--space-xs); font-size: var(--hud-tag-size); font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); font-variant-numeric: var(--hud-figure-numeric); white-space: nowrap; background: var(--hud-tag-fill); box-shadow: var(--hud-tag-rim); -webkit-text-stroke-width: 0; text-shadow: none; clip-path: polygon(var(--hud-tag-cut) 0, 100% 0, 100% calc(100% - var(--hud-tag-cut)), calc(100% - var(--hud-tag-cut)) 100%, 0 100%, 0 var(--hud-tag-cut)); }
.delta[data-trend=up] { color: var(--trend-up); }
.delta[data-trend=down] { color: var(--trend-down); }
@media (width < 600px) {
  .score { font-size: var(--hud-size-score-compact); }
  .accuracy { font-size: var(--hud-size-sm); }
  .grade { font-size: var(--hud-size-md); }
  .line { margin-block-start: var(--space-xxs); }
}
</style>
