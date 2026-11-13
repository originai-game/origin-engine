<meta name="component" content="RacingLapHud">
<meta name="strings" content="./racing.en.ogt">
<meta name="parts" content="RacingLapHud.frame RacingLapHud.cell RacingLapHud.title RacingLapHud.row RacingLapHud.delta RacingLapHud.badge">
<meta name="prop:lap" content="number" data-doc="The lap the player is on, from 1; 0 before the start shows a dash.">
<meta name="prop:laps" content="number" data-doc="Laps in the race; 0 hides the lap count (an open session).">
<meta name="prop:position" content="number" data-doc="The player's place, from 1; 0 shows a dash.">
<meta name="prop:racers" content="number" data-doc="Cars in the race; 0 hides the position.">
<meta name="prop:current" content="string" data-doc="The lap time running now, as the game formats it (1:02.418); empty shows the blank time.">
<meta name="prop:best" content="string" data-doc="The player's best lap, formatted; empty shows the blank time.">
<meta name="prop:last" content="string" data-doc="The last lap, formatted; empty shows the blank time.">
<meta name="prop:split" content="string" data-doc="The split delta as the game words it (-0.412, +1.204); empty hides it.">
<meta name="prop:splitState" content="string" data-doc="ahead, behind or even: the delta's sense, named in words beside it and on data-state.">
<meta name="prop:freshBest" content="bool" data-doc="The last lap was a new best: the Best lap badge shows on the last row.">
<meta name="prop:finalLap" content="bool" data-doc="The player is on the final lap: the Final lap badge shows on the lap cell.">
<meta name="prop:timeLeft" content="number" data-doc="Seconds left to the next checkpoint or the session's end (HudTimer); negative hides it.">
<meta name="prop:timeTotal" content="number" data-doc="The checkpoint's whole time, for the timer's ring; 0 draws none.">
<meta name="prop:warnAt" content="number" data-doc="Seconds left when the timer turns warn (HudTimer's warnAt); 10 when left out.">
<script lang="ts">let { warnAt = 10 }: { warnAt?: number } = $props();</script>
<meta name="fixture:default" content='{"lap":2,"laps":3,"position":3,"racers":8,"current":"0:48.207","best":"1:21.904","last":"1:23.116","split":"-0.412","splitState":"ahead","freshBest":false,"finalLap":false,"timeLeft":-1,"timeTotal":0}'>
<meta name="fixture:grid" content='{"lap":0,"laps":3,"position":0,"racers":8,"current":"","best":"","last":"","split":"","splitState":"even","freshBest":false,"finalLap":false,"timeLeft":-1,"timeTotal":0}'>
<meta name="fixture:finalLap" content='{"lap":3,"laps":3,"position":1,"racers":8,"current":"0:12.660","best":"1:20.871","last":"1:20.871","split":"+1.204","splitState":"behind","freshBest":true,"finalLap":true,"timeLeft":-1,"timeTotal":0}'>
<meta name="fixture:checkpoint" content='{"lap":4,"laps":0,"position":0,"racers":0,"current":"3:41.090","best":"0:55.020","last":"0:57.311","split":"0.000","splitState":"even","freshBest":false,"finalLap":false,"timeLeft":8.4,"timeTotal":30}'>
<meta name="fixture:overflow" content='{"lap":118,"laps":150,"position":24,"racers":24,"current":"1:02:13.456","best":"1:58:59.999","last":"10:21:04.120","split":"+12:31.877","splitState":"behind","freshBest":true,"finalLap":true,"timeLeft":-1,"timeTotal":0}'>

<!--
  The race's own clock and place: lap n of N, position p of N, the lap
  time running, the best and last laps and the split delta, ahead or
  behind. Every time comes formatted from the game; read only.
