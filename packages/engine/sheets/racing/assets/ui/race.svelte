<meta name="component" content="RacePage">

<!-- The normal page, as the approved HUD mockup lays it out over a stage:
     the place, the lap and the lap times top left over the position tower;
     the call and the slipstream with the points earned top centre; the
     track map top right over the tachometer at the bottom right; along the
     foot the pedals and the tyres at the left and the nitro meter. The road ahead
     (the middle and the lower middle, where the car drives) stays clear.
     The art is sample art of the sheet only. -->
<div id="Page">
  <img id="Stage" src={$hudStage} alt="">
  <div id="Scrim"></div>
  <div id="Hud">
    <div id="Left">
      <div id="Lap"><RacingLapHud lap={$lapHud_default.lap} laps={$lapHud_default.laps} position={$lapHud_default.position} racers={$lapHud_default.racers} gained={$lapHud_default.gained} current={$lapHud_default.current} best={$lapHud_default.best} last={$lapHud_default.last} sector={$lapHud_default.sector} split={$lapHud_default.split} splitState={$lapHud_default.splitState} freshBest={$lapHud_default.freshBest} finalLap={$lapHud_default.finalLap} timeLeft={$lapHud_default.timeLeft} timeTotal={$lapHud_default.timeTotal} /></div>
      <div id="Tower"><RacingStandings title={$standings_default.title} hasPlayer={$standings_default.hasPlayer} rows={$standings_default_rows} /></div>
    </div>
    <div id="Middle">
      <div id="Call"><RacingBanner kind={$banner_default.kind} figure={$banner_default.figure} text={$banner_default.text} at={$banner_default.at} place={$banner_default.place} detail={$banner_default.detail} /></div>
      <div id="Bonus"><RacingBonus meter={$bonus_default.meter} value={$bonus_default.value} points={$bonus_default.points} detail={$bonus_default.detail} at={$bonus_default.at} /></div>
    </div>
    <div id="Right">
      <div id="Map"><RacingTrackMap path={$trackMap_default.path} driven={$trackMap_default.driven} startX={$trackMap_default.startX} startY={$trackMap_default.startY} startAngle={$trackMap_default.startAngle} name={$trackMap_default.name} length={$trackMap_default.length} cars={$trackMap_default_cars} /></div>
      <div id="Gauge"><RacingTelemetry speed={$telemetry_default.speed} units={$telemetry_default.units} gear={$telemetry_default.gear} mode={$telemetry_default.mode} rpm={$telemetry_default.rpm} rpmMax={$telemetry_default.rpmMax} redline={$telemetry_default.redline} scale={$telemetry_default_scale} lights={$telemetry_default.lights} shift={$telemetry_default.shift} /></div>
    </div>
    <div id="Foot">
      <div id="Pedals"><RacingPedals throttle={$pedals_default.throttle} brake={$pedals_default.brake} /></div>
      <div id="Car"><RacingCar tyres={$car_default_tyres} fuel={$car_default.fuel} fuelText={$car_default.fuelText} fuelLow={$car_default.fuelLow} pit={$car_default.pit} /></div>
      <div id="Boost"><RacingBoost label={$boost_default.label} icon={$hudNitro} key={$boost_default.key} value={$boost_default.value} max={$boost_default.max} cells={$boost_default.cells} firing={$boost_default.firing} /></div>
    </div>
  </div>
</div>

<style>
/* The race's key colour, a lime, and the tokens made from it (a token
   reads the accent where the style declares it, so a page that sets the
   accent sets those too, as a game's pack would at its root). */
#Page { position: fixed; inset: 0; overflow: hidden; background: #05070c;
  --hud-accent: rgb(214 255 58); --hud-player-fill: linear-gradient(90deg, rgb(214 255 58), rgb(214 255 58 / 0.55));
  --hud-plate-lit-rim: inset 0 0 0 var(--stroke-hairline) rgb(214 255 58 / 0.45), inset 0 calc(2 * var(--stroke-hairline)) 0 rgb(214 255 58);
  --hud-accent-fill: linear-gradient(180deg, rgb(255 255 255 / 0.55), transparent 42%), rgb(214 255 58);
  --hud-accent-glow: calc(2 * var(--font-step-px)) 0 calc(10 * var(--font-step-px)) rgb(214 255 58 / 0.75); }
