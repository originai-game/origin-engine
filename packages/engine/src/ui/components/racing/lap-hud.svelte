<meta name="component" content="RacingLapHud">
<meta name="strings" content="./racing.en.ogt">
<meta name="parts" content="RacingLapHud.frame RacingLapHud.place RacingLapHud.of RacingLapHud.lap RacingLapHud.gained RacingLapHud.times RacingLapHud.row RacingLapHud.label RacingLapHud.time RacingLapHud.delta RacingLapHud.badge">
<meta name="prop:lap" content="number" data-doc="The lap the player is on, from 1; 0 before the start shows a dash.">
<meta name="prop:laps" content="number" data-doc="Laps in the race; 0 hides the lap's total (an open session).">
<meta name="prop:position" content="number" data-doc="The player's place, from 1; 0 shows a dash.">
<meta name="prop:racers" content="number" data-doc="Cars in the race; 0 hides the place.">
<meta name="prop:gained" content="number" data-doc="Places gained since the start as the game counts them: positive reads Up n places, negative Down n places (on data-trend), 0 hides it.">
<meta name="prop:current" content="string" data-doc="The lap time running now, as the game formats it (1:24.618); empty shows the blank time.">
<meta name="prop:best" content="string" data-doc="The player's best lap, formatted, in the best lap's violet; empty shows the blank time.">
<meta name="prop:last" content="string" data-doc="The last lap, formatted; empty shows the blank time.">
<meta name="prop:sector" content="string" data-doc="The split's name as the game words it (Sector 2); shown beside the delta.">
<meta name="prop:split" content="string" data-doc="The split delta as the game words it (−0.412, +1.204), on a green tag ahead and a red one behind; empty hides the row.">
<meta name="prop:splitState" content="string" data-default="even" data-doc="ahead, behind or even: the delta's sense, on data-state and read with it.">
<meta name="prop:freshBest" content="bool" data-doc="The last lap was a new best: a Best lap tag stands on the last row.">
<meta name="prop:finalLap" content="bool" data-doc="The player is on the final lap: the lap plate reads Final lap (data-final).">
<meta name="prop:timeLeft" content="number" data-default="-1" data-doc="Seconds left to the next checkpoint or the session's end (HudTimer); negative hides it.">
<meta name="prop:timeTotal" content="number" data-doc="The checkpoint's whole time, for the timer's ring; 0 draws none.">
<meta name="prop:warnAt" content="number" data-default="10" data-doc="Seconds left when the timer turns warn (HudTimer's warnAt); 10 when left out.">
<meta name="fixture:default" content='{"lap":2,"laps":3,"position":3,"racers":8,"gained":2,"current":"1:24.618","best":"1:57.790","last":"1:58.204","sector":"Sector 2","split":"−0.412","splitState":"ahead","freshBest":false,"finalLap":false,"timeLeft":-1,"timeTotal":0}'>
<meta name="fixture:leading" content='{"lap":3,"laps":3,"position":1,"racers":8,"gained":5,"current":"0:12.660","best":"1:20.871","last":"1:20.871","sector":"Sector 1","split":"+0.204","splitState":"behind","freshBest":true,"finalLap":true,"timeLeft":-1,"timeTotal":0}'>
<meta name="fixture:last" content='{"lap":2,"laps":5,"position":8,"racers":8,"gained":-3,"current":"0:48.207","best":"1:31.904","last":"1:33.116","sector":"Sector 3","split":"+2.950","splitState":"behind","freshBest":false,"finalLap":false,"timeLeft":-1,"timeTotal":0}'>
<meta name="fixture:grid" content='{"lap":0,"laps":3,"position":0,"racers":8,"gained":0,"current":"","best":"","last":"","sector":"","split":"","splitState":"even","freshBest":false,"finalLap":false,"timeLeft":-1,"timeTotal":0}'>
<meta name="fixture:checkpoint" content='{"lap":4,"laps":0,"position":0,"racers":0,"gained":0,"current":"3:41.090","best":"0:55.020","last":"0:57.311","sector":"Checkpoint 6","split":"0.000","splitState":"even","freshBest":false,"finalLap":false,"timeLeft":8.4,"timeTotal":30}'>
<meta name="fixture:overflow" content='{"lap":118,"laps":150,"position":24,"racers":24,"gained":-23,"current":"1:02:13.456","best":"1:58:59.999","last":"10:21:04.120","sector":"Sector 14 · Nordschleife Hatzenbach","split":"+12:31.877","splitState":"behind","freshBest":true,"finalLap":true,"timeLeft":-1,"timeTotal":0}'>

