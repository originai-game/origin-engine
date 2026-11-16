<meta name="component" content="RpgEnemyInfo">
<meta name="strings" content="./rpg.en.ogt">
<meta name="parts" content="RpgEnemyInfo.frame RpgEnemyInfo.band RpgEnemyInfo.title RpgEnemyInfo.name RpgEnemyInfo.row">
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
<section id="RpgEnemyInfo" part="RpgEnemyInfo.frame" class="hud" aria-label={name}>
  <header class="small" part="RpgEnemyInfo.band"><h3 part="RpgEnemyInfo.title">{t("rpg.enemy.name")}</h3></header>
  <p part="RpgEnemyInfo.name">{name}</p>
  <div class="picture">{#if portrait}<img class="portrait" src={portrait} alt="">{/if}</div>
  <p class="enemy-level">{t("rpg.party.level", { level: level })}</p>
  <div class="bar" role="meter" aria-label={t("rpg.party.health")} style:--fill={healthMax > 0 ? health / healthMax : 0}><div class="bar-fill"></div><span class="bar-figure">{t("rpg.party.figure", { now: health, max: healthMax })}</span></div>
  {#if effects.length}<ul class="effects">
    {#each effects as e (e.id)}
      <li part="RpgEnemyInfo.row" data-tone={e.tone}><og-icon class="effect-mark" icon={e.icon}></og-icon><span class="effect-label">{e.label}</span></li>
    {/each}
  </ul>{/if}
</section>

<style>
/* Drawn over the scene: the style's tag halo holds its text there. */
:host { text-shadow: var(--tag-halo); }
:host { display: flex; flex-direction: column; inline-size: stretch; gap: var(--space-sm); align-items: center; }
[part="RpgEnemyInfo.band"] { display: flex; width: stretch; align-items: center; justify-content: center; flex: none; }
[part="RpgEnemyInfo.name"], .enemy-level { margin: 0; text-align: center; }
.picture { width: 46%; aspect-ratio: 1; overflow: hidden; }
.portrait { width: stretch; height: stretch; object-fit: cover; object-position: center top; }
.bar { position: relative; display: flex; align-self: stretch; height: calc(var(--space-md) * 1.4); align-items: center; justify-content: center; background: var(--well); border-radius: var(--meter-radius); overflow: hidden; }
.bar-fill { position: absolute; inset-inline-start: 0; top: 0; bottom: 0; inline-size: calc(var(--fill) * 100%); background: var(--health); }
.bar-figure { position: relative; font-family: var(--figure-font); font-variant-numeric: tabular-nums; color: var(--scene-ink); text-shadow: var(--tag-halo); }
.enemy-level { color: var(--ink-muted); font: var(--text-caption); }
.effects { display: flex; flex-direction: column; width: stretch; margin: 0; padding: 0; list-style: none; gap: var(--space-xs); }
[part="RpgEnemyInfo.row"] { display: flex; align-items: center; gap: var(--space-sm); }
.effect-mark { width: var(--size-sm); height: var(--size-sm); flex: none; }
@media (width < 600px), (height < 500px) { .picture { display: none; } .effects { flex-direction: row; flex-wrap: wrap; column-gap: var(--space-md); } }
</style>
