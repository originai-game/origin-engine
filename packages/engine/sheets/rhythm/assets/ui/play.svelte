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
    <div id="Song"><RhythmSongPlate title={$songPlate_default.title} artist={$songPlate_default.artist} bpm={$songPlate_default.bpm} jacket={$jacket} tier={$songPlate_default.tier} chart={$songPlate_default.chart} rating={$songPlate_default.rating} modifiers={$songPlate_default_modifiers} /></div>
    <div id="Chain"><RhythmChain multiplier={$chain_default.multiplier} step={$chain_default.step} steps={$chain_default.steps} hit={$chain_default.hit} /></div>
    <div id="Score"><RhythmScoreHud score={$scoreHud_default.score} accuracy={$scoreHud_default.accuracy} grade={$scoreHud_default.grade} best={$scoreHud_default.best} /></div>
    <div id="Life"><RhythmGauge kind={$gauge_default.kind} value={$gauge_default.value} side={$gauge_default.side} danger={$gauge_default.danger} failed={$gauge_default.failed} ready={$gauge_default.ready} key={$gauge_default.key} /></div>
    <div id="Timing"><RhythmTiming counts={$timing_default_counts} hits={$timing_default_hits} /></div>
    <div id="Judge"><RhythmJudgement judgement={$judgement_default.judgement} label={$judgement_default.label} timing={$judgement_default.timing} offset={$judgement_default.offset} hit={$judgement_default.hit} combo={$judgement_default.combo} broken={$judgement_default.broken} mark={$judgement_default.mark} /></div>
    <div id="Fever"><RhythmGauge kind={$gauge_fever.kind} value={$gauge_fever.value} side={$gauge_fever.side} danger={$gauge_fever.danger} failed={$gauge_fever.failed} ready={$gauge_fever.ready} key={$gauge_fever.key} /></div>
    <div id="Keys"><RhythmLaneKeys lanes={$laneKeys_default_lanes} /></div>
  </div>
</div>

<style>
#Page { position: relative; width: stretch; height: stretch; overflow: hidden; background: #05070c; }
#Stage { position: absolute; left: 0; top: 0; width: stretch; height: stretch; object-fit: cover; }
#Progress { position: absolute; left: 0; right: 0; top: 0; }
/* The HUD keeps the hit band clear: nothing but the lane keys sits on or
   under the judgement line (the "band" row, a share of the screen's
   height where the stage's line falls). The tally and timing stand under
   the life gauge on a wide screen and at the top on a phone. */
#Hud { position: relative; display: grid; width: stretch; height: stretch; box-sizing: border-box; column-gap: var(--space-lg); row-gap: var(--space-md);
  grid-template-columns: auto minmax(0, 1fr) auto minmax(0, 1fr) auto;
  grid-template-rows: auto minmax(0, 1fr) 13% auto;
  grid-template-areas: "song song chain score score" "life timing judge . fever" "band band band band band" "keys keys keys keys keys";
  padding: calc(env(safe-area-inset-top) + var(--space-xl)) var(--space-xxl) calc(env(safe-area-inset-bottom) + var(--space-md)); }
#Song { grid-area: song; display: flex; min-width: 0; }
#Chain { grid-area: chain; display: flex; justify-content: center; padding-top: var(--space-xs); }
#Score { grid-area: score; display: flex; justify-content: flex-end; min-width: 0; }
#Life { grid-area: life; display: flex; min-height: 0; }
#Timing { grid-area: timing; display: flex; align-self: end; min-width: 0; }
/* A size container: the combo and judgement figures size to this cell. */
#Judge { grid-area: judge; display: flex; align-items: center; justify-content: center; min-width: 0; min-height: 0; container-type: size; }
#Fever { grid-area: fever; display: flex; justify-content: flex-end; min-height: 0; }
#Keys { grid-area: keys; display: flex; min-width: 0; }
#Hud:size(compact) { column-gap: var(--space-sm); row-gap: var(--space-sm);
    grid-template-columns: auto minmax(0, 1fr) auto;
    grid-template-rows: auto auto auto auto minmax(0, 1fr) 11% auto;
    grid-template-areas: "song song song" "score score score" "chain chain chain" "timing timing timing" "life judge fever" "band band band" "keys keys keys";
    padding: calc(env(safe-area-inset-top) + var(--space-md)) var(--space-md) calc(env(safe-area-inset-bottom) + var(--space-sm)); }
#Chain:size(compact) { justify-content: flex-start; padding-top: 0; }
#Timing:size(compact) { align-self: auto; }
</style>
