<meta name="component" content="RacingLapHud">
<meta name="strings" content="./racing.en.ogt">
<meta name="parts" content="RacingLapHud.frame RacingLapHud.readout RacingLapHud.title RacingLapHud.row RacingLapHud.delta RacingLapHud.badge RacingLapHud.figure RacingLapHud.caption">
<meta name="prop:lap" content="number" data-doc="The lap the player is on, from 1; 0 before the start shows a dash.">
<meta name="prop:laps" content="number" data-doc="Laps in the race; 0 hides the lap count (an open session).">
<meta name="prop:position" content="number" data-doc="The player's place, from 1; 0 shows a dash.">
<meta name="prop:racers" content="number" data-doc="Cars in the race; 0 hides the position.">
<meta name="prop:current" content="string" data-doc="The lap time running now, as the game formats it (1:02.418); empty shows the blank time.">
<meta name="prop:best" content="string" data-doc="The player's best lap, formatted; empty shows the blank time.">
<meta name="prop:last" content="string" data-doc="The last lap, formatted; empty shows the blank time.">
<meta name="prop:split" content="string" data-doc="The split delta as the game words it (-0.412, +1.204); empty hides it.">
<meta name="prop:splitState" content="string" data-default="even" data-doc="ahead, behind or even: the delta's sense, named in words beside it and on data-state.">
<meta name="prop:freshBest" content="bool" data-doc="The last lap was a new best: the Best lap badge shows on the last row.">
<meta name="prop:finalLap" content="bool" data-doc="The player is on the final lap: the Final lap badge shows on the lap cell.">
<meta name="prop:timeLeft" content="number" data-default="-1" data-doc="Seconds left to the next checkpoint or the session's end (HudTimer); negative hides it.">
<meta name="prop:timeTotal" content="number" data-doc="The checkpoint's whole time, for the timer's ring; 0 draws none.">
<meta name="prop:warnAt" content="number" data-default="10" data-doc="Seconds left when the timer turns warn (HudTimer's warnAt); 10 when left out.">
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
<section id="RacingLapHud" part="RacingLapHud.frame" class="hud" data-layout="stack" aria-label={t("racing.lap.name")} data-final={finalLap}>
  <div class="cells" data-layout="row" data-gap="loose">
    {#if laps > 0 || lap > 0}<div part="RacingLapHud.readout RacingLapHud.figure" class="lap" data-layout="stack" data-gap="none" role="group" aria-label={laps > 0 && lap > 0 ? t("racing.lap.lapValue", { lap: lap, laps: laps }) : t("racing.lap.lap")} data-final={finalLap}>
      <span part="RacingLapHud.title RacingLapHud.caption" class="caption">{t("racing.lap.lap")}</span>
      <div class="figure" data-layout="row" data-gap="hair"><span class="now">{lap > 0 ? formatNumber(lap) : t("racing.lap.pending")}</span>{#if laps > 0}<span class="of">{t("racing.lap.of", { total: laps })}</span>{/if}</div>
      {#if finalLap}<span part="RacingLapHud.badge" class="final">{t("racing.lap.finalLap")}</span>{/if}
    </div>{/if}
    {#if racers > 0}<div part="RacingLapHud.readout RacingLapHud.figure" class="place" data-layout="stack" data-gap="none" role="group" aria-label={position > 0 ? t("racing.lap.positionValue", { position: position, racers: racers }) : t("racing.lap.position")}>
      <span part="RacingLapHud.title RacingLapHud.caption" class="caption">{t("racing.lap.position")}</span>
      <div class="figure" data-layout="row" data-gap="hair"><span class="now">{position > 0 ? formatNumber(position) : t("racing.lap.pending")}</span><span class="of">{t("racing.lap.of", { total: racers })}</span></div>
    </div>{/if}
  </div>
  <div class="times" data-layout="stack" data-gap="hair" role="list" aria-label={t("racing.lap.name")}>
    <div part="RacingLapHud.row" data-layout="row" class="current" role="listitem"><span class="label" part="RacingLapHud.caption">{t("racing.lap.current")}</span><span class="time" part="RacingLapHud.figure" data-fill>{current != '' ? current : t("racing.lap.none")}</span></div>
    {#if split != ''}<div part="RacingLapHud.delta RacingLapHud.figure" data-layout="row" class="split" role="listitem" data-state={splitState} data-trend={splitState == 'ahead' ? 'up' : splitState == 'behind' ? 'down' : null}><span class="sense">{splitState == 'ahead' ? t("racing.lap.ahead") : (splitState == 'behind' ? t("racing.lap.behind") : t("racing.lap.even"))}</span><span class="time" data-fill>{split}</span></div>{/if}
    <div part="RacingLapHud.row" data-layout="row" class="best" role="listitem"><span class="label" part="RacingLapHud.caption">{t("racing.lap.best")}</span><span class="time" part="RacingLapHud.figure" data-fill>{best != '' ? best : t("racing.lap.none")}</span></div>
    <div part="RacingLapHud.row" data-layout="row" class="last" role="listitem" data-fresh={freshBest}><span class="label" part="RacingLapHud.caption">{t("racing.lap.last")}</span><span class="time" part="RacingLapHud.figure" data-fill>{last != '' ? last : t("racing.lap.none")}</span>{#if freshBest}<span part="RacingLapHud.badge" class="fresh">{t("racing.lap.newBest")}</span>{/if}</div>
  </div>
  {#if timeLeft >= 0}<div class="clock"><HudTimer seconds={timeLeft} total={timeTotal} countdown={true} warnAt={warnAt} paused={false} label={t("racing.lap.timeLeft")} /></div>{/if}
</section>

<style>
:host { width: fit-content; max-width: 100%; padding: var(--space-sm) var(--space-md); }
.cells { align-items: flex-end; }
[part="RacingLapHud.readout RacingLapHud.figure"] { align-items: flex-start; }
.caption { white-space: nowrap; }
.figure { align-items: baseline; white-space: nowrap; }
.now, .of, .time { white-space: nowrap; }
.final { white-space: nowrap; padding: 0 var(--space-xs); }
[part="RacingLapHud.row"], [part="RacingLapHud.delta RacingLapHud.figure"] { align-items: baseline; }
.label, .sense { flex: none; white-space: nowrap; min-width: calc(var(--size-xl) * 0.9); }
.time { overflow: hidden; text-overflow: ellipsis; }
.fresh { flex: none; white-space: nowrap; padding: 0 var(--space-xs); }
.clock { align-self: flex-start; }
@media (width < 600px) {
  [part="RacingLapHud.frame"] { padding: var(--space-xs) var(--space-sm); }
  .cells { --layout-gap-loose: var(--layout-gap); }
  .label, .sense { min-width: var(--size-xl); }
}
</style>