<!--
  The race's place and clock at the top left: the place large and slanted
  over the field's size, the lap on a lit plate and the places gained
  beside it; under them the lap time running, the last and best laps and
  the split delta on a green or red tag. Every figure and time comes from
  the game, formatted; read only.
-->
<section id="RacingLapHud" part="RacingLapHud.frame" class="hud bare" data-layout="stack" aria-label={t("racing.lap.name")} data-final={finalLap}>
  <div id="Head" data-layout="row">
    {#if racers > 0}<div id="Place" data-layout="row" data-gap="none" role="group" aria-label={position > 0 ? t("racing.lap.positionValue", { position: position, racers: racers }) : t("racing.lap.position")}>
      <span part="RacingLapHud.place" class="place">{position > 0 ? formatNumber(position) : t("racing.lap.pending")}</span><span part="RacingLapHud.of" class="of">{t("racing.lap.of", { total: racers })}</span>
    </div>{/if}
    <div id="Side" data-layout="stack">
      {#if laps > 0 || lap > 0}<div part="RacingLapHud.lap" id="Lap" data-layout="row" role="group" data-final={finalLap} aria-label={laps > 0 && lap > 0 ? t("racing.lap.lapValue", { lap: lap, laps: laps }) : t("racing.lap.lap")}>
        <span class="caption">{finalLap ? t("racing.lap.finalLap") : t("racing.lap.lap")}</span>
        <span class="now">{lap > 0 ? formatNumber(lap) : t("racing.lap.pending")}</span>{#if laps > 0}<span class="total">{t("racing.lap.ofSpaced", { total: laps })}</span>{/if}
      </div>{/if}
      {#if gained != 0}<div part="RacingLapHud.gained" class="gained" data-layout="row" data-trend={gained > 0 ? 'up' : 'down'}><span class="arrow"></span><span class="moved">{gained > 0 ? t("racing.lap.up", { n: gained }) : t("racing.lap.down", { n: -gained })}</span></div>{/if}
    </div>
  </div>
  <div part="RacingLapHud.times" id="Times" role="list" aria-label={t("racing.lap.times")}>
    <div part="RacingLapHud.row" class="row current" role="listitem"><span part="RacingLapHud.label" class="label">{t("racing.lap.current")}</span><span part="RacingLapHud.time" class="time">{current != '' ? current : t("racing.lap.none")}</span></div>
    <div part="RacingLapHud.row" class="row last" role="listitem" data-fresh={freshBest}><span part="RacingLapHud.label" class="label">{t("racing.lap.last")}</span><div class="end">{#if freshBest}<span part="RacingLapHud.badge" class="fresh">{t("racing.lap.newBest")}</span>{/if}<span part="RacingLapHud.time" class="time">{last != '' ? last : t("racing.lap.none")}</span></div></div>
    <div part="RacingLapHud.row" class="row best" role="listitem"><span part="RacingLapHud.label" class="label">{t("racing.lap.best")}</span><span part="RacingLapHud.time" class="time">{best != '' ? best : t("racing.lap.none")}</span></div>
    {#if split != ''}<div part="RacingLapHud.row" class="row split" role="listitem" data-state={splitState} aria-label={splitState == 'ahead' ? t("racing.lap.splitAhead", { sector: sector, delta: split }) : (splitState == 'behind' ? t("racing.lap.splitBehind", { sector: sector, delta: split }) : t("racing.lap.splitEven", { sector: sector, delta: split }))}><span part="RacingLapHud.label" class="label sector">{sector != '' ? sector : t("racing.lap.split")}</span><span part="RacingLapHud.delta" class="delta" data-state={splitState} data-trend={splitState == 'ahead' ? 'up' : splitState == 'behind' ? 'down' : null}>{split}</span></div>{/if}
  </div>
  {#if timeLeft >= 0}<div class="clock"><HudTimer seconds={timeLeft} total={timeTotal} countdown={true} warnAt={warnAt} paused={false} label={t("racing.lap.timeLeft")} /></div>{/if}
</section>

<style>
:host { row-gap: calc(18 * var(--font-step-px)); inline-size: calc(250 * var(--font-step-px)); max-inline-size: stretch; color: var(--hud-ink); text-shadow: var(--hud-shadow); -webkit-text-stroke-width: var(--hud-stroke-width); -webkit-text-stroke-color: var(--hud-stroke-color); paint-order: stroke fill; }
/* The place, large and slanted over the field's size, the lap and the
   places gained beside it. */
#Head { align-items: flex-end; column-gap: calc(16 * var(--font-step-px)); min-inline-size: 0; }
#Place { flex: none; align-items: baseline; }
.place { font-size: var(--hud-size-place); line-height: 0.8; font-style: var(--hud-figure-slant); font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); font-variant-numeric: var(--hud-figure-numeric); white-space: nowrap; }
.of { font-size: var(--hud-size-place-of); line-height: 0.8; font-style: var(--hud-figure-slant); font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); color: var(--hud-ink-dim); white-space: nowrap; }
#Side { flex: 0 1 auto; min-inline-size: 0; row-gap: calc(8 * var(--font-step-px)); padding-block-end: calc(4 * var(--font-step-px)); align-items: flex-start; }
/* The lap on smoked glass with a lit top edge in the accent. */
#Lap { flex: none; align-items: baseline; column-gap: calc(8 * var(--font-step-px)); padding: calc(3 * var(--font-step-px)) calc(9 * var(--font-step-px)); background: var(--hud-plate-fill); box-shadow: var(--hud-plate-lit-rim); clip-path: polygon(var(--hud-plate-cut) 0, 100% 0, 100% calc(100% - var(--hud-plate-cut)), calc(100% - var(--hud-plate-cut)) 100%, 0 100%, 0 var(--hud-plate-cut)); -webkit-text-stroke-width: 0; text-shadow: none; }
.caption { font-size: var(--hud-label-size); font-weight: var(--hud-label-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); color: var(--hud-label-ink); white-space: nowrap; }
#Lap[data-final=true] .caption { color: var(--hud-accent); }
.now { font-size: var(--hud-size-name); font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); font-variant-numeric: var(--hud-figure-numeric); white-space: nowrap; }
.total { font-size: var(--hud-size-xs); font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); font-variant-numeric: var(--hud-figure-numeric); color: var(--hud-ink-dim); white-space: nowrap; }
/* The places gained: a small triangle, up or down, before the words. */
.gained { align-items: center; column-gap: calc(5 * var(--font-step-px)); max-inline-size: 100%; min-inline-size: 0; color: var(--hud-accent); }
.arrow { flex: none; inline-size: calc(9 * var(--font-step-px)); block-size: calc(7 * var(--font-step-px)); clip-path: polygon(50% 0, 100% 100%, 0 100%); background: var(--hud-accent); }
.gained[data-trend=down] .arrow { clip-path: polygon(0 0, 100% 0, 50% 100%); background: var(--hud-harm); }
.moved { min-inline-size: 0; font-size: var(--hud-label-size); font-weight: var(--hud-label-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); white-space: nowrap; text-overflow: -og-scroll var(--motion-duration-orbit) var(--motion-duration-turn); overflow: hidden; }
.gained[data-trend=down] { color: var(--hud-harm); }
/* The times: a label at the start, the time at the end of each row. */
#Times { display: grid; grid-template-columns: minmax(0, 1fr) auto; row-gap: calc(4 * var(--font-step-px)); column-gap: var(--space-sm); align-items: baseline; }
.row { display: contents; }
.label { min-inline-size: 0; font-size: var(--hud-label-size); font-weight: var(--hud-label-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); color: var(--hud-label-ink); white-space: nowrap; text-overflow: -og-scroll var(--motion-duration-orbit) var(--motion-duration-turn); overflow: hidden; }
.time { justify-self: end; font-size: var(--hud-size-sm); font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); font-variant-numeric: var(--hud-figure-numeric); white-space: nowrap; }
.current .time { font-size: var(--hud-size-time); line-height: 1.05; }
.last .time, .last .label { color: var(--hud-ink-dim); }
.best .label { color: var(--hud-best-label-ink); }
.best .time { color: var(--hud-best-ink); }
/* A new best on the last row: a small tag before the time. */
.end { display: flex; justify-self: end; align-items: baseline; column-gap: calc(6 * var(--font-step-px)); }
.fresh { padding: 0 calc(5 * var(--font-step-px)); font-size: var(--hud-tag-size); font-weight: var(--hud-label-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); white-space: nowrap; color: var(--hud-best-ink); background: var(--hud-tag-fill); box-shadow: var(--hud-tag-rim); clip-path: polygon(var(--hud-tag-cut) 0, 100% 0, 100% calc(100% - var(--hud-tag-cut)), calc(100% - var(--hud-tag-cut)) 100%, 0 100%, 0 var(--hud-tag-cut)); -webkit-text-stroke-width: 0; text-shadow: none; }
.last[data-fresh=true] .time { color: var(--hud-best-ink); }
.split .label { margin-block-start: calc(4 * var(--font-step-px)); }
.delta { justify-self: end; margin-block-start: calc(4 * var(--font-step-px)); padding: 0 calc(6 * var(--font-step-px)); font-size: var(--hud-size-name); font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); font-variant-numeric: var(--hud-figure-numeric); white-space: nowrap; color: var(--hud-ink); background: var(--hud-tag-fill); box-shadow: var(--hud-tag-rim); clip-path: polygon(var(--hud-tag-cut) 0, 100% 0, 100% calc(100% - var(--hud-tag-cut)), calc(100% - var(--hud-tag-cut)) 100%, 0 100%, 0 var(--hud-tag-cut)); -webkit-text-stroke-width: 0; text-shadow: none; }
.delta[data-state=ahead] { color: var(--hud-delta-ink); background: var(--hud-boon); }
.delta[data-state=behind] { color: var(--hud-ink); background: var(--hud-harm); }
.clock { align-self: flex-start; }
/* On a phone: the place smaller, the lap and the places gained beside it,
   then the lap time running over the best lap and the delta; the last lap
   is left out. */
@media (width < 600px) {
  :host { row-gap: calc(8 * var(--font-step-px)); inline-size: stretch; }
  .place { font-size: var(--hud-size-place-compact); }
  .of { font-size: var(--hud-size-md); }
  #Head { column-gap: var(--space-sm); }
  #Side { row-gap: calc(4 * var(--font-step-px)); padding-block-end: 0; }
  .moved { letter-spacing: normal; }
  /* The times in two lines across the full width: the lap time running,
     then the best lap and the split's delta. */
  #Times { row-gap: calc(2 * var(--font-step-px)); column-gap: var(--space-md); grid-template-columns: auto auto minmax(0, 1fr) auto; grid-template-areas: "cl ct ct ct" "bl bt sl sd"; }
  .current .label { grid-area: cl; }
  .current .time { grid-area: ct; }
  .best .label { grid-area: bl; }
  .best .time { grid-area: bt; justify-self: start; }
  .split .label { grid-area: sl; justify-self: end; }
  .delta { grid-area: sd; }
  .current .time { font-size: var(--hud-size-md); }
  .last .label, .last .end { display: none; }
  .label { letter-spacing: normal; }
  .delta, .split .label { margin-block-start: 0; }
}
</style>
