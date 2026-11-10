<meta name="component" content="Compass">
<meta name="strings" content="./compass.en.ogt">
<meta name="prop:heading" content="number" data-doc="The player's heading in radians clockwise from north, 0 to 2π.">
<meta name="prop:pois" content="list" data-doc="Its points: each { id, bearing, distance, kind, icon, label }, bearing in radians clockwise from north (0 to 2π), distance in m (negative: none), kind objective, ping, enemy, ally, loot or poi.">
<meta name="prop:units" content="string" data-doc="The distance's unit (m).">
<meta name="fixture:default" content='{"heading":0.6,"units":"m","pois":[{"id":"quest","bearing":0.9,"distance":240,"kind":"objective","icon":"star","label":"Quest"},{"id":"camp","bearing":0.15,"distance":85,"kind":"poi","icon":"flag","label":"Camp"},{"id":"foe","bearing":1.6,"distance":-1,"kind":"enemy","icon":"","label":"Enemy"}]}'>

<div id="Compass" role="status" aria-label={t("component.compass.name")} aria-valuetext={formatNumber(heading * 57.29578) + "°"}>
  <div id="Window">
    <div id="Strip" style:--h={heading}>
      <svg id="Laps" viewBox="0 0 3360 52">
        <defs>
          <symbol id="Lap" viewBox="0 0 1120 52">
        <text x="0.0" y="24" text-anchor="middle" font-size="16" fill="#e5484dff">{t("component.compass.n")}</text>
        <rect x="45.67" y="8" width="2" height="8" fill="#ffffffb3"></rect>
        <rect x="92.33" y="8" width="2" height="8" fill="#ffffffb3"></rect>
        <text x="140.0" y="22" text-anchor="middle" font-size="12" fill="#ffffffff">{t("component.compass.ne")}</text>
        <rect x="185.67" y="8" width="2" height="8" fill="#ffffffb3"></rect>
        <rect x="232.33" y="8" width="2" height="8" fill="#ffffffb3"></rect>
        <text x="280.0" y="24" text-anchor="middle" font-size="16" fill="#ffffffff">{t("component.compass.e")}</text>
        <rect x="325.67" y="8" width="2" height="8" fill="#ffffffb3"></rect>
        <rect x="372.33" y="8" width="2" height="8" fill="#ffffffb3"></rect>
        <text x="420.0" y="22" text-anchor="middle" font-size="12" fill="#ffffffff">{t("component.compass.se")}</text>
        <rect x="465.67" y="8" width="2" height="8" fill="#ffffffb3"></rect>
        <rect x="512.33" y="8" width="2" height="8" fill="#ffffffb3"></rect>
        <text x="560.0" y="24" text-anchor="middle" font-size="16" fill="#ffffffff">{t("component.compass.s")}</text>
        <rect x="605.67" y="8" width="2" height="8" fill="#ffffffb3"></rect>
        <rect x="652.33" y="8" width="2" height="8" fill="#ffffffb3"></rect>
        <text x="700.0" y="22" text-anchor="middle" font-size="12" fill="#ffffffff">{t("component.compass.sw")}</text>
        <rect x="745.67" y="8" width="2" height="8" fill="#ffffffb3"></rect>
        <rect x="792.33" y="8" width="2" height="8" fill="#ffffffb3"></rect>
        <text x="840.0" y="24" text-anchor="middle" font-size="16" fill="#ffffffff">{t("component.compass.w")}</text>
        <rect x="885.67" y="8" width="2" height="8" fill="#ffffffb3"></rect>
        <rect x="932.33" y="8" width="2" height="8" fill="#ffffffb3"></rect>
        <text x="980.0" y="22" text-anchor="middle" font-size="12" fill="#ffffffff">{t("component.compass.nw")}</text>
        <rect x="1025.67" y="8" width="2" height="8" fill="#ffffffb3"></rect>
        <rect x="1072.33" y="8" width="2" height="8" fill="#ffffffb3"></rect>
          </symbol>
        </defs>
        <use href="#Lap" x="0" y="0" width="1120" height="52"></use>
        <use href="#Lap" x="1120" y="0" width="1120" height="52"></use>
        <use href="#Lap" x="2240" y="0" width="1120" height="52"></use>
      </svg>
      {#each pois as poi (poi.id)}
        <div class="poi" class:objective={poi.kind == 'objective'} class:ping={poi.kind == 'ping'} class:enemy={poi.kind == 'enemy'} class:ally={poi.kind == 'ally'} class:loot={poi.kind == 'loot'} style:--b={poi.bearing} aria-label={poi.label}>
          <div class="copy back"><og-icon class="mark" icon={poi.icon} hidden={!poi.icon}></og-icon><svg class="mark" viewBox="0 0 16 16" hidden={!!poi.icon}><path class="diamond" d="M8 1l7 7-7 7-7-7z"></path></svg><span class="distance" hidden={poi.distance < 0}>{formatNumber(poi.distance) + units}</span></div>
          <div class="copy"><og-icon class="mark" icon={poi.icon} hidden={!poi.icon}></og-icon><svg class="mark" viewBox="0 0 16 16" hidden={!!poi.icon}><path class="diamond" d="M8 1l7 7-7 7-7-7z"></path></svg><span class="distance" hidden={poi.distance < 0}>{formatNumber(poi.distance) + units}</span></div>
          <div class="copy ahead"><og-icon class="mark" icon={poi.icon} hidden={!poi.icon}></og-icon><svg class="mark" viewBox="0 0 16 16" hidden={!!poi.icon}><path class="diamond" d="M8 1l7 7-7 7-7-7z"></path></svg><span class="distance" hidden={poi.distance < 0}>{formatNumber(poi.distance) + units}</span></div>
        </div>
      {/each}
    </div>
    <div id="FadeStart"></div>
    <div id="FadeEnd"></div>
  </div>
  <div id="Notch"></div>
  <span id="Bearing">{formatNumber(heading * 57.29578)}</span>
</div>

<style>
#Compass { align-items: center; gap: 0; }
#Window { width: 560px; height: 52px; overflow: hidden; border-radius: 4px; background: linear-gradient(90deg, #00000000, #00000073 18%, #00000073 82%, #00000000); }
#Strip { position: absolute; top: 0; inset-inline-start: 0; width: 3360px; height: 52px; translate: calc(var(--h) * -178.2535px - 840px) 0; }
#Laps { position: absolute; top: 0; inset-inline-start: 0; width: 3360px; height: 52px; }
.poi { position: absolute; top: 0; inset-inline-start: 0; width: 3360px; height: 52px; --ink-mark: var(--ink); }
.poi.objective { --ink-mark: var(--gold); }
.poi.ping { --ink-mark: var(--info); }
.poi.enemy { --ink-mark: var(--danger); }
.poi.ally { --ink-mark: var(--success); }
.poi.loot { --ink-mark: var(--xp); }
.copy { position: absolute; top: 18px; width: 48px; align-items: center; gap: 1px; inset-inline-start: calc(var(--b) * 178.2535px + 1096px); }
.copy.back { inset-inline-start: calc(var(--b) * 178.2535px - 24px); }
.copy.ahead { inset-inline-start: calc(var(--b) * 178.2535px + 2216px); }
.mark { width: 16px; height: 16px; color: var(--ink-mark); }
.diamond { fill: var(--ink-mark); stroke: #000000b3; stroke-width: 1; }
.distance { font: var(--text-caption); color: var(--ink); white-space: nowrap; text-shadow: 0 1px 2px #000000ff; }
#FadeStart { position: absolute; top: 0; inset-inline-start: 0; width: 80px; height: 52px; background: linear-gradient(90deg, #000000b3, #00000000); }
#FadeEnd { position: absolute; top: 0; inset-inline-start: 480px; width: 80px; height: 52px; background: linear-gradient(90deg, #00000000, #000000b3); }
#Notch { width: 2px; height: 8px; background: var(--ink); margin-top: -4px; }
#Bearing { font: var(--text-caption); padding: 1px 6px; border-radius: 4px; background: #000000a6; color: var(--ink); }
</style>
