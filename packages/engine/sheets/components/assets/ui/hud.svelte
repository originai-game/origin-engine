<meta name="component" content="UngroupedHudPage">

<!-- The normal page, in play: vitals and buffs top left, the round timer and
     compass top centre, the minimap top right, the ability bar and orbs
     along the bottom with the action hints, over a bright sky and a dark
     floor (docs/ui.md §6.3: bright and dark HUD backgrounds). A phone,
     upright or on its side, has no room for every corner at once, so there
     they stack in a column that scrolls, each seen whole. -->
<div id="Page">
  <div id="Sky"></div>
  <div id="Floor"></div>
  <div id="TopLeft">
    <div class="bar"><StatBar value={$statBar_default.value} max={$statBar_default.max} kind={$statBar_default.kind} segment={$statBar_default.segment} shield={$statBar_default.shield} shieldMax={$statBar_default.shieldMax} heal={$statBar_default.heal} recoverable={$statBar_default.recoverable} label={$statBar_default.label} icon={$statBar_default.icon} showValue={$statBar_default.showValue} valueInside={$statBar_default.valueInside} valueText={$statBar_default.valueText} low={$statBar_default.low} critical={$statBar_default.critical} bare={$statBar_default.bare} name={$statBar_default.name} /></div>
    <div class="bar"><StatBar value={$statBar_mana.value} max={$statBar_mana.max} kind={$statBar_mana.kind} segment={$statBar_mana.segment} shield={$statBar_mana.shield} shieldMax={$statBar_mana.shieldMax} heal={$statBar_mana.heal} recoverable={$statBar_mana.recoverable} label={$statBar_mana.label} icon={$statBar_mana.icon} showValue={$statBar_mana.showValue} valueInside={$statBar_mana.valueInside} valueText={$statBar_mana.valueText} low={$statBar_mana.low} critical={$statBar_mana.critical} bare={$statBar_mana.bare} name={$statBar_mana.name} /></div>
    <div class="bar"><StatBar value={$statBar_stamina.value} max={$statBar_stamina.max} kind={$statBar_stamina.kind} segment={$statBar_stamina.segment} shield={$statBar_stamina.shield} shieldMax={$statBar_stamina.shieldMax} heal={$statBar_stamina.heal} recoverable={$statBar_stamina.recoverable} label={$statBar_stamina.label} icon={$statBar_stamina.icon} showValue={$statBar_stamina.showValue} valueInside={$statBar_stamina.valueInside} valueText={$statBar_stamina.valueText} low={$statBar_stamina.low} critical={$statBar_stamina.critical} bare={$statBar_stamina.bare} name={$statBar_stamina.name} /></div>
    <StatusGrid buffs={$statusGrid_default_buffs} debuffs={$statusGrid_default_debuffs} more={$statusGrid_default.more} />
  </div>
  <div id="TopMid">
    <HudTimer seconds={$hudTimer_default.seconds} total={$hudTimer_default.total} countdown={$hudTimer_default.countdown} warnAt={$hudTimer_default.warnAt} paused={$hudTimer_default.paused} label={$hudTimer_default.label} />
    <div id="Compass"><Compass heading={$compass_default.heading} pois={$compass_default_pois} units={$compass_default.units} /></div>
  </div>
  <div id="TopRight"><MinimapView texture={$minimapView_default.texture} centerX={$minimapView_default.centerX} centerY={$minimapView_default.centerY} zoom={$minimapView_default.zoom} heading={$minimapView_default.heading} rotate={$minimapView_default.rotate} square={$minimapView_default.square} markers={$minimapView_default_markers} zone={$minimapView_default.zone} /></div>
  <div id="Lower">
  <div id="Bottom">
    <OrbMeter value={$orbMeter_default.value} max={$orbMeter_default.max} kind={$orbMeter_default.kind} showValue={$orbMeter_default.showValue} label={$orbMeter_default.label} name={$orbMeter_default.name} />
    <div id="Abilities">
      <AbilityBar slots={$abilityBar_default_slots} column={$abilityBar_default.column} />
      <StarBar value={$starBar_default.value} max={$starBar_default.max} stars={$starBar_default_stars} label={$starBar_default.label} valueText={$starBar_default.valueText} compact={$starBar_default.compact} />
    </div>
    <OrbMeter value={$orbMeter_mana.value} max={$orbMeter_mana.max} kind={$orbMeter_mana.kind} showValue={$orbMeter_mana.showValue} label={$orbMeter_mana.label} name={$orbMeter_mana.name} />
  </div>
  <div id="Hints"><ActionHints hints={$actionHints_default_hints} label={$actionHints_default.label} /></div>
  </div>
</div>

<style>
#Page { position: relative; width: stretch; height: stretch; overflow: hidden; }
#Sky { position: absolute; left: 0; top: 0; width: stretch; height: 50%; background: linear-gradient(180deg, #9cc7e8, #e2e6df); }
#Floor { position: absolute; left: 0; bottom: 0; width: stretch; height: 50%; background: linear-gradient(180deg, #4b4038, #16120e); }
#TopLeft { position: absolute; left: env(-og-hud-inset-left, var(--space-lg)); top: env(-og-hud-inset-top, var(--space-lg)); display: flex; flex-direction: column; gap: var(--space-xs); width: calc(var(--size-xxl) * 4.5); }
.bar { display: flex; width: stretch; }
#TopMid { position: absolute; left: 30%; right: 30%; top: env(-og-hud-inset-top, var(--space-lg)); display: flex; flex-direction: column; align-items: center; gap: var(--space-sm); }
#Compass { display: flex; width: stretch; }
#TopRight { position: absolute; right: env(-og-hud-inset-right, var(--space-lg)); top: env(-og-hud-inset-top, var(--space-lg)); }
/* The hints sit right above the bottom cluster, so they clear the
   ability bar however tall the text scale makes it. */
#Lower { position: absolute; left: 0; right: 0; bottom: env(-og-hud-inset-bottom, var(--space-lg)); display: flex; flex-direction: column-reverse; align-items: flex-end; gap: var(--space-sm); }
#Bottom { display: flex; width: stretch; align-items: flex-end; justify-content: center; gap: var(--space-lg); }
#Abilities { display: flex; flex-direction: column; align-items: center; gap: var(--space-sm); }
#Hints { margin-inline-end: env(-og-hud-inset-right, var(--space-lg)); }
@media (width < 600px), (height < 500px) {
  #Page { display: flex; flex-direction: column; align-items: center; gap: var(--space-lg); padding: env(safe-area-inset-top) var(--space-xs) env(safe-area-inset-bottom); overflow-y: auto; }
  #TopLeft, #TopMid, #TopRight, #Lower { position: relative; left: auto; right: auto; top: auto; bottom: auto; }
  #Lower { width: stretch; flex-direction: column; align-items: stretch; gap: var(--space-lg); }
  #Hints { margin-inline-end: 0; }
  #TopLeft, #TopMid, #Hints { width: stretch; }
  #Bottom { width: stretch; gap: var(--space-xs); }
}
</style>
