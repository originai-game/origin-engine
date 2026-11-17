<meta name="component" content="ShooterVitals">
<meta name="strings" content="./shooter.en.ogt">
<meta name="parts" content="ShooterVitals.frame ShooterVitals.portrait ShooterVitals.face ShooterVitals.callsign ShooterVitals.rank ShooterVitals.value ShooterVitals.max ShooterVitals.shield ShooterVitals.health ShooterVitals.ghost">
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
  The player's vitals at the screen's bottom corner: the portrait in an
  octagon with a lit edge, the name and level, health as a figure over its
  most, a shield bar cut into cells over a fat health bar that leaves a
  warm ghost when it drops and turns to danger when low. Read only.
-->
<section id="ShooterVitals" part="ShooterVitals.frame" class="hud bare" data-layout="row" role="group" aria-label={t("shooter.vitals.name")} class:low={health <= healthMax * low}>
  <div id="Portrait" part="ShooterVitals.portrait" data-layout="overlay">
    {#if portrait}<img id="Face" part="ShooterVitals.face" data-anchor="fill" src={portrait} alt="">{:else}<div id="Blank" data-anchor="fill" data-layout="center"><og-icon id="Glyph" icon="person"></og-icon></div>{/if}
  </div>
  <div id="Body" data-layout="stack" data-fill>
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
    {#if shieldMax > 0}
    <div id="Shield" part="ShooterVitals.shield" class="bar" data-layout="overlay" role="meter" aria-label={t("shooter.vitals.shield")} aria-valuemin="0" aria-valuemax={shieldMax} aria-valuenow={shield} style:--value={shield <= 0 ? 0 : shield >= shieldMax ? 1 : shield / shieldMax} style:--cells={segment > 0 ? shieldMax / segment : 1}>
      <div id="ShieldFill" class="fill" data-anchor="top-start"></div>
      <div class="cuts" data-anchor="fill"></div>
    </div>
    {/if}
    <div id="Health" part="ShooterVitals.health" class="bar" data-layout="overlay" role="meter" aria-label={t("shooter.vitals.health")} aria-valuemin="0" aria-valuemax={healthMax} aria-valuenow={health} style:--value={health <= 0 ? 0 : health >= healthMax ? 1 : health / healthMax} style:--ghost={ghost <= health ? 0 : ghost >= healthMax ? 1 : ghost / healthMax}>
      <div id="Ghost" part="ShooterVitals.ghost" data-anchor="top-start"></div>
      <div id="HealthFill" class="fill" data-anchor="top-start"></div>
    </div>
  </div>
</section>

<style>
:host { align-items: flex-end; column-gap: calc(14 * var(--font-step-px)); inline-size: calc(330 * var(--font-step-px)); max-inline-size: stretch; color: var(--hud-ink); text-shadow: var(--hud-shadow); -webkit-text-stroke-width: var(--hud-stroke-width); -webkit-text-stroke-color: var(--hud-stroke-color); paint-order: stroke fill; --layout-anchor-inset: 0; }
/* The portrait: an octagon with a lit edge, the art inset inside it. */
#Portrait { flex: none; --layout-anchor-inset: calc(2 * var(--font-step-px)); inline-size: calc(64 * var(--font-step-px)); block-size: calc(64 * var(--font-step-px)); clip-path: var(--hud-octagon); background: var(--hud-edge); filter: var(--hud-lift); }
#Face, #Blank { clip-path: var(--hud-octagon); background: var(--hud-plate-fill); }
#Face { object-fit: cover; }
#Glyph { inline-size: 50%; block-size: 50%; color: var(--hud-ink-dim); }
#Body { min-inline-size: 0; row-gap: calc(5 * var(--font-step-px)); }
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
/* The bars lean forward; the shield's cells are cut across its whole
   well, so a level stops between them. */
.bar { inline-size: 100%; overflow: hidden; transform: skewX(var(--bar-skew)); background: var(--bar-well); box-shadow: var(--bar-rim); }
#Shield { block-size: calc(9 * var(--font-step-px)); }
#Health { block-size: calc(16 * var(--font-step-px)); }
#ShieldFill { inline-size: calc(var(--value) * 100%); block-size: 100%; background: var(--hud-shield-fill); box-shadow: var(--hud-shield-glow); }
.cuts { background: var(--bar-notch); mask-image: repeating-linear-gradient(90deg, transparent 0 calc(100% / var(--cells) - 3 * var(--font-step-px)), #000 calc(100% / var(--cells) - 3 * var(--font-step-px)) calc(100% / var(--cells))); }
#Ghost { inline-size: calc(var(--ghost) * 100%); block-size: 100%; background: var(--bar-ghost); transition: var(--bar-ghost-motion); }
#HealthFill { inline-size: calc(var(--value) * 100%); block-size: 100%; background: var(--hud-health-fill); box-shadow: var(--hud-health-glow); }
.low #HealthFill { background: var(--hud-low-fill); box-shadow: var(--hud-low-glow); }
.low #Health { box-shadow: var(--bar-critical-rim); animation: var(--pulse-motion); }
.low .value { color: var(--hud-empty-ink); }
@media (width < 600px) {
  :host { column-gap: var(--space-sm); inline-size: stretch; }
  #Portrait { inline-size: calc(40 * var(--font-step-px)); block-size: calc(40 * var(--font-step-px)); }
  .value { font-size: var(--hud-size-sm); }
  #Shield { block-size: calc(6 * var(--font-step-px)); }
  #Health { block-size: calc(11 * var(--font-step-px)); }
}
@media (prefers-reduced-motion: reduce) { .low #Health { animation: none; } }
</style>