-->
<section id="RacingLapHud" part="RacingLapHud.frame" class="hud" aria-label={t("racing.lap.name")} data-final={finalLap}>
  <div class="cells">
    <div part="RacingLapHud.cell" class="lap" hidden={!(laps > 0) && !(lap > 0)} role="group" aria-label={laps > 0 && lap > 0 ? t("racing.lap.lapValue", { lap: lap, laps: laps }) : t("racing.lap.lap")} data-final={finalLap}>
      <span part="RacingLapHud.title" class="caption">{t("racing.lap.lap")}</span>
      <div class="figure"><span class="now">{lap > 0 ? formatNumber(lap) : t("racing.lap.pending")}</span><span class="of" hidden={!(laps > 0)}>{t("racing.lap.of", { total: laps })}</span></div>
      <span part="RacingLapHud.badge" class="final" hidden={!finalLap}>{t("racing.lap.finalLap")}</span>
    </div>
    <div part="RacingLapHud.cell" class="place" hidden={!(racers > 0)} role="group" aria-label={position > 0 ? t("racing.lap.positionValue", { position: position, racers: racers }) : t("racing.lap.position")}>
      <span part="RacingLapHud.title" class="caption">{t("racing.lap.position")}</span>
      <div class="figure"><span class="now">{position > 0 ? formatNumber(position) : t("racing.lap.pending")}</span><span class="of">{t("racing.lap.of", { total: racers })}</span></div>
    </div>
  </div>
  <div class="times" role="list" aria-label={t("racing.lap.name")}>
    <div part="RacingLapHud.row" class="current" role="listitem"><span class="label">{t("racing.lap.current")}</span><span class="time">{current != '' ? current : t("racing.lap.none")}</span></div>
    <div part="RacingLapHud.delta" class="split" hidden={split == ''} role="listitem" data-state={splitState}><span class="sense">{splitState == 'ahead' ? t("racing.lap.ahead") : (splitState == 'behind' ? t("racing.lap.behind") : t("racing.lap.even"))}</span><span class="time">{split}</span></div>
    <div part="RacingLapHud.row" class="best" role="listitem"><span class="label">{t("racing.lap.best")}</span><span class="time">{best != '' ? best : t("racing.lap.none")}</span></div>
    <div part="RacingLapHud.row" class="last" role="listitem" data-fresh={freshBest}><span class="label">{t("racing.lap.last")}</span><span class="time">{last != '' ? last : t("racing.lap.none")}</span><span part="RacingLapHud.badge" class="fresh" hidden={!freshBest}>{t("racing.lap.newBest")}</span></div>
  </div>
  <div class="clock" hidden={timeLeft < 0}><HudTimer seconds={timeLeft < 0 ? 0 : timeLeft} total={timeTotal} countdown={true} warnAt={warnAt} paused={false} label={t("racing.lap.timeLeft")} /></div>
</section>

<style>
[part="RacingLapHud.frame"][hidden], [part="RacingLapHud.frame"] [hidden] { display: none; }
[part="RacingLapHud.frame"] { display: flex; flex-direction: column; width: fit-content; max-width: stretch; gap: var(--space-xs); padding: var(--space-sm) var(--space-md); }
.cells { display: flex; gap: var(--space-md); align-items: flex-end; }
[part="RacingLapHud.cell"] { display: flex; flex-direction: column; gap: 0; align-items: flex-start; min-width: 0; }
.caption { white-space: nowrap; }
.figure { display: flex; align-items: baseline; gap: var(--space-xxs); white-space: nowrap; }
.now, .of, .time { font-variant-numeric: tabular-nums; white-space: nowrap; }
.final { white-space: nowrap; padding: 0 var(--space-xs); }
.times { display: flex; flex-direction: column; gap: var(--space-xxs); min-width: 0; }
[part="RacingLapHud.row"], [part="RacingLapHud.delta"] { display: flex; align-items: baseline; gap: var(--space-sm); min-width: 0; }
.label, .sense { flex: none; white-space: nowrap; min-width: calc(var(--size-xl) * 0.9); }
.time { min-width: 0; overflow: hidden; text-overflow: ellipsis; }
.fresh { flex: none; white-space: nowrap; padding: 0 var(--space-xs); }
.clock { align-self: flex-start; }
@media (width < 600px) {
  [part="RacingLapHud.frame"] { padding: var(--space-xs) var(--space-sm); }
  .cells { gap: var(--space-sm); }
  .label, .sense { min-width: var(--size-xl); }
}
</style>
