<meta name="component" content="ShooterVitals">
<meta name="strings" content="./shooter.en.ogt">
<meta name="parts" content="ShooterVitals.frame ShooterVitals.portrait ShooterVitals.plate ShooterVitals.callsign ShooterVitals.rank ShooterVitals.value ShooterVitals.max ShooterVitals.shield ShooterVitals.health">
<meta name="prop:name" content="string" data-doc="The player's name as the game words it; empty hides it.">
<meta name="prop:level" content="number" data-doc="The player's level beside the name; 0 hides it.">
<meta name="prop:portrait" content="number" data-doc="The player's portrait texture (og.load.texture); 0 shows the person glyph in its place.">
<meta name="prop:health" content="number" data-doc="Health now.">
<meta name="prop:healthMax" content="number" data-default="100" data-doc="Most health.">
<meta name="prop:ghost" content="number" data-doc="Health before the latest loss: the bar keeps a warm ghost to it that drains; at or under health shows none.">
<meta name="prop:shield" content="number" data-doc="Shield now, a cut bar over the health bar.">
<meta name="prop:shieldMax" content="number" data-doc="Most shield; 0 hides the shield bar.">
<meta name="prop:segment" content="number" data-default="25" data-doc="Shield per cell of the shield bar (25: four cells for 100).">
<meta name="prop:low" content="number" data-default="0.3" data-doc="The fraction of most health at or under which the health bar turns to danger (0.3).">
<meta name="fixture:default" content='{"name":"Ysolde","level":24,"portrait":0,"health":64,"healthMax":100,"ghost":82,"shield":75,"shieldMax":100,"segment":25,"low":0.3}'>
<meta name="fixture:full" content='{"name":"Ysolde","level":24,"portrait":0,"health":100,"healthMax":100,"ghost":0,"shield":100,"shieldMax":100,"segment":25,"low":0.3}'>
<meta name="fixture:low" content='{"name":"Ysolde","level":24,"portrait":0,"health":18,"healthMax":100,"ghost":41,"shield":0,"shieldMax":100,"segment":25,"low":0.3}'>
<meta name="fixture:critical" content='{"name":"Ysolde","level":24,"portrait":0,"health":3,"healthMax":100,"ghost":12,"shield":0,"shieldMax":100,"segment":25,"low":0.3}'>
<meta name="fixture:noshield" content='{"name":"","level":0,"portrait":0,"health":150,"healthMax":200,"ghost":0,"shield":0,"shieldMax":0,"segment":25,"low":0.3}'>
<meta name="fixture:overflow" content='{"name":"Ysolde Varrenhallow-Okonkwo of the Ninth Relay Company","level":1234,"portrait":0,"health":12345,"healthMax":99999,"ghost":20000,"shield":250,"shieldMax":300,"segment":50,"low":0.3}'>

<!--
  The player's vitals at the screen's bottom corner, built from the HUD's
  blocks: an HudPortrait in a lit octagon beside a smoked plate that holds
  the name and level, health as a figure over its most, a shield HudBar
  cut into cells and a fat health HudBar that leaves a warm ghost when it
  drops and turns to danger when low. A game that wants another plate lays
  the same blocks out itself. Read only.
