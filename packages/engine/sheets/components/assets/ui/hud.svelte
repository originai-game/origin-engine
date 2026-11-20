<meta name="component" content="UngroupedHudPage">

<!-- The normal page, in play, laid out as a role-playing game's HUD and
     built from the blocks a game would use: vitals and buffs top left; the
     round's timer and score over the compass top centre; the minimap top
     right; along the foot one console (the life orb, a strip of ability
     slots over the level's bar, the mana orb) with the action hints above
     its end. Over a bright sky and a dark floor (docs/ui.md §6.3: bright
     and dark HUD backgrounds). A phone, upright or on its side, has no room
     for every corner at once, so there they stack in a column that scrolls,
     each seen whole; on its side, a screen each. -->
<div id="Page" data-layout="shell" data-gap="wide">
  <div id="Upper" data-layout="grid" data-gap="wide">
  <div id="TopLeft" data-layout="stack" data-gap="tight">
    <div class="bar"><StatBar value={$statBar_default.value} max={$statBar_default.max} kind={$statBar_default.kind} segment={$statBar_default.segment} shield={$statBar_default.shield} shieldMax={$statBar_default.shieldMax} heal={$statBar_default.heal} recoverable={$statBar_default.recoverable} label={$statBar_default.label} icon={$statBar_default.icon} showValue={$statBar_default.showValue} valueInside={$statBar_default.valueInside} valueText={$statBar_default.valueText} low={$statBar_default.low} critical={$statBar_default.critical} bare={$statBar_default.bare} name={$statBar_default.name} /></div>
    <div class="bar"><StatBar value={$statBar_mana.value} max={$statBar_mana.max} kind={$statBar_mana.kind} segment={$statBar_mana.segment} shield={$statBar_mana.shield} shieldMax={$statBar_mana.shieldMax} heal={$statBar_mana.heal} recoverable={$statBar_mana.recoverable} label={$statBar_mana.label} icon={$statBar_mana.icon} showValue={$statBar_mana.showValue} valueInside={$statBar_mana.valueInside} valueText={$statBar_mana.valueText} low={$statBar_mana.low} critical={$statBar_mana.critical} bare={$statBar_mana.bare} name={$statBar_mana.name} /></div>
    <div class="bar"><StatBar value={$statBar_stamina.value} max={$statBar_stamina.max} kind={$statBar_stamina.kind} segment={$statBar_stamina.segment} shield={$statBar_stamina.shield} shieldMax={$statBar_stamina.shieldMax} heal={$statBar_stamina.heal} recoverable={$statBar_stamina.recoverable} label={$statBar_stamina.label} icon={$statBar_stamina.icon} showValue={$statBar_stamina.showValue} valueInside={$statBar_stamina.valueInside} valueText={$statBar_stamina.valueText} low={$statBar_stamina.low} critical={$statBar_stamina.critical} bare={$statBar_stamina.bare} name={$statBar_stamina.name} /></div>
    <StatusGrid buffs={$statusGrid_default_buffs} debuffs={$statusGrid_default_debuffs} more={$statusGrid_default.more} />
  </div>
  <div id="TopMid" data-layout="stack">
    <div id="Match" data-layout="cluster" data-gap="wide">
      <HudTimer seconds={$hudTimer_default.seconds} total={$hudTimer_default.total} countdown={$hudTimer_default.countdown} warnAt={$hudTimer_default.warnAt} paused={$hudTimer_default.paused} label={$hudTimer_default.label} />
      <StarBar value={$starBar_compact.value} max={$starBar_compact.max} stars={$starBar_compact_stars} label={$starBar_compact.label} valueText={$starBar_compact.valueText} compact={$starBar_compact.compact} />
    </div>
    <div id="Compass"><Compass heading={$compass_default.heading} pois={$compass_default_pois} units={$compass_default.units} /></div>
  </div>
  <div id="TopRight"><MinimapView texture={$minimapView_default.texture} centerX={$minimapView_default.centerX} centerY={$minimapView_default.centerY} zoom={$minimapView_default.zoom} heading={$minimapView_default.heading} rotate={$minimapView_default.rotate} square={$minimapView_default.square} markers={$minimapView_default_markers} zone={$minimapView_default.zone} /></div>
  </div>
  <div id="Lower" data-layout="stack">
  <div id="Hints"><ActionHints hints={$actionHints_default_hints} label={$actionHints_default.label} /></div>
  <div id="Console" data-layout="row" data-gap="tight">
    <OrbMeter value={$orbMeter_default.value} max={$orbMeter_default.max} kind={$orbMeter_default.kind} showValue={$orbMeter_default.showValue} label={$orbMeter_default.label} name={$orbMeter_default.name} />
    <div id="Deck" data-layout="stack" data-gap="tight">
      <AbilityBar slots={$abilityBar_loadout_slots} column={$abilityBar_loadout.column} />
      <StatBar value={$statBar_level.value} max={$statBar_level.max} kind={$statBar_level.kind} segment={$statBar_level.segment} shield={$statBar_level.shield} shieldMax={$statBar_level.shieldMax} heal={$statBar_level.heal} recoverable={$statBar_level.recoverable} label={$statBar_level.label} icon={$statBar_level.icon} showValue={$statBar_level.showValue} valueInside={$statBar_level.valueInside} valueText={$statBar_level.valueText} low={$statBar_level.low} critical={$statBar_level.critical} bare={$statBar_level.bare} name={$statBar_level.name} />
    </div>
    <OrbMeter value={$orbMeter_mana.value} max={$orbMeter_mana.max} kind={$orbMeter_mana.kind} showValue={$orbMeter_mana.showValue} label={$orbMeter_mana.label} name={$orbMeter_mana.name} />
  </div>
  </div>
</div>

<style>
#Page { overflow-y: auto; padding: var(--space-lg); background: linear-gradient(180deg, #9cc7e8, #e2e6df 50%, #4b4038 50%, #16120e); }
#Upper { position: relative; grid-template-columns: minmax(0, 1fr) minmax(0, 1fr) auto; align-items: start; }
.bar, #Compass { inline-size: stretch; }
#TopMid { align-items: center; }
#Match { justify-content: center; }
/* Each top region owns its grid column. The foot follows its contents,
   rather than overlapping the top when text or controls grow. The console
   is as wide as its blocks and no wider than the page: the deck between
   the orbs gives way first, its slots shrinking alike. */
#Lower { position: relative; margin-block-start: auto; align-items: center; }
#Hints { align-self: flex-end; }
#Console { max-inline-size: 100%; align-items: flex-end; justify-content: center; }
#Deck { flex: 0 1 auto; min-inline-size: 0; }
@media (width < 600px), (height < 500px) {
  #Page { padding: env(safe-area-inset-top) var(--space-xs) env(safe-area-inset-bottom); }
  #Upper { grid-template-columns: minmax(0, 1fr); }
  #TopRight { justify-self: center; }
  #Lower { inline-size: stretch; }
  #TopLeft, #TopMid { inline-size: stretch; }
  #Hints { align-self: center; }
}
/* On a phone the orbs stand side by side over the deck, which takes the
   width. */
@media (width < 600px) {
  #Console { flex-wrap: wrap; }
  #Deck { order: 1; flex: 1 1 100%; }
}
/* A phone on its side is too short for two corners at once: each corner
   fills the screen, centred, and the scroll snaps a screen at a time, so
   no corner is ever cut at the screen's edge. The bottom cluster, the one
   play needs, comes first. */
@media (height < 500px) and (width >= 600px) {
  #Page { scroll-snap-type: y mandatory; }
  #TopLeft, #TopMid, #TopRight, #Lower { box-sizing: border-box; flex-shrink: 0; min-height: calc(100dvh - env(safe-area-inset-top) - env(safe-area-inset-bottom)); justify-content: center; scroll-snap-align: start; }
  #Lower { order: -1; }
}
</style>
