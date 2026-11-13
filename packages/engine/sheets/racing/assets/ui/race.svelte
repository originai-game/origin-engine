<meta name="component" content="RacePage">

<!-- The normal page, the HUD in a race: lap and position top left with the
     standings under them, the start call across the top third, the
     instruments bottom right; over a bright sky and a dark track so each
     readout is reviewed on both (docs/ui.md §6.3). -->
<div id="Page">
  <div id="Sky"></div>
  <div id="Track"></div>
  <div id="Hud">
    <div id="Left"><div id="Lap"><RacingLapHud lap={$lapHud_default.lap} laps={$lapHud_default.laps} position={$lapHud_default.position} racers={$lapHud_default.racers} current={$lapHud_default.current} best={$lapHud_default.best} last={$lapHud_default.last} split={$lapHud_default.split} splitState={$lapHud_default.splitState} freshBest={$lapHud_default.freshBest} finalLap={$lapHud_default.finalLap} timeLeft={$lapHud_default.timeLeft} timeTotal={$lapHud_default.timeTotal} /></div>
    <div id="Board"><RacingStandings title={$standings_default.title} rows={$standings_default_rows} /></div></div>
    <div id="Call"><RacingBanner kind={$banner_default.kind} figure={$banner_default.figure} at={$banner_default.at} place={$banner_default.place} detail={$banner_default.detail} /></div>
    <div id="Gauge"><RacingTelemetry speed={$telemetry_default.speed} units={$telemetry_default.units} gear={$telemetry_default.gear} rpm={$telemetry_default.rpm} rpmMax={$telemetry_default.rpmMax} redline={$telemetry_default.redline} shift={$telemetry_default.shift} boost={$telemetry_default.boost} boostMax={$telemetry_default.boostMax} boosting={$telemetry_default.boosting} dial={$telemetry_default.dial} /></div>
  </div>
</div>

<style>
#Page { position: relative; width: stretch; height: stretch; overflow: hidden; }
#Sky { position: absolute; left: 0; top: 0; width: stretch; height: 55%; background: linear-gradient(180deg, #8fc3ea, #dfe9ef); }
#Track { position: absolute; left: 0; bottom: 0; width: stretch; height: 45%; background: linear-gradient(180deg, #4a4d52, #17181b); }
#Hud { position: absolute; left: 0; top: 0; width: stretch; height: stretch; }
#Left { position: absolute; left: var(--space-xl); top: var(--space-xl); bottom: var(--space-xl); display: flex; flex-direction: column; align-items: flex-start; gap: var(--space-lg); width: calc(var(--size-xxl) * 3.4); }
#Board { width: stretch; min-height: 0; flex: 0 1 auto; display: flex; }
#Call { position: absolute; left: 0; top: 18%; width: stretch; }
#Gauge { position: absolute; right: var(--space-xl); bottom: var(--space-xl); }
@media (width < 600px) {
  #Left { left: var(--space-sm); right: var(--space-sm); top: var(--space-sm); bottom: auto; width: auto; flex-direction: row; gap: var(--space-sm); max-height: 36%; }
  #Lap { flex: none; }
  #Board { flex: 1; min-width: 0; height: stretch; }
  #Call { top: 42%; }
  #Gauge { right: var(--space-sm); bottom: var(--space-sm); }
}
</style>
