<meta name="component" content="CombatPage">

<!-- The combat page, as the approved HUD mockup lays it out over a stage:
     the compass top centre; the squad and objectives top left; the radar,
     the round and score, the killfeed and the pickups down the right; the
     effects over the vitals bottom left, the ability bar bottom centre and
     the ammunition bottom right. The reticle, the damage dealt, the
     incoming damage and the prompt stand at the screen's centre, and
     nothing else comes near the aim. The art is sample art of the sheet
     only. The game sets the HUD's accent to its key colour (amber). -->
<div id="Page">
  <img id="Stage" src={$stage} alt="">
  <div id="Scrim"></div>
  <div id="Hud">
    <div id="Left">
      <div id="Squad"><ShooterSquad members={$squad} segment={25} /></div>
      <div id="Objective"><ShooterObjectives label="" goals={$goals} /></div>
      <div id="Effects"><ShooterEffects effects={$effects} /></div>
      <div id="Vitals"><ShooterVitals name={$vitals.name} level={$vitals.level} portrait={$vitals.portrait} health={$vitals.health} healthMax={$vitals.healthMax} ghost={$vitals.ghost} shield={$vitals.shield} shieldMax={$vitals.shieldMax} segment={$vitals.segment} low={$vitals.low} /></div>
    </div>
    <div id="Middle">
      <div id="Compass"><Compass heading={$heading} pois={$pois} units={$units} /></div>
      <div id="Abilities"><ShooterAbilities slots={$abilities} /></div>
    </div>
    <div id="Right">
      <div id="Radar"><ShooterRadar map={$radar.map} centerX={$radar.centerX} centerY={$radar.centerY} zoom={$radar.zoom} heading={$radar.heading} blips={$blips} /></div>
      <div id="Round"><ShooterRound round={$round.round} clock={$round.clock} ours={$round.ours} theirs={$round.theirs} urgent={$round.urgent} /></div>
      <div id="Feed"><Killfeed rows={$kills} /></div>
      <div id="Pickups"><ShooterPickups items={$pickups} /></div>
      <div id="Ammo"><AmmoCounter weapon={$ammo.weapon} mode={$ammo.mode} magazine={$ammo.magazine} capacity={$ammo.capacity} reserve={$ammo.reserve} reloading={$ammo.reloading} low={$ammo.low} secondary={$ammo.secondary} secondaryAmmo={$ammo.secondaryAmmo} secondaryKey={$ammo.secondaryKey} /></div>
    </div>
    <div id="Aim">
      <div id="Incoming"><DamageDirection hits={$hits} /></div>
      <div id="Reticle"><Crosshair spread={$spread} hit={$hit} hitAt={$hitAt} aimed={$aimed} shape={$shape} /></div>
      <div id="Dealt"><ShooterDamage hits={$dealt} /></div>
      <div id="Prompt"><ShooterPrompt key={$prompt.key} text={$prompt.text} disabled={$prompt.disabled} reason={$prompt.reason} /></div>
    </div>
  </div>
</div>

<style>
#Page { position: fixed; inset: 0; overflow: hidden; background: #05070c;
  --hud-accent: rgb(255 181 71); --hud-pip-on: rgb(255 181 71); --hud-pip-glow: 0 0 calc(6 * var(--font-step-px)) rgb(255 181 71);
  --hud-plate-lit-rim: inset 0 0 0 var(--stroke-hairline) rgb(255 181 71 / 0.45), inset 0 calc(2 * var(--stroke-hairline)) 0 rgb(255 181 71);
  --hud-ready-edge: linear-gradient(180deg, rgb(255 255 255), rgb(255 181 71)); --hud-ready-glow: drop-shadow(0 0 calc(10 * var(--font-step-px)) rgb(255 181 71)); }
#Stage { position: absolute; left: 0; top: 0; width: stretch; height: stretch; object-fit: cover; }
/* The frame darkens toward the edges the HUD stands at, never a flat band. */
#Scrim { position: absolute; left: 0; top: 0; width: stretch; height: stretch; background: radial-gradient(120% 90% at 50% 45%, transparent 55%, rgb(3 5 10 / 0.55) 100%), linear-gradient(180deg, rgb(3 5 10 / 0.5), transparent 16%, transparent 76%, rgb(3 5 10 / 0.62)); }
/* The HUD is three columns round an empty middle, each stacked on its
   own: the squad and objectives over the effects and vitals on the left,
   the compass over the ability bar in the middle, and the radar, round,
   killfeed, pickups and ammunition down the right. Nothing but the compass
   and the ability bar sits in the middle column, so the aim stays clear. */