#Stage { position: absolute; left: 0; top: 0; width: stretch; height: stretch; object-fit: cover; }
/* The frame darkens toward the edges the HUD stands at, never a flat band. */
#Scrim { position: absolute; left: 0; top: 0; width: stretch; height: stretch; background: radial-gradient(120% 90% at 50% 45%, transparent 55%, rgb(3 5 10 / 0.55) 100%), linear-gradient(180deg, rgb(3 5 10 / 0.5), transparent 16%, transparent 76%, rgb(3 5 10 / 0.62)); }
/* Three columns round the road. The side columns take their content's
   width up to a share of the screen, the middle (the call and the
   slipstream over the road) the rest; the road below them is left clear.
   The right column runs the screen's height, the track map at its top and
   the tachometer at its foot; the foot's row under the other two holds
   the pedals and the tyres at its start and the nitro meter at its end. */
#Hud { position: relative; display: grid; width: stretch; height: stretch; box-sizing: border-box; column-gap: var(--space-xl); row-gap: var(--space-sm);
  grid-template-columns: fit-content(30%) minmax(0, 1fr) fit-content(30%); grid-template-rows: minmax(0, 1fr) auto;
  grid-template-areas: "left middle right" "foot foot right";
  padding: calc(env(-og-hud-inset-top) + var(--space-lg)) calc(env(-og-hud-inset-right) + var(--space-xl)) calc(env(-og-hud-inset-bottom) + var(--space-lg)) calc(env(-og-hud-inset-left) + var(--space-xl)); }
#Left { grid-area: left; }
#Middle { grid-area: middle; }
#Right { grid-area: right; }
#Foot { grid-area: foot; display: flex; align-items: flex-end; column-gap: var(--space-xl); min-width: 0; }
#Left:not(:size(narrow)), #Middle:not(:size(narrow)), #Right:not(:size(narrow)) { display: flex; flex-direction: column; min-width: 0; min-height: 0; }
#Left:not(:size(narrow)) { row-gap: var(--space-xl); align-items: flex-start; }
#Middle:not(:size(narrow)) { row-gap: var(--space-lg); align-items: center; }
#Call:not(:size(narrow)) { align-self: stretch; }
#Right:not(:size(narrow)) { align-items: flex-end; justify-content: space-between; row-gap: var(--space-lg); }
#Lap, #Tower, #Call, #Bonus, #Map, #Pedals, #Car, #Boost, #Gauge { display: flex; min-width: 0; max-width: 100%; }
/* The tower takes the height left in its column as a size container, so
   at a large text scale it keeps the rows round the player whole. */
#Tower { flex: 1 1 0; min-height: 0; align-self: stretch; container-type: size; }
#Call { justify-content: center; }
#Bonus { justify-content: center; }
#Boost { flex: 0 1 auto; margin-left: auto; padding-bottom: var(--space-md); }
#Gauge { flex: none; }
/* On a phone the columns dissolve into full-width rows at the top: the
   place and the lap times, the three rows of the tower round the player,
   the call, then the slipstream and the points in one compact row. The
   road, the lower middle where the car drives, is left clear. At the foot
   the track map, the nitro meter and the tyres stacked at the left beside
   the tachometer. The pedals are left out. */
#Hud:size(compact) { column-gap: var(--space-md); row-gap: var(--space-xs);
    grid-template-columns: minmax(0, 1fr) auto;
    grid-template-rows: auto auto auto auto minmax(0, 1fr) auto auto auto;
    grid-template-areas: "lap lap" "tower tower" "call call" "bonus bonus" "road road" "map gauge" "boost gauge" "car gauge";
    padding: calc(env(-og-hud-inset-top) + var(--space-sm)) calc(env(-og-hud-inset-right) + var(--space-md)) calc(env(-og-hud-inset-bottom) + var(--space-sm)) calc(env(-og-hud-inset-left) + var(--space-md)); }
#Left:size(compact), #Middle:size(compact), #Right:size(compact), #Foot:size(compact) { display: contents; }
#Lap:size(compact) { grid-area: lap; }
#Tower:size(compact) { grid-area: tower; flex: none; min-height: auto; align-self: auto; container-type: normal; }
#Call:size(compact) { grid-area: call; }
#Bonus:size(compact) { grid-area: bonus; }
#Pedals:size(compact) { display: none; }
#Map:size(compact) { grid-area: map; align-self: end; }
#Boost:size(compact) { grid-area: boost; margin-left: 0; padding-bottom: 0; align-self: end; }
#Car:size(compact) { grid-area: car; align-self: end; }
#Gauge:size(compact) { grid-area: gauge; align-self: end; }
</style>
