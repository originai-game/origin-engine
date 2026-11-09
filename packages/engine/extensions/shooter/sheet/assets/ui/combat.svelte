<meta name="component" content="CombatPage">

<div id="Page">
  <div id="Sky"></div>
  <div id="Ground"></div>
  <CombatHud health={$health} healthMax={$healthMax} shield={$shield} shieldMax={$shieldMax} spread={$spread} hit={$hit} hitAt={$hitAt} aimed={$aimed} shape={$shape} weapon={$weapon} mode={$mode} magazine={$magazine} capacity={$capacity} reserve={$reserve} reloading={$reloading} hits={$hits} feed={$feed} />
</div>

<style>
/* A stand-in scene, bright above and dark below the horizon, so the HUD is
   reviewed on both (docs/ui.md §6.3). */
#Page { width: stretch; height: stretch; }
#Sky { position: absolute; left: 0; top: 0; width: stretch; height: 55%; background: linear-gradient(180deg, #8fc3ea, #dfe9ef); }
#Ground { position: absolute; left: 0; bottom: 0; width: stretch; height: 45%; background: linear-gradient(180deg, #4a5240, #1b1f18); }
</style>
