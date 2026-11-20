<meta name="component" content="StrategyFieldPage">

<!-- The normal page, as the approved RTS HUD mockup lays it out over a
     battlefield: the resources and the alerts at the top left, the wave and
     the clock at the top right; along the foot the minimap at the left, the
     control groups over the selection panel (the portrait, the name and
     stats, the selected units' cards and the barracks queue), and the
     command card with the info card of the command in focus at the right.
     The battlefield's middle holds no plate. The art is sample art of the
     sheet only. -->
<div id="Page">
  <img id="Stage" src={$hudField} alt="">
  <div id="Scrim"></div>
  <div id="Hud">
    <div id="Top">
      <div id="Resources"><StrategyResources tier={$field_resources.tier} resources={$field_resources_resources} /></div>
      <div id="Alerts"><StrategyAlerts alerts={$field_alerts_alerts} /></div>
    </div>
    <div id="Corner">
      <div id="Wave"><StrategyWave wave={$field_wave.wave} waves={$field_wave.waves} name={$field_wave.name} icon={$field_wave.icon} phase={$field_wave.phase} next={$field_wave.next} nextTotal={$field_wave.nextTotal} warnAt={$field_wave.warnAt} finalWave={$field_wave.finalWave} canCallEarly={$field_wave.canCallEarly} reward={$field_wave.reward} enemies={$field_wave_enemies} /></div>
      <div id="Clock"><StrategyClock label={$field_clock.label} time={$field_clock.time} speed={$field_clock.speed} paused={$field_clock.paused} /></div>
    </div>
    <div id="Map"><StrategyMinimap map={$field_minimap.map} viewX={$field_minimap.viewX} viewY={$field_minimap.viewY} viewWidth={$field_minimap.viewWidth} viewHeight={$field_minimap.viewHeight} dots={$field_minimap_dots} pings={$field_minimap_pings} /></div>
    <div id="Foot">
      <div id="Groups"><StrategyGroups groups={$field_groups_groups} /></div>
      <div id="Panel">
        <StrategySelection name={$field_selection.name} count={$field_selection.count} kind={$field_selection.kind} line={$field_selection.line} status={$field_selection.status} portrait={$field_selection.portrait} icon={$field_selection.icon} health={$field_selection.health} healthMax={$field_selection.healthMax} stats={$field_selection_stats} focused={$field_selection.focused} units={$field_selection_units} />
        <div id="Queue"><StrategyQueue title={$field_queue.title} queued={$field_queue.queued} limit={$field_queue.limit} full={$field_queue.full} items={$field_queue_items} /></div>
      </div>
    </div>
    <div id="Cmds"><StrategyCommands focused={$field_commands.focused} commands={$field_commands_commands} /></div>
  </div>
</div>

<style>
/* The army's key colour, a blue, and the tokens made from it (a token
   reads the accent where the style declares it, so a page that sets the
   accent sets those too, as a game's pack would at its root). */
#Page { position: fixed; inset: 0; overflow: hidden; background: #05070c;
  --hud-accent: rgb(110 168 255); --hud-accent-ink: rgb(4 16 31);
  --hud-plate-lit-rim: inset 0 0 0 var(--stroke-hairline) rgb(110 168 255 / 0.45), inset 0 calc(2 * var(--stroke-hairline)) 0 rgb(110 168 255);
  --hud-ready-edge: linear-gradient(180deg, rgb(255 255 255), rgb(110 168 255));
  --hud-key-lit-fill: linear-gradient(180deg, rgb(234 244 255), rgb(110 168 255));
  --hud-accent-fill: linear-gradient(180deg, rgb(255 255 255 / 0.55), transparent 42%), rgb(110 168 255); }
#Stage { position: absolute; left: 0; top: 0; width: stretch; height: stretch; object-fit: cover; }
/* The frame darkens toward the top and the foot the HUD stands at. */
#Scrim { position: absolute; left: 0; top: 0; width: stretch; height: stretch; background: linear-gradient(180deg, rgb(3 5 10 / 0.6), transparent 12%, transparent 66%, rgb(3 5 10 / 0.7)); }
/* The top row and the foot row round the battlefield, which is left
   clear: the minimap, the panel and the command card along the foot. */
