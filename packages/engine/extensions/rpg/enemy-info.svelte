<meta name="component" content="RpgEnemyInfo">
<meta name="strings" content="./rpg.en.ogt">
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
<section id="RpgEnemyInfo" class="rpg-enemy-info" aria-label={name}>
  <header class="rpg-band small"><h3 class="rpg-title">{t("rpg.enemy.name")}</h3></header>
  <p class="enemy-name">{name}</p>
  <div class="picture"><img class="portrait" src={portrait} alt="" hidden={!portrait}></div>
  <p class="enemy-level">{t("rpg.party.level", { level: level })}</p>
  <div class="bar" role="meter" aria-label={t("rpg.party.health")} style:--fill={healthMax > 0 ? health / healthMax : 0}><div class="bar-fill"></div><span class="bar-figure">{t("rpg.party.figure", { now: health, max: healthMax })}</span></div>
  <ul class="effects" hidden={!effects.length}>
    {#each effects as e (e.id)}
      <li class="effect" data-tone={e.tone}><icon class="effect-mark" icon={e.icon}></icon><span class="effect-label">{e.label}</span></li>
    {/each}
  </ul>
</section>

<style>
.rpg-enemy-info[hidden], .rpg-enemy-info [hidden] { display: none; }
.rpg-enemy-info { display: flex; flex-direction: column; width: stretch; gap: var(--space-xs); align-items: center; }
.rpg-band { display: flex; width: stretch; align-items: center; justify-content: center; flex: none; }
.enemy-name, .enemy-level { margin: 0; text-align: center; }
.picture { width: 46%; aspect-ratio: 1; overflow: hidden; }
.portrait { width: stretch; height: stretch; object-fit: cover; object-position: center top; }
.bar { position: relative; display: flex; width: stretch; height: calc(var(--space-md) * 1.4); align-items: center; justify-content: center; }
.bar-fill { position: absolute; left: 0; top: 0; width: calc(var(--fill) * 100%); height: stretch; }
.bar-fill:dir(rtl) { left: auto; right: 0; }
.bar-figure { position: relative; font-variant-numeric: tabular-nums; }
.effects { display: flex; flex-direction: column; width: stretch; margin: 0; padding: 0; list-style: none; gap: var(--space-xs); }
.effect { display: flex; align-items: center; gap: var(--space-sm); }
.effect-mark { width: var(--size-sm); height: var(--size-sm); flex: none; }
@media (width < 600px) { .picture { display: none; } .effects { flex-direction: row; flex-wrap: wrap; column-gap: var(--space-md); } }
</style>