-->
<section id="ShooterVitals" part="ShooterVitals.frame" class="hud bare" data-layout="row" role="group" aria-label={t("shooter.vitals.name")} class:low={health <= healthMax * low}>
  <div id="Portrait" part="ShooterVitals.portrait"><HudPortrait portrait={portrait} dimmed={false} /></div>
  <div id="Body" part="ShooterVitals.plate" data-layout="stack" data-fill>
    <div id="Head" data-layout="row" data-gap="tight">
      <div id="Who" data-layout="row" data-gap="tight">
        {#if name != ''}<span part="ShooterVitals.callsign" class="callsign">{name}</span>{/if}
        {#if level > 0}<span part="ShooterVitals.rank" class="rank">{t("shooter.vitals.level", { level: level })}</span>{/if}
      </div>
      <div id="Reading" data-layout="row" data-gap="tight">
        <span part="ShooterVitals.value" class="value">{formatNumber(health)}</span>
        <span part="ShooterVitals.max" class="max">{t("shooter.vitals.max", { max: formatNumber(healthMax) })}</span>
      </div>
    </div>
    {#if shieldMax > 0}<div id="Shield" part="ShooterVitals.shield"><HudBar value={shield} max={shieldMax} kind="shield" ghost={0} segment={segment} low={false} showValue={false} name={t("shooter.vitals.shield")} /></div>{/if}
    <div id="Health" part="ShooterVitals.health"><HudBar value={health} max={healthMax} kind="health" ghost={ghost} segment={0} low={health <= healthMax * low} showValue={false} name={t("shooter.vitals.health")} /></div>
  </div>
</section>

<style>
:host { align-items: center; column-gap: calc(10 * var(--font-step-px)); inline-size: calc(340 * var(--font-step-px)); max-inline-size: stretch; color: var(--hud-ink); text-shadow: var(--hud-shadow); -webkit-text-stroke-width: var(--hud-stroke-width); -webkit-text-stroke-color: var(--hud-stroke-color); paint-order: stroke fill; }
/* The portrait's place: a large octagon lifted off the scene. */
#Portrait { flex: none; filter: var(--hud-lift); --hud-portrait-size: calc(64 * var(--font-step-px)); }
/* The plate: smoked glass with two corners cut, the bars set into it. */
#Body { min-inline-size: 0; row-gap: calc(5 * var(--font-step-px)); padding: calc(5 * var(--font-step-px)) calc(10 * var(--font-step-px)) calc(8 * var(--font-step-px)); background: var(--hud-plate-fill); box-shadow: var(--hud-plate-rim); border-radius: var(--hud-plate-cut) 0; corner-shape: var(--cut-shape); }
/* The name and level take the row first; the value stands at its end and
   goes under them when they need its room, so a short name never shrinks
   (only a long one scrolls). */
#Head { flex-wrap: wrap; align-items: baseline; row-gap: 0; min-inline-size: 0; }
#Who { flex: 0 1 auto; align-items: baseline; min-inline-size: 0; }
#Reading { flex: none; align-items: baseline; margin-inline-start: auto; }
.callsign { min-inline-size: 0; font-size: var(--hud-size-sm); font-weight: var(--hud-label-weight); white-space: nowrap; text-overflow: -og-scroll var(--motion-duration-orbit) var(--motion-duration-turn); overflow: hidden; }
.rank { flex: none; font-size: var(--hud-label-size); font-weight: var(--hud-label-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); color: var(--hud-accent); white-space: nowrap; }
.value { flex: none; font-size: var(--hud-size-md); line-height: 1; font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); font-variant-numeric: var(--hud-figure-numeric); white-space: nowrap; }
.max { flex: none; font-size: var(--hud-size-xs); font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); font-variant-numeric: var(--hud-figure-numeric); color: var(--hud-ink-dim); white-space: nowrap; }
/* The bars' places: a thin shield over a fat health bar in the shooter's
   bright health fill. */
#Shield { inline-size: 100%; --hud-bar-size: calc(9 * var(--font-step-px)); }
#Health { inline-size: 100%; --hud-bar-size: calc(16 * var(--font-step-px)); --hud-life-fill: var(--hud-health-fill); --hud-life-glow: var(--hud-health-glow); }
.low .value { color: var(--hud-empty-ink); }
@media (width < 600px), (height < 500px) {
  :host { column-gap: var(--space-sm); inline-size: stretch; }
  #Portrait { --hud-portrait-size: calc(40 * var(--font-step-px)); }
  #Body { row-gap: calc(4 * var(--font-step-px)); padding: calc(3 * var(--font-step-px)) calc(7 * var(--font-step-px)) calc(5 * var(--font-step-px)); }
  .value { font-size: var(--hud-size-sm); }
  #Shield { --hud-bar-size: calc(6 * var(--font-step-px)); }
  #Health { --hud-bar-size: calc(11 * var(--font-step-px)); }
}
</style>
