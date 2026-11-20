<meta name="component" content="CombatHud">
<meta name="strings" content="./shooter.en.ogt">
<meta name="prop:health" content="number" data-doc="Health now.">
<meta name="prop:healthMax" content="number" data-default="1" data-doc="Most health.">
<meta name="prop:shield" content="number" data-doc="Shield now, a strip over the health bar.">
<meta name="prop:shieldMax" content="number" data-doc="Most shield; 0 hides the strip.">
<meta name="prop:spread" content="number" data-doc="Crosshair spread, 0 to 1 (Crosshair).">
<meta name="prop:hit" content="string" data-default="none" data-doc="Crosshair hit mark: none, hit, headshot, kill or shieldBreak.">
<meta name="prop:hitAt" content="number" data-doc="The hit's id; a new one replays the mark.">
<meta name="prop:aimed" content="bool" data-doc="The aim rests on an enemy.">
<meta name="prop:shape" content="string" data-default="cross" data-doc="Crosshair shape: cross, dot or circle.">
<meta name="prop:weapon" content="string" data-doc="Weapon name (AmmoCounter).">
<meta name="prop:mode" content="string" data-default="auto" data-doc="Fire mode: auto, burst or single.">
<meta name="prop:magazine" content="number" data-doc="Rounds in the magazine.">
<meta name="prop:capacity" content="number" data-doc="Magazine size.">
<meta name="prop:reserve" content="number" data-doc="Rounds in reserve.">
<meta name="prop:reloading" content="number" data-default="-1" data-doc="Reload progress 0 to 1; negative when not reloading.">
<meta name="prop:hits" content="list" data-doc="Incoming hits (DamageDirection).">
<meta name="prop:feed" content="list" data-doc="Killfeed rows, newest first (Killfeed).">
<meta name="fixture:default" content='{"health":72,"healthMax":100,"shield":50,"shieldMax":100,"spread":0.25,"hit":"headshot","hitAt":1,"aimed":true,"shape":"cross","weapon":"Halcyon Carbine","mode":"auto","magazine":24,"capacity":28,"reserve":112,"reloading":-1,"hits":[{"id":"h1","angle":0.9,"strength":0.9,"kind":"normal"}],"feed":[{"id":"k1","killer":"Kestrel","victim":"Vanta","weapon":"Halcyon","headshot":true,"self":true,"team":"ally"}]}'>

<div id="CombatHud" layout="overlay">
  <div id="Centre" anchor="fill" layout="overlay">
    <div id="Damage"><DamageDirection hits={hits} /></div>
    <div layout="center" id="Aim" anchor="fill"><Crosshair spread={spread} hit={hit} hitAt={hitAt} aimed={aimed} shape={shape} /></div>
  </div>
  <div id="Feed" anchor="top-end"><Killfeed rows={feed} /></div>
  <div class="vitals plate" id="Vitals" part="frame" variant="hud" anchor="bottom-start"><StatBar value={health} max={healthMax} kind="health" segment={25} shield={shield} shieldMax={shieldMax} heal={0} recoverable={0} label="" icon="heart" showValue={true} valueInside={false} valueText="" low={0.3} critical={0.15} bare={true} name={t("shooter.hud.health")} /></div>
  <div id="Ammo" anchor="bottom-end"><AmmoCounter weapon={weapon} mode={mode} magazine={magazine} capacity={capacity} reserve={reserve} reloading={reloading} low={0.25} /></div>
</div>

<style>
/* The HUD keeps to the corners and the centre: vitals bottom left, ammunition
   bottom right, the feed top right, the reticle and hit directions centred.
   Edges sit inside the safe area, so a phone's notch and home bar stay clear. */
#CombatHud { position: fixed; inset: 0; pointer-events: none; --edge: var(--space-xl); --layout-anchor-inset: var(--edge); text-shadow: var(--tag-halo); --scene-halo: var(--tag-halo); }
#CombatHud:size(compact) { --edge: var(--space-md); }
#Damage { width: calc(var(--size-xxl) * 3); height: calc(var(--size-xxl) * 3); }
#Feed { right: calc(env(safe-area-inset-right) + var(--edge)); top: calc(env(safe-area-inset-top) + var(--edge)); max-width: calc(100% - var(--edge) * 2); }
/* The vitals plate has the bar's length: a plate that hugs a bar whose
   length is its container's left it none. */
#Vitals { left: calc(env(safe-area-inset-left) + var(--edge)); bottom: calc(env(safe-area-inset-bottom) + var(--edge)); padding: var(--space-sm) var(--space-md); width: var(--measure-sm); max-width: calc(100% - var(--edge) * 2 - env(safe-area-inset-left) - env(safe-area-inset-right)); }
#Ammo { right: calc(env(safe-area-inset-right) + var(--edge)); bottom: calc(env(safe-area-inset-bottom) + var(--edge)); }
/* Right to left the corners mirror with the reading order: the feed and
   the ammunition on the left, the vitals on the right. */
#CombatHud:dir(rtl) #Feed, #CombatHud:dir(rtl) #Ammo { right: auto; left: calc(env(safe-area-inset-left) + var(--edge)); }
#CombatHud:dir(rtl) #Vitals { left: auto; right: calc(env(safe-area-inset-right) + var(--edge)); }
/* Right to left the corners mirror with the reading order (the feed's and the
   readouts' own rows already run from the right): the feed and the ammunition
   on the left { right: auto; left: calc(env(safe-area-inset-left) + var(--edge)); }
/* On a phone the vitals take the bottom edge and the ammunition sits above
   them, and the centre ring shrinks. */
#Vitals:size(compact) { width: calc(100% - var(--edge) * 2 - env(safe-area-inset-left) - env(safe-area-inset-right)); }
#Ammo:size(compact) { bottom: calc(env(safe-area-inset-bottom) + var(--edge) + var(--size-xl)); }
#Damage:size(compact) { width: calc(var(--size-xxl) * 2); height: calc(var(--size-xxl) * 2); }
</style>
