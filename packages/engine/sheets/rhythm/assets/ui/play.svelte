<meta name="component" content="PlayPage">

<!-- The stress page: the judgement over the lanes' hit line and the score HUD
     in the corner, on a bright sky over dark lanes so each readout is reviewed
     on both (docs/ui.md §6.3), beside the long results screen (the overflow
     fixture: a long title, a large score, eight tally rows). On a phone they
     stack, the results under the play field. -->
<div id="Page">
  <div id="Field">
    <div id="Sky"></div>
    <div id="Lanes"></div>
    <div id="Hud"><RhythmScoreHud score={$scoreHud_danger.score} accuracy={$scoreHud_danger.accuracy} life={$scoreHud_danger.life} danger={$scoreHud_danger.danger} failed={$scoreHud_danger.failed} progress={$scoreHud_danger.progress} /></div>
    <div id="Judge"><RhythmJudgement judgement={$judgement_default.judgement} label={$judgement_default.label} timing={$judgement_default.timing} hit={$judgement_default.hit} combo={$judgement_default.combo} broken={$judgement_default.broken} /></div>
  </div>
  <div id="Results"><RhythmResults song={$results_overflow.song} grade={$results_overflow.grade} cleared={$results_overflow.cleared} score={$results_overflow.score} previous={$results_overflow.previous} newBest={$results_overflow.newBest} maxCombo={$results_overflow.maxCombo} accuracy={$results_overflow.accuracy} mark={$results_overflow.mark} counts={$results_overflow_counts} canRetry={$results_overflow.canRetry} /></div>
</div>

<style>
#Page { display: flex; width: stretch; height: stretch; gap: var(--space-xl); padding: var(--space-xl); }
#Field { position: relative; flex: 1; min-width: 0; height: stretch; overflow: hidden; border-radius: var(--radius-sm); }
#Sky { position: absolute; left: 0; top: 0; width: stretch; height: 40%; background: linear-gradient(180deg, #8fc3ea, #dfe9ef); }
#Lanes { position: absolute; left: 0; bottom: 0; width: stretch; height: 60%; background: linear-gradient(180deg, #2a2440, #0c0b12); }
#Hud { position: absolute; left: var(--space-md); top: var(--space-md); right: var(--space-md); display: flex; }
#Judge { position: absolute; left: 0; right: 0; top: 52%; }
#Results { flex: 1; min-width: 0; height: stretch; }
@media (width < 600px) {
  #Page { flex-direction: column; gap: var(--space-sm); padding: var(--space-sm); }
  #Field { flex: none; width: stretch; height: 42%; }
  #Hud { left: var(--space-xs); top: var(--space-xs); right: var(--space-xs); }
  #Results { flex: 1; min-height: 0; width: stretch; height: auto; }
}
</style>
