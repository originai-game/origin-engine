<meta name="component" content="RpgEnemyInfo">
<meta name="strings" content="./rpg.en.ogt">
<meta name="parts" content="RpgEnemyInfo.frame RpgEnemyInfo.band RpgEnemyInfo.title RpgEnemyInfo.name RpgEnemyInfo.row RpgEnemyInfo.headline RpgEnemyInfo.caption RpgEnemyInfo.track RpgEnemyInfo.fill RpgEnemyInfo.badge">
<meta name="prop:name" content="string" data-doc="The enemy's name.">
<meta name="prop:portrait" content="number" data-doc="Its portrait (og.load.texture); 0 is none.">
<meta name="prop:level" content="number" data-doc="Its level.">
<meta name="prop:health" content="number" data-doc="Health now.">
<meta name="prop:healthMax" content="number" data-doc="Most health.">
<meta name="prop:effects" content="list" data-doc="Status effects and traits, keyed rows of { id, icon, label, tone }: tone harm (a debuff on it), boon (a buff), weak (vulnerable to) or strong (resistant to).">
<meta name="fixture:default" content='{"name":"Cultist","portrait":0,"level":6,"health":72,"healthMax":120,"effects":[{"id":"stun","icon":"bolt","label":"Stunned (1)","tone":"harm"},{"id":"fire","icon":"fire","label":"Vulnerable to Fire","tone":"weak"}]}'>

<!--
  The targeted enemy: name, portrait, level, a health bar and its status
  effects. Read only.
-->
<section id="RpgEnemyInfo" part="RpgEnemyInfo.frame" class="hud" data-layout="stack" aria-label={name}>
  <header class="small" part="RpgEnemyInfo.band" data-layout="row"><h3 part="RpgEnemyInfo.title">{t("rpg.enemy.name")}</h3></header>
  <p part="RpgEnemyInfo.name RpgEnemyInfo.headline">{name}</p>
  {#if portrait}<div class="picture"><img class="portrait" src={portrait} alt=""></div>{/if}
  <p class="enemy-level" part="RpgEnemyInfo.caption">{t("rpg.party.level", { level: level })}</p>
  <div class="bar" part="RpgEnemyInfo.track" data-layout="row" role="meter" aria-label={t("rpg.party.health")} style:--fill={healthMax > 0 ? health / healthMax : 0}><div class="bar-fill" part="RpgEnemyInfo.fill"></div><div class="bar-figure" part="RpgEnemyInfo.badge"><span>{t("rpg.party.figure", { now: health, max: healthMax })}</span></div></div>
  {#if effects.length}<ul class="effects" data-layout="cluster" data-gap="tight">
    {#each effects as e (e.id)}
      <li part="RpgEnemyInfo.row" data-layout="row" data-tone={e.tone}><og-icon class="effect-mark" icon={e.icon}></og-icon><span class="effect-label">{e.label}</span></li>
    {/each}
  </ul>{/if}
</section>

<style>
/* Drawn over the scene: the style's tag halo holds its text there. */
:host { text-shadow: var(--tag-halo); }
:host { inline-size: stretch; align-items: center; }
[part="RpgEnemyInfo.band"] { width: stretch; justify-content: center; flex: none; }
[part~="RpgEnemyInfo.name"], .enemy-level { margin: 0; text-align: center; }
.picture { width: min(46%, var(--size-5xl)); aspect-ratio: 1; overflow: hidden; }
.portrait { width: stretch; height: stretch; object-fit: cover; object-position: center top; }
/* The one meter: it leans forward in its well, its figure upright
   over it. */
.bar { position: relative; align-self: stretch; height: calc(var(--space-md) * 1.4); min-block-size: var(--size-sm); justify-content: center; overflow: hidden; margin-inline: var(--space-xs); transform: skewX(var(--bar-skew)); border-radius: 0; background: var(--bar-well); box-shadow: var(--bar-rim); --fill-color: var(--bar-danger); }
.bar-fill { position: absolute; inset-inline-start: 0; top: 0; bottom: 0; inline-size: calc(var(--fill) * 100%); }
.bar-figure { position: relative; padding-inline: var(--space-xs); transform: skewX(calc(-1 * var(--bar-skew))); background: none; box-shadow: none; color: var(--hud-ink); text-shadow: var(--hud-shadow); -webkit-text-stroke-width: var(--hud-stroke-width); -webkit-text-stroke-color: var(--hud-stroke-color); paint-order: stroke fill; }
.effects { width: stretch; margin: 0; padding: 0; list-style: none; }
[part="RpgEnemyInfo.row"] { inline-size: 100%; }
.effect-mark { width: var(--size-sm); height: var(--size-sm); flex: none; }
@media (width < 600px), (height < 500px) { .picture { display: none; } [part="RpgEnemyInfo.row"] { inline-size: auto; } }
</style>
