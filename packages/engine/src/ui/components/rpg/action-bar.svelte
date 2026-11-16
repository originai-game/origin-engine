<meta name="component" content="RpgActionBar">
<meta name="strings" content="./rpg.en.ogt">
<meta name="parts" content="RpgActionBar.frame RpgActionBar.skill RpgActionBar.item RpgActionBar.cell">
<meta name="prop:skills" content="list" data-doc="Skill slots, keyed rows of { id, icon, key, name, cooldown, ready }: key the hotkey as the game words it (1); cooldown 0 to 1 still to wait; ready false greys it.">
<meta name="prop:items" content="list" data-doc="Quick items, keyed rows of { id, icon, key, name, count }: count above 0 is shown.">
<meta name="fixture:default" content='{"skills":[{"id":"s1","icon":"fire","key":"1","name":"Firebrand","cooldown":0,"ready":true},{"id":"s2","icon":"shield","key":"2","name":"Guard","cooldown":0.6,"ready":true}],"items":[{"id":"p","icon":"item-potion","key":"Q","name":"Healing Draught","count":9}]}'>

<!--
  The action bar: numbered skill slots, each with its hotkey and a
  cooldown shade, and a group of quick items with counts. Reports use (a
  slot's key, tap or click); the game binds the hotkeys.
-->
<nav id="RpgActionBar" part="RpgActionBar.frame" class="bare" aria-label={t("rpg.actions.name")}>
  <div class="skills" role="toolbar">
    {#each skills as s (s.id)}
      <button part="RpgActionBar.skill RpgActionBar.cell" data-ready={s.ready} aria-label={s.name} disabled={!s.ready} onclick={use}><div class="face">
        <og-icon class="skill-art" icon={s.icon}></og-icon>
        {#if s.cooldown > 0}<div class="cool" style:--cool={s.cooldown}></div>{/if}
        <span class="hotkey">{s.key}</span>
      </div></button>
    {/each}
  </div>
  {#if items.length}<div class="items" role="toolbar">
    {#each items as i (i.id)}
      <button part="RpgActionBar.item RpgActionBar.cell" aria-label={i.name} onclick={use}><div class="face">
        <og-icon class="item-art" icon={i.icon}></og-icon>
        {#if i.count > 0}<span class="count">{formatNumber(i.count)}</span>{/if}
        <span class="hotkey">{i.key}</span>
      </div></button>
    {/each}
  </div>{/if}
</nav>

<style>
/* Drawn over the scene: the style's tag halo holds its text there. */
:host { text-shadow: var(--tag-halo); }
:host { display: flex; align-items: flex-end; gap: var(--space-lg); inline-size: fit-content; }
.skills, .items { display: flex; gap: var(--space-xs); padding: var(--space-xs); }
[part="RpgActionBar.skill RpgActionBar.cell"], [part="RpgActionBar.item RpgActionBar.cell"] { width: calc(var(--size-xl) * 0.85); height: calc(var(--size-xl) * 0.85); padding: 0; }
.face { display: flex; flex-direction: column; position: relative; inline-size: 100%; block-size: 100%; align-items: center; justify-content: center; overflow: hidden; border-radius: var(--radius-control); }
.skill-art { inline-size: 100%; block-size: 100%; }
.item-art { width: 84%; height: 84%; }
.cool { position: absolute; inset-inline: 0; top: 0; block-size: calc(var(--cool) * 100%); background: var(--scrim); }
.hotkey { position: absolute; inset-inline-start: var(--space-xxs); top: var(--space-xxs); padding: 0 var(--space-xs); background: var(--scene-plate); color: var(--scene-ink); border-radius: var(--radius-control); font: var(--text-caption); }
.count { position: absolute; inset-inline-end: var(--space-xxs); bottom: var(--space-xxs); padding: 0 var(--space-xs); background: var(--scene-plate); color: var(--scene-ink); border-radius: var(--radius-control); font-family: var(--figure-font); }
@media (width < 600px), (height < 500px) { .items { display: none; } [part="RpgActionBar.skill RpgActionBar.cell"] { width: var(--size-lg); height: var(--size-lg); } }
</style>
