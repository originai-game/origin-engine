<meta name="component" content="StrategyFieldPage">

<!-- The normal page, the RTS HUD in play: the production queue top left,
     the wave readout top right, the selection and command card along the
     foot; over a bright meadow and a dark forest so each readout is reviewed
     on both (docs/ui.md §6.3). A phone stacks the three in one scrolling
     column. -->
<div id="Page">
  <div id="Meadow"></div>
  <div id="Forest"></div>
  <div id="Hud">
    <div id="Queue"><StrategyQueue title={$queue_default.title} queued={$queue_default.queued} limit={$queue_default.limit} full={$queue_default.full} items={$queue_default_items} /></div>
    <div id="Wave"><StrategyWave wave={$wave_default.wave} waves={$wave_default.waves} phase={$wave_default.phase} next={$wave_default.next} nextTotal={$wave_default.nextTotal} finalWave={$wave_default.finalWave} canCallEarly={$wave_default.canCallEarly} reward={$wave_default.reward} enemies={$wave_default_enemies} /></div>
    <div id="Foot"><StrategySelection title={$selection_default.title} summary={$selection_default.summary} focused={$selection_default.focused} units={$selection_default_units} commands={$selection_default_commands} /></div>
  </div>
</div>

<style>
#Page { position: relative; width: stretch; height: stretch; overflow: hidden; }
#Meadow { position: absolute; left: 0; top: 0; width: stretch; height: 50%; background: linear-gradient(180deg, #b9d98a, #e4edc4); }
#Forest { position: absolute; left: 0; bottom: 0; width: stretch; height: 50%; background: linear-gradient(180deg, #2c3b24, #0e140c); }
#Hud { position: absolute; left: 0; top: 0; width: stretch; height: stretch; }
@media (width >= 600px) {
  #Queue { position: absolute; left: var(--space-xl); top: var(--space-xl); width: calc(var(--size-xxl) * 4); }
  #Wave { position: absolute; right: var(--space-xl); top: var(--space-xl); }
  #Foot { position: absolute; left: var(--space-xl); right: var(--space-xl); bottom: var(--space-xl); }
}
@media (width < 600px) {
  #Hud { display: grid; grid-template-columns: minmax(0, 1fr); align-content: start; gap: var(--space-sm); padding: var(--space-sm); overflow-y: auto; }
}
</style>
