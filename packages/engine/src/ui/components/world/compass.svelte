<meta name="component" content="Compass">
<meta name="strings" content="./compass.en.ogt">
<meta name="parts" content="Compass.frame Compass.window Compass.poi Compass.distance Compass.notch Compass.badge Compass.readout Compass.figure">
<meta name="prop:heading" content="number" data-doc="The player's heading in radians clockwise from north, 0 to 2π.">
<meta name="prop:pois" content="list" data-doc="Its points: each { id, bearing, distance, kind, icon, label }, bearing in radians clockwise from north (0 to 2π), distance in m (negative: none), kind objective, ping, enemy, ally, loot or poi.">
<meta name="prop:units" content="string" data-doc="The distance's unit (m).">
<meta name="fixture:default" content='{"heading":0.733,"units":"m","pois":[{"id":"quest","bearing":0.9,"distance":240,"kind":"objective","icon":"star","label":"Quest"},{"id":"camp","bearing":0.15,"distance":85,"kind":"poi","icon":"flag","label":"Camp"},{"id":"foe","bearing":1.6,"distance":-1,"kind":"enemy","icon":"","label":"Enemy"}]}'>

<div id="Compass" part="Compass.frame" class="bare" data-layout="stack" data-gap="none" role="status" aria-label={t("component.compass.name")} aria-valuetext={formatNumber(heading * 57.29578) + "°"}>
  <div class="heading" part="Compass.readout" data-layout="stack" data-gap="none">
    <span id="Bearing" part="Compass.badge">{(heading * 57.29578 < 9.5 ? '00' : heading * 57.29578 < 99.5 ? '0' : '') + formatNumber(heading * 57.29578) + "°"}</span>
  </div>
  <div id="Window" part="Compass.window Compass.frame" class="hud bare">
    <div id="Band"></div>
    <div id="Strip" style:--h={heading}>
      <span class="point cardinal" style:--at={0}>{t("component.compass.n")}</span>
      <span class="point inter" style:--at={0.375}>{t("component.compass.ne")}</span>
      <span class="point cardinal" style:--at={0.75}>{t("component.compass.e")}</span>
      <span class="point inter" style:--at={1.125}>{t("component.compass.se")}</span>
      <span class="point cardinal" style:--at={1.5}>{t("component.compass.s")}</span>
      <span class="point inter" style:--at={1.875}>{t("component.compass.sw")}</span>
      <span class="point cardinal" style:--at={2.25}>{t("component.compass.w")}</span>
      <span class="point inter" style:--at={2.625}>{t("component.compass.nw")}</span>
      <span class="point cardinal" style:--at={3}>{t("component.compass.n")}</span>
      <span class="point inter" style:--at={3.375}>{t("component.compass.ne")}</span>
      <span class="point cardinal" style:--at={3.75}>{t("component.compass.e")}</span>
      <span class="point inter" style:--at={4.125}>{t("component.compass.se")}</span>
      <span class="point cardinal" style:--at={4.5}>{t("component.compass.s")}</span>
      <span class="point inter" style:--at={4.875}>{t("component.compass.sw")}</span>
      <span class="point cardinal" style:--at={5.25}>{t("component.compass.w")}</span>
      <span class="point inter" style:--at={5.625}>{t("component.compass.nw")}</span>
      <span class="point cardinal" style:--at={6}>{t("component.compass.n")}</span>
      <span class="point inter" style:--at={6.375}>{t("component.compass.ne")}</span>
      <span class="point cardinal" style:--at={6.75}>{t("component.compass.e")}</span>
      <span class="point inter" style:--at={7.125}>{t("component.compass.se")}</span>
      <span class="point cardinal" style:--at={7.5}>{t("component.compass.s")}</span>
      <span class="point inter" style:--at={7.875}>{t("component.compass.sw")}</span>
      <span class="point cardinal" style:--at={8.25}>{t("component.compass.w")}</span>
      <span class="point inter" style:--at={8.625}>{t("component.compass.nw")}</span>
      <div id="Minor" class="ticks"></div>
      <div id="Major" class="ticks"></div>
      {#each pois as poi (poi.id)}
        <div part="Compass.poi" class="poi" class:objective={poi.kind == 'objective'} class:ping={poi.kind == 'ping'} class:enemy={poi.kind == 'enemy'} class:ally={poi.kind == 'ally'} class:loot={poi.kind == 'loot'} style:--b={poi.bearing} aria-label={poi.label}>
          <div class="copy back" data-layout="stack" data-gap="none">{#if poi.icon}<og-icon class="mark" icon={poi.icon}></og-icon>{:else}<div class="mark diamond"></div>{/if}{#if poi.distance >= 0}<span part="Compass.distance Compass.figure" class="distance">{formatNumber(poi.distance) + units}</span>{/if}</div>
          <div class="copy" data-layout="stack" data-gap="none">{#if poi.icon}<og-icon class="mark" icon={poi.icon}></og-icon>{:else}<div class="mark diamond"></div>{/if}{#if poi.distance >= 0}<span part="Compass.distance Compass.figure" class="distance">{formatNumber(poi.distance) + units}</span>{/if}</div>
          <div class="copy ahead" data-layout="stack" data-gap="none">{#if poi.icon}<og-icon class="mark" icon={poi.icon}></og-icon>{:else}<div class="mark diamond"></div>{/if}{#if poi.distance >= 0}<span part="Compass.distance Compass.figure" class="distance">{formatNumber(poi.distance) + units}</span>{/if}</div>
        </div>
      {/each}
    </div>
    <div id="Notch" part="Compass.notch"></div>
  </div>
</div>

<style>
/* A strip of the horizon centred on the heading: the cardinal letters
   large, the intercardinal ones dim, a tick each 15 degrees and a tall
   one each 45, on a dark band that fades out at both ends. The heading
   stands on a tag in the HUD's accent over it with a caret under the
   ticks; each point of interest is a mark under the strip with its
   distance. A lap (360 degrees) is three windows wide, so a window shows
   120 degrees. The band is as tall as what stands on it: a cardinal
   letter (--hud-size-cardinal), then its ticks (2 steps under the letter, 10 tall, 4 off the
   band's foot), so the letter never sits on a tick at any size. */
:host { align-items: center; inline-size: var(--measure-xl); max-inline-size: 100%; container-type: inline-size; color: var(--hud-ink); text-shadow: var(--hud-shadow); -webkit-text-stroke-width: var(--hud-stroke-width); -webkit-text-stroke-color: var(--hud-stroke-color); paint-order: stroke fill; --band: calc(var(--hud-size-cardinal) + 16 * var(--font-step-px)); }
#Window { position: relative; inline-size: 100%; block-size: calc(var(--band) + 34 * var(--font-step-px)); overflow: hidden; mask-image: linear-gradient(90deg, transparent, #000 14%, #000 86%, transparent); --w: 100cqi; }
#Band { position: absolute; inset-inline: 0; top: 0; block-size: var(--band); background: var(--hud-compass-fill); }
#Strip { position: absolute; top: 0; bottom: 0; inset-inline-start: 50%; inline-size: calc(var(--w) * 9); translate: calc(var(--h) * var(--w) * -0.4774648 - var(--w) * 3) 0; }
.point { position: absolute; top: 0; inset-inline-start: calc(var(--at) * var(--w)); translate: -50% 0; font-size: var(--hud-size-sm); line-height: 1.2; font-weight: var(--hud-figure-weight); color: var(--hud-ink-dim); white-space: nowrap; }
.point.cardinal { font-size: var(--hud-size-cardinal); line-height: 1; color: var(--hud-ink); }
/* A letter under the heading tag fades out, so the tag never sits on a
   letter: its distance from the centre, in windows (the heading is at
   h × 0.477 + 3), fades it from 0.09 to 0.05 of a window. */
.point { --d: max(calc(var(--at) - var(--h) * 0.4774648 - 3), calc(var(--h) * 0.4774648 + 3 - var(--at))); opacity: clamp(0, calc((var(--d) - 0.05) * 25), 1); }
/* Ticks: a short one each 15 degrees, a tall one each 45 (a lap is 3w). */
.ticks { position: absolute; inset-inline: 0; bottom: auto; top: calc(var(--band) - 10 * var(--font-step-px)); block-size: calc(6 * var(--font-step-px)); margin-inline-start: calc(-1 * var(--font-step-px)); background: var(--hud-ink-dim); mask-image: repeating-linear-gradient(90deg, #000 0 calc(2 * var(--font-step-px)), transparent calc(2 * var(--font-step-px)) calc(var(--w) / 8)); }
#Major { top: calc(var(--band) - 14 * var(--font-step-px)); block-size: calc(10 * var(--font-step-px)); background: var(--hud-tick); mask-image: repeating-linear-gradient(90deg, #000 0 calc(2 * var(--font-step-px)), transparent calc(2 * var(--font-step-px)) calc(var(--w) * 0.375)); }
.poi { position: absolute; inset: 0; }
.copy { position: absolute; top: calc(var(--band) + 3 * var(--font-step-px)); inline-size: calc(80 * var(--font-step-px)); align-items: center; row-gap: calc(3 * var(--font-step-px)); inset-inline-start: calc(var(--b) * var(--w) * 0.4774648 + var(--w) * 3 - 40 * var(--font-step-px)); }
.copy.back { inset-inline-start: calc(var(--b) * var(--w) * 0.4774648 - 40 * var(--font-step-px)); }
.copy.ahead { inset-inline-start: calc(var(--b) * var(--w) * 0.4774648 + var(--w) * 6 - 40 * var(--font-step-px)); }
.mark { inline-size: calc(12 * var(--font-step-px)); block-size: calc(12 * var(--font-step-px)); color: var(--hud-ink); }
.diamond { inline-size: calc(10 * var(--font-step-px)); block-size: calc(10 * var(--font-step-px)); margin: calc(1 * var(--font-step-px)); transform: rotate(45deg); background: var(--hud-ink); box-shadow: var(--hud-marker-rim); }
.objective .mark { color: var(--hud-gold); }
.objective .diamond { background: var(--hud-gold); }
.ping .mark { color: var(--hud-accent); }
.ping .diamond { background: var(--hud-accent); }
.enemy .mark { color: var(--hud-foe); }
.enemy .diamond { background: var(--hud-foe); }
.ally .mark { color: var(--hud-ally); }
.ally .diamond { background: var(--hud-ally); }
.loot .mark { color: var(--hud-epic); }
.loot .diamond { background: var(--hud-epic); }
.distance { font-size: var(--hud-size-xs); line-height: 1.1; font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); font-variant-numeric: var(--hud-figure-numeric); color: var(--hud-ink); white-space: nowrap; text-shadow: var(--tag-halo); }
/* The heading on a tag in the accent; the caret under the ticks. */
[part="Compass.readout"] { align-items: center; margin-block-end: calc(-4 * var(--font-step-px)); position: relative; z-index: 1; }
#Bearing { padding: 0 calc(5 * var(--font-step-px)); font-size: var(--hud-tag-size); line-height: 1.35; font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); font-variant-numeric: var(--hud-figure-numeric); color: var(--hud-key-ink); background: var(--hud-accent); border: 0; box-shadow: none; -webkit-text-stroke-width: 0; text-shadow: none; border-radius: var(--hud-tag-cut) 0; corner-shape: var(--cut-shape); clip-path: border-box; }
#Notch { position: absolute; top: calc(var(--band) - 4 * var(--font-step-px)); inset-inline-start: calc(50% - 6 * var(--font-step-px)); inline-size: calc(12 * var(--font-step-px)); block-size: calc(8 * var(--font-step-px)); background: var(--hud-accent); clip-path: polygon(0 0, 100% 0, 50% 100%); }
@media (width < 600px) {
  :host { --hud-size-cardinal: var(--hud-size-sm); }
  /* Room under the band for a mark and its distance (1 + 12 + 14.3 steps). */
  #Window { block-size: calc(var(--band) + 30 * var(--font-step-px)); }
  .copy { top: calc(var(--band) + 1 * var(--font-step-px)); row-gap: 0; }
  .point { font-size: var(--hud-size-xs); }
}
</style>
