<meta name="component" content="Compass">
<meta name="strings" content="./compass.en.ogt">
<meta name="parts" content="Compass.frame Compass.window Compass.poi Compass.distance Compass.notch Compass.badge Compass.readout Compass.figure">
<meta name="prop:heading" content="number" data-doc="The player's heading in radians clockwise from north, 0 to 2π.">
<meta name="prop:pois" content="list" data-doc="Its points: each { id, bearing, distance, kind, icon, label }, bearing in radians clockwise from north (0 to 2π), distance in m (negative: none), kind objective, ping, enemy, ally, loot or poi.">
<meta name="prop:units" content="string" data-doc="The distance's unit (m).">
<meta name="fixture:default" content='{"heading":0.6,"units":"m","pois":[{"id":"quest","bearing":0.9,"distance":240,"kind":"objective","icon":"star","label":"Quest"},{"id":"camp","bearing":0.15,"distance":85,"kind":"poi","icon":"flag","label":"Camp"},{"id":"foe","bearing":1.6,"distance":-1,"kind":"enemy","icon":"","label":"Enemy"}]}'>

<div id="Compass" part="Compass.frame" class="bare" data-layout="stack" data-gap="tight" role="status" aria-label={t("component.compass.name")} aria-valuetext={formatNumber(heading * 57.29578) + "°"}>
  <div id="Window" part="Compass.window Compass.frame" class="hud" data-fade-ends="x">
    <div id="Strip" style:--h={heading}>
      <svg id="Laps" viewBox="0 0 3360 52">
        <defs>
          <symbol id="Lap" viewBox="0 0 1120 52">
        <text x="0.0" y="24" text-anchor="middle" font-size="16" class="north">{t("component.compass.n")}</text>
        <rect x="45.67" y="8" width="2" height="8" class="tick"></rect>
        <rect x="92.33" y="8" width="2" height="8" class="tick"></rect>
        <text x="140.0" y="22" text-anchor="middle" font-size="12" class="point">{t("component.compass.ne")}</text>
        <rect x="185.67" y="8" width="2" height="8" class="tick"></rect>
        <rect x="232.33" y="8" width="2" height="8" class="tick"></rect>
        <text x="280.0" y="24" text-anchor="middle" font-size="16" class="point">{t("component.compass.e")}</text>
        <rect x="325.67" y="8" width="2" height="8" class="tick"></rect>
        <rect x="372.33" y="8" width="2" height="8" class="tick"></rect>
        <text x="420.0" y="22" text-anchor="middle" font-size="12" class="point">{t("component.compass.se")}</text>
        <rect x="465.67" y="8" width="2" height="8" class="tick"></rect>
        <rect x="512.33" y="8" width="2" height="8" class="tick"></rect>
        <text x="560.0" y="24" text-anchor="middle" font-size="16" class="point">{t("component.compass.s")}</text>
        <rect x="605.67" y="8" width="2" height="8" class="tick"></rect>
        <rect x="652.33" y="8" width="2" height="8" class="tick"></rect>
        <text x="700.0" y="22" text-anchor="middle" font-size="12" class="point">{t("component.compass.sw")}</text>
        <rect x="745.67" y="8" width="2" height="8" class="tick"></rect>
        <rect x="792.33" y="8" width="2" height="8" class="tick"></rect>
        <text x="840.0" y="24" text-anchor="middle" font-size="16" class="point">{t("component.compass.w")}</text>
        <rect x="885.67" y="8" width="2" height="8" class="tick"></rect>
        <rect x="932.33" y="8" width="2" height="8" class="tick"></rect>
        <text x="980.0" y="22" text-anchor="middle" font-size="12" class="point">{t("component.compass.nw")}</text>
        <rect x="1025.67" y="8" width="2" height="8" class="tick"></rect>
        <rect x="1072.33" y="8" width="2" height="8" class="tick"></rect>
          </symbol>
        </defs>
        <use href="#Lap" x="0" y="0" width="1120" height="52"></use>
        <use href="#Lap" x="1120" y="0" width="1120" height="52"></use>
        <use href="#Lap" x="2240" y="0" width="1120" height="52"></use>
      </svg>
      {#each pois as poi (poi.id)}
        <div part="Compass.poi" class="poi" class:objective={poi.kind == 'objective'} class:ping={poi.kind == 'ping'} class:enemy={poi.kind == 'enemy'} class:ally={poi.kind == 'ally'} class:loot={poi.kind == 'loot'} style:--b={poi.bearing} aria-label={poi.label}>
          <div class="copy back" data-layout="stack" data-gap="none">{#if poi.icon}<og-icon class="mark" icon={poi.icon}></og-icon>{:else}<svg class="mark" viewBox="0 0 16 16"><path class="diamond" d="M8 1l7 7-7 7-7-7z"></path></svg>{/if}{#if poi.distance >= 0}<span part="Compass.distance Compass.figure" class="distance">{formatNumber(poi.distance) + units}</span>{/if}</div>
          <div class="copy" data-layout="stack" data-gap="none">{#if poi.icon}<og-icon class="mark" icon={poi.icon}></og-icon>{:else}<svg class="mark" viewBox="0 0 16 16"><path class="diamond" d="M8 1l7 7-7 7-7-7z"></path></svg>{/if}{#if poi.distance >= 0}<span part="Compass.distance Compass.figure" class="distance">{formatNumber(poi.distance) + units}</span>{/if}</div>
          <div class="copy ahead" data-layout="stack" data-gap="none">{#if poi.icon}<og-icon class="mark" icon={poi.icon}></og-icon>{:else}<svg class="mark" viewBox="0 0 16 16"><path class="diamond" d="M8 1l7 7-7 7-7-7z"></path></svg>{/if}{#if poi.distance >= 0}<span part="Compass.distance Compass.figure" class="distance">{formatNumber(poi.distance) + units}</span>{/if}</div>
        </div>
      {/each}
    </div>
    <div id="FadeStart"></div>
    <div id="FadeEnd"></div>
  </div>
  <div class="heading" part="Compass.readout" data-layout="stack" data-gap="hair">
    <div id="Notch" part="Compass.notch"></div>
    <span id="Bearing" part="Compass.badge">{formatNumber(heading * 57.29578) + "°"}</span>
  </div>
</div>

<style>
:host { align-items: center; max-width: 100%; text-shadow: var(--tag-halo); }
#Window { position: relative; width: var(--measure-xl); max-width: 100%; height: var(--size-xl); overflow: hidden; --plate-pad: 0; }
#Strip { position: absolute; top: 0; bottom: 0; inset-inline-start: 50%; width: calc(var(--measure-xl) * 6); translate: calc(var(--h) * var(--measure-xl) * -0.3183099 - var(--measure-xl) * 2) 0; }
#Laps { position: absolute; top: 0; inset-inline: 0; width: 100%; height: calc(var(--measure-xl) / 10.77); }
.north { fill: var(--dot-fill); --dot-fill: var(--tone-danger); }
.point { fill: var(--figure-ink); }
.tick { fill: var(--notch-ink); }
.poi { position: absolute; inset: 0; --dot-fill: var(--figure-ink); }
.poi.objective { --dot-fill: var(--gold); }
.poi.ping { --dot-fill: var(--tone-info); }
.poi.enemy { --dot-fill: var(--tone-danger); }
.poi.ally { --dot-fill: var(--tone-success); }
.poi.loot { --dot-fill: var(--xp); }
.copy { position: absolute; top: calc(var(--space-xl) + var(--space-xs)); width: var(--size-lg); align-items: center; inset-inline-start: calc(var(--b) * var(--measure-xl) * 0.3183099 + var(--measure-xl) * 2 - var(--size-lg) * 0.5); }
.copy.back { inset-inline-start: calc(var(--b) * var(--measure-xl) * 0.3183099 - var(--size-lg) * 0.5); }
.copy.ahead { inset-inline-start: calc(var(--b) * var(--measure-xl) * 0.3183099 + var(--measure-xl) * 4 - var(--size-lg) * 0.5); }
.mark { width: var(--size-xs); height: var(--size-xs); color: var(--dot-fill); }
.diamond { fill: var(--dot-fill); stroke: var(--dot-rim); stroke-width: var(--dot-rim-width); }
.distance { --figure-size: var(--font-step-sm); white-space: nowrap; text-shadow: var(--tag-halo); }
#FadeStart, #FadeEnd { display: none; }
[part="Compass.readout"] { align-items: center; }
#Notch { width: var(--stroke-regular); height: var(--space-sm); }
#Bearing { padding: 0 var(--space-xs); }
</style>