#Hud { position: relative; display: grid; width: stretch; height: stretch; box-sizing: border-box; column-gap: var(--space-lg); row-gap: var(--space-sm);
  grid-template-columns: auto minmax(0, 1fr) auto; grid-template-rows: auto minmax(0, 1fr) auto;
  grid-template-areas: "top top corner" ". . ." "map foot cmds";
  padding: calc(env(-og-hud-inset-top) + var(--space-md)) calc(env(-og-hud-inset-right) + var(--space-xl)) calc(env(-og-hud-inset-bottom) + var(--space-lg)) calc(env(-og-hud-inset-left) + var(--space-xl)); }
#Top { grid-area: top; display: flex; flex-direction: column; align-items: flex-start; row-gap: var(--space-md); min-width: 0; }
#Corner { grid-area: corner; align-self: start; display: flex; align-items: stretch; justify-content: flex-end; column-gap: var(--space-sm); min-width: 0; }
#Map { grid-area: map; align-self: end; display: flex; min-width: 0; }
#Foot { grid-area: foot; align-self: end; justify-self: start; display: flex; flex-direction: column; align-items: flex-start; row-gap: var(--space-sm); width: stretch; max-width: calc(var(--size-xxl) * 5.8); min-width: 0; }
#Cmds { grid-area: cmds; align-self: end; display: flex; justify-content: flex-end; min-width: 0; }
#Resources, #Alerts, #Wave, #Clock, #Groups { display: flex; min-width: 0; max-width: 100%; }
/* The alerts keep to the top left corner: a short alert fits whole at
   150 % text and a long notice scrolls in its row rather than reach over
   the battlefield. */
#Alerts { max-width: calc(var(--size-xxl) * 4.6); }
/* The panel: smoked glass with a lit top edge holding the selection and,
   under its cards, the barracks queue. */
#Panel { display: flex; flex-direction: column; row-gap: var(--space-md); width: stretch; min-width: 0; box-sizing: border-box; padding: var(--space-md) var(--space-lg); background: var(--hud-plate-fill); box-shadow: var(--hud-plate-rim); clip-path: polygon(var(--hud-plate-cut) 0, 100% 0, 100% calc(100% - var(--hud-plate-cut)), calc(100% - var(--hud-plate-cut)) 100%, 0 100%, 0 var(--hud-plate-cut)); }
#Queue { display: flex; min-width: 0; }
/* On a phone on its side the same rows round the battlefield, tighter:
   the members are compact and the panel's padding and gaps shrink, so the
   foot row keeps inside the screen in every pack's type. */
@media (height < 500px) and (width >= 600px) {
  #Hud { row-gap: var(--space-xs);
    padding: calc(env(-og-hud-inset-top) + var(--space-sm)) calc(env(-og-hud-inset-right) + var(--space-lg)) calc(env(-og-hud-inset-bottom) + var(--space-sm)) calc(env(-og-hud-inset-left) + var(--space-lg)); }
  #Top { row-gap: var(--space-xs); }
  #Foot { row-gap: var(--space-xs); }
  #Panel { row-gap: var(--space-xs); padding: var(--space-xs) var(--space-md); }
}
/* On a phone: the resources across the top, the wave and the clock, the
   alerts; the battlefield in the middle is left clear; at the foot the
   control groups, the small minimap in its corner beside the compact
   panel, and the command card across the width. */
@media (width < 600px) {
  #Hud { column-gap: var(--space-sm); row-gap: var(--space-xs);
    grid-template-columns: auto minmax(0, 1fr);
    grid-template-rows: auto auto auto minmax(0, 1fr) auto auto auto;
    grid-template-areas: "res res" "corner corner" "alerts alerts" "field field" "groups groups" "map panel" "cmds cmds";
    padding: calc(env(-og-hud-inset-top) + var(--space-xs)) calc(env(-og-hud-inset-right) + var(--space-sm)) calc(env(-og-hud-inset-bottom) + var(--space-xs)) calc(env(-og-hud-inset-left) + var(--space-sm)); }
  #Top, #Foot { display: contents; }
  #Resources { grid-area: res; }
  #Alerts { grid-area: alerts; }
  /* The wave and the clock share a row at any text scale: the clock keeps
     its words whole and the wave takes the rest, its name scrolling. */
  #Corner { justify-content: space-between; }
  #Wave { flex: 1 1 0; }
  #Clock { flex: none; }
  #Groups { grid-area: groups; }
  #Map { align-self: stretch; }
  #Panel { grid-area: panel; align-self: end; row-gap: var(--space-xs); padding: var(--space-xs) var(--space-sm); }
  #Cmds { justify-content: stretch; }
}
</style>