#Hud { position: relative; display: grid; width: stretch; height: stretch; box-sizing: border-box; column-gap: var(--space-xl);
  grid-template-columns: minmax(0, 1fr) auto minmax(0, 1fr);
  padding: calc(env(-og-hud-inset-top) + var(--space-lg)) calc(env(-og-hud-inset-right) + var(--space-xl)) calc(env(-og-hud-inset-bottom) + var(--space-lg)) calc(env(-og-hud-inset-left) + var(--space-xl)); }
@media (width >= 600px) {
  #Left, #Middle, #Right { display: flex; flex-direction: column; min-width: 0; min-height: 0; }
  #Left { row-gap: var(--space-sm); }
  #Right { row-gap: var(--space-xs); align-items: flex-end; }
}
#Squad, #Objective, #Effects, #Vitals, #Radar, #Round, #Feed, #Pickups, #Ammo, #Compass, #Abilities { display: flex; min-width: 0; }
#Objective { padding-top: var(--space-sm); }
#Effects { margin-top: auto; padding-left: calc(78 * var(--font-step-px)); }
#Compass { justify-content: center; }
#Abilities { margin-top: auto; justify-content: center; }
#Radar, #Feed, #Ammo { justify-content: flex-end; }
#Round { margin-top: var(--space-xs); }
#Round > * { width: calc(var(--size-5xl) * 0.8); }
#Pickups { flex: 1 1 auto; align-items: center; justify-content: flex-end; }
/* The aim: a point at the screen's centre; the incoming damage rings it,
   the reticle sits on it, the damage dealt stands up and to its right and
   the prompt under it. */
#Aim { position: absolute; left: 50%; top: 50%; width: 0; height: 0; --ring: calc(var(--size-xxl) * 2.8); }
#Incoming { position: absolute; left: calc(var(--ring) / -2); top: calc(var(--ring) / -2); width: var(--ring); height: var(--ring); display: flex; }
#Reticle { position: absolute; left: calc(var(--size-xxl) / -2); top: calc(var(--size-xxl) / -2); }
#Dealt { position: absolute; left: calc(32 * var(--font-step-px)); bottom: calc(14 * var(--font-step-px)); }
#Prompt { position: absolute; top: calc(var(--size-xl) * 1.1); left: calc(var(--size-5xl) * -1); width: calc(var(--size-5xl) * 2); display: flex; justify-content: center; }
/* On a phone the columns dissolve into full-width rows: the compass and
   the radar, the round on its own row, the killfeed's newest row across
   the width, the objective in play, the squad's plates side by side; then
   the aim's band, which nothing else
   shares; then the ability bar, the effects beside the ammunition, and
   the vitals across the foot. The pickup feed is left out on a phone. */
@media (width < 600px) {
  #Hud { column-gap: var(--space-sm); row-gap: var(--space-xs);
    grid-template-columns: repeat(7, minmax(0, 1fr));
    grid-template-rows: auto auto auto auto auto minmax(0, 1fr) auto auto auto;
    grid-template-areas: "compass compass compass compass compass radar radar" "round round round round round round round" "feed feed feed feed feed feed feed" "objective objective objective objective objective objective objective" "squad squad squad squad squad squad squad" ". . . . . . ." "abilities abilities abilities abilities abilities abilities abilities" "effects effects effects ammo ammo ammo ammo" "vitals vitals vitals vitals vitals vitals vitals";
    padding: calc(env(-og-hud-inset-top) + var(--space-sm)) calc(env(-og-hud-inset-right) + var(--space-md)) calc(env(-og-hud-inset-bottom) + var(--space-sm)) calc(env(-og-hud-inset-left) + var(--space-md)); }
  #Left, #Middle, #Right { display: contents; }
  #Compass { grid-area: compass; }
  #Radar { grid-area: radar; align-items: flex-start; }
  #Round { grid-area: round; margin-top: 0; align-items: flex-start; }
  #Round > * { width: auto; }
  #Radar { justify-content: flex-end; }
  #Feed { grid-area: feed; align-items: flex-start; }
  #Objective { grid-area: objective; padding-top: 0; }
  #Squad { grid-area: squad; }
  #Pickups { display: none; }
  #Abilities { grid-area: abilities; margin-top: 0; }
  #Effects { grid-area: effects; margin-top: 0; padding-left: 0; align-items: flex-end; }
  #Ammo { grid-area: ammo; align-items: flex-end; }
  #Vitals { grid-area: vitals; }
  /* The aim's column: the prompt takes its width under the reticle; the
     damage dealt stands above and right of it, clear of the
     hit marks. */
  #Aim { --column: calc(var(--size-xxl) * 3); --ring: calc(var(--size-xxl) * 1.1); }
  #Dealt { left: calc(var(--size-xxl) * 0.4); bottom: calc(-4 * var(--font-step-px)); }
  #Prompt { top: var(--space-xxl); left: calc(var(--column) / -2); width: var(--column); }
}
</style>
