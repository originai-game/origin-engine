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
  The targeted enemy: name, portrait, level, a health HudBar with its
  figure and its status effects. Read only.
-->
<section id="RpgEnemyInfo" part="frame" variant="hud" layout="stack" aria-label={name}>
  <header class="band" variant="small" part="band" layout="row"><h3 part="title">{t("rpg.enemy.name")}</h3></header>
  <p class="name" part="headline">{name}</p>
  {#if portrait}<div class="picture"><img class="portrait" src={portrait} alt=""></div>{/if}
  <p class="enemy-level" part="caption">{t("rpg.party.level", { level: level })}</p>
  <div class="bar" part="health"><HudBar value={health} max={healthMax} kind="health" ghost={0} segment={0} low={false} showValue={true} name="" /></div>
  {#if effects.length}<ul class="effects" layout="wrap" gap="tight">
    {#each effects as e (e.id)}
      <li class="row" part="row" layout="row" tone={e.tone}><og-icon class="effect-mark" icon={e.icon}></og-icon><span class="effect-label">{e.label}</span></li>
    {/each}
  </ul>{/if}
</section>

<style>
/* Drawn over the scene: the style's tag halo holds its text there. */
#RpgEnemyInfo { text-shadow: var(--tag-halo); --scene-halo: var(--tag-halo); }
#RpgEnemyInfo { inline-size: stretch; align-items: center; }
.band { width: stretch; justify-content: center; flex: none; }
.name, .enemy-level { margin: 0; text-align: center; }
.picture { width: min(46%, var(--size-5xl)); aspect-ratio: 1; overflow: hidden; }
.portrait { width: stretch; height: stretch; object-fit: cover; object-position: center top; }
/* The bar's place: thick enough for its figure, filled in danger. */
.bar { align-self: stretch; margin-inline: var(--space-xs); --hud-bar-size: max(calc(var(--space-md) * 1.4), var(--size-sm)); --hud-life-fill: var(--fill-gloss), var(--bar-danger); --hud-life-glow: none; }
.effects { width: stretch; margin: 0; padding: 0; list-style: none; }
.row { inline-size: 100%; }
.effect-mark { width: var(--size-sm); height: var(--size-sm); flex: none; }
@media (width < 600px), (height < 500px) { .picture { display: none; } .row { inline-size: auto; } }
</style>
