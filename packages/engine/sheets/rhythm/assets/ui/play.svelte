<meta name="component" content="PlayPage">

<!-- The play page, as the approved HUD mockup lays it out over a stage: the
     song's progress along the top edge, the song plate top left, the chain
     top centre, the score top right, life and fever at the edges, the combo
     and the judgement in the centre, the tally and timing bottom left and
     the lane keys along the foot. The art is sample art of the sheet only. -->
<div id="Page">
  <img id="Stage" src={$stage} alt="">
  <div id="Progress"><RhythmSongProgress value={$songProgress_default.value} elapsed={$songProgress_default.elapsed} length={$songProgress_default.length} /></div>
  <div id="Hud">
    <div id="Top">
      <div id="Song"><RhythmSongPlate title={$songPlate_default.title} artist={$songPlate_default.artist} bpm={$songPlate_default.bpm} jacket={$jacket} tier={$songPlate_default.tier} chart={$songPlate_default.chart} rating={$songPlate_default.rating} modifiers={$songPlate_default_modifiers} /></div>
      <div id="Chain"><RhythmChain multiplier={$chain_default.multiplier} step={$chain_default.step} steps={$chain_default.steps} hit={$chain_default.hit} /></div>
      <div id="Score"><RhythmScoreHud score={$scoreHud_default.score} accuracy={$scoreHud_default.accuracy} grade={$scoreHud_default.grade} best={$scoreHud_default.best} /></div>
    </div>
    <div id="Mid">
      <div id="Life"><RhythmGauge kind={$gauge_default.kind} value={$gauge_default.value} side={$gauge_default.side} danger={$gauge_default.danger} failed={$gauge_default.failed} ready={$gauge_default.ready} key={$gauge_default.key} /></div>
      <div id="Judge"><RhythmJudgement judgement={$judgement_default.judgement} label={$judgement_default.label} timing={$judgement_default.timing} offset={$judgement_default.offset} hit={$judgement_default.hit} combo={$judgement_default.combo} broken={$judgement_default.broken} mark={$judgement_default.mark} /></div>
      <div id="Fever"><RhythmGauge kind={$gauge_fever.kind} value={$gauge_fever.value} side={$gauge_fever.side} danger={$gauge_fever.danger} failed={$gauge_fever.failed} ready={$gauge_fever.ready} key={$gauge_fever.key} /></div>
    </div>
    <div id="Foot">
      <div id="Timing"><RhythmTiming counts={$timing_default_counts} hits={$timing_default_hits} /></div>
      <div id="Keys"><RhythmLaneKeys lanes={$laneKeys_default_lanes} /></div>
    </div>
  </div>
</div>

<style>
#Page { position: relative; width: stretch; height: stretch; overflow: hidden; background: #05070c; }
#Stage { position: absolute; left: 0; top: 0; width: stretch; height: stretch; object-fit: cover; }
#Progress { position: absolute; left: 0; right: 0; top: 0; }
#Hud { position: relative; display: flex; flex-direction: column; width: stretch; height: stretch; gap: var(--space-md); padding: calc(env(safe-area-inset-top) + var(--space-xl)) var(--space-xxl) calc(env(safe-area-inset-bottom) + var(--space-lg)); }
#Top { display: flex; width: stretch; align-items: flex-start; gap: var(--space-lg); flex: none; }
#Song { flex: 1; min-width: 0; display: flex; }
#Chain { flex: none; display: flex; justify-content: center; padding-top: var(--space-xs); }
#Score { flex: 1; min-width: 0; display: flex; justify-content: flex-end; }
#Mid { display: flex; flex: 1; min-height: 0; width: stretch; align-items: center; gap: var(--space-lg); padding: var(--space-lg) var(--space-xxl); }
#Life, #Fever { flex: none; display: flex; height: 60%; }
#Judge { flex: 1; min-width: 0; display: flex; justify-content: center; }
#Foot { display: flex; width: stretch; align-items: flex-end; gap: var(--space-xl); flex: none; }
#Timing { flex: none; display: flex; }
#Keys { flex: 1; min-width: 0; display: flex; }
@media (width < 600px) {
  #Hud { gap: var(--space-sm); padding: calc(env(safe-area-inset-top) + var(--space-md)) var(--space-md) calc(env(safe-area-inset-bottom) + var(--space-md)); }
  #Top { flex-wrap: wrap; gap: var(--space-sm); }
  #Song { flex: 1 1 100%; }
  #Score { flex: 1 1 100%; }
  #Chain { flex: 1 1 100%; justify-content: flex-start; padding-top: 0; }
  #Mid { padding: var(--space-sm) 0; gap: var(--space-sm); }
  #Life, #Fever { height: 50%; }
  #Foot { flex-direction: column; align-items: stretch; gap: var(--space-md); }
}
</style>
