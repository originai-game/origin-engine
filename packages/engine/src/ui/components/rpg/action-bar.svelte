<meta name="component" content="RpgActionBar">
<meta name="strings" content="./rpg.en.ogt">
<meta name="parts" content="RpgActionBar.frame RpgActionBar.skill RpgActionBar.item">
<meta name="prop:skills" content="list" data-doc="Skill slots, keyed rows of { id, icon, key, name, cooldown, ready }: key the hotkey as the game words it (1); cooldown 0 to 1 still to wait; ready false greys it.">
<meta name="prop:items" content="list" data-doc="Quick items, keyed rows of { id, icon, key, name, count }: count above 0 is shown.">
<meta name="fixture:default" content='{"skills":[{"id":"s1","icon":"fire","key":"1","name":"Firebrand","cooldown":0,"ready":true},{"id":"s2","icon":"shield","key":"2","name":"Guard","cooldown":0.6,"ready":true}],"items":[{"id":"p","icon":"item-potion","key":"Q","name":"Healing Draught","count":9}]}'>

<!--
  The action bar: numbered skill slots, each with its hotkey and a
  cooldown shade, and a group of quick items with counts. Reports use (a
  slot's key, tap or click); the game binds the hotkeys.
-->
<nav id="RpgActionBar" part="RpgActionBar.frame" aria-label={t("rpg.actions.name")}>
  <div class="skills" role="toolbar">
    {#each skills as s (s.id)}
      <button part="RpgActionBar.skill" data-ready={s.ready} aria-label={s.name} disabled={!s.ready} onclick={use}><div class="face">
        <og-icon class="skill-art" icon={s.icon}></og-icon>
        <div class="cool" hidden={!(s.cooldown > 0)} style:--cool={s.cooldown}></div>
        <span class="hotkey">{s.key}</span>
      </div></button>
    {/each}
  </div>
  <div class="items" role="toolbar" hidden={!items.length}>
    {#each items as i (i.id)}
      <button part="RpgActionBar.item" aria-label={i.name} onclick={use}><div class="face">
        <og-icon class="item-art" icon={i.icon}></og-icon>
        <span class="count" hidden={!(i.count > 0)}>{formatNumber(i.count)}</span>
        <span class="hotkey">{i.key}</span>
      </div></button>
    {/each}
  </div>
</nav>

<style>
[part="RpgActionBar.frame"][hidden], [part="RpgActionBar.frame"] [hidden] { display: none; }
[part="RpgActionBar.frame"] { display: flex; align-items: flex-end; gap: var(--space-lg); width: fit-content; }
.skills, .items { display: flex; gap: var(--space-xs); padding: var(--space-xs); }
[part="RpgActionBar.skill"], [part="RpgActionBar.item"] { width: calc(var(--size-xl) * 0.85); height: calc(var(--size-xl) * 0.85); padding: 0; }
.face { position: relative; width: stretch; height: stretch; align-items: center; justify-content: center; }
.skill-art { width: stretch; height: stretch; }
.item-art { width: 84%; height: 84%; }
.cool { position: absolute; left: 0; top: 0; width: stretch; height: calc(var(--cool) * 100%); }
.hotkey { position: absolute; left: 0; top: calc(100% + var(--space-xxs)); width: stretch; text-align: center; }
.count { position: absolute; left: 0; top: calc(100% - var(--space-lg)); width: calc(100% - var(--space-xxs)); text-align: end; }
@media (width < 600px), (height < 500px) { .items { display: none; } [part="RpgActionBar.skill"] { width: var(--size-lg); height: var(--size-lg); } }
</style>
