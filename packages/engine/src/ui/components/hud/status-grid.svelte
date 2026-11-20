<meta name="component" content="StatusGrid">
<meta name="strings" content="./status-grid.en.ogt">
<meta name="prop:buffs" content="list" data-doc="The buffs, in the game's order: each { id, icon, name, stacks, remaining, duration, expiring }, remaining and duration in seconds (duration 0 lasts until removed; remaining past duration, an extended effect, sweeps nothing), expiring in its last five seconds.">
<meta name="prop:debuffs" content="list" data-doc="The debuffs, in their own row and bordered by type: each as a buff with type magic, curse, poison, disease or bleed.">
<meta name="prop:more" content="number" data-doc="Effects past the shown ones, folded into a +n tile (0: none).">
<meta name="action:pick" content="Cancel">
<meta name="fixture:default" content='{"more":2,"buffs":[{"id":"haste","icon":"bolt","name":"Haste","stacks":0,"remaining":12,"duration":30,"expiring":false},{"id":"shield","icon":"shield","name":"Barrier","stacks":3,"remaining":4.2,"duration":10,"expiring":true},{"id":"regen","icon":"heart","name":"Regrowth","stacks":0,"remaining":0,"duration":0,"expiring":false}],"debuffs":[{"id":"burn","icon":"fire","name":"Burning","stacks":2,"remaining":6,"duration":8,"expiring":false,"type":"magic"},{"id":"venom","icon":"drop","name":"Venom","stacks":0,"remaining":18,"duration":20,"expiring":false,"type":"poison"}]}'>

<div id="StatusGrid" layout="stack" gap="tight">
  <ul id="Buffs" layout="wrap" gap="loose" role="list" aria-label={t("component.status-grid.buffs")}>
    {#each buffs as e (e.id)}
      <li class="tile" layout="stack" gap="hair" class:expiring={e.expiring} aria-label={e.name}>
        <div class="face cell cover" part="cell" layout="stack" fill gap="none">
          <og-icon class="icon" play={e.expiring ? 'pulse' : null} icon={e.icon}></og-icon>
          {#if e.duration > 0}<svg class="sweep" anchor="fill" viewBox="0 0 32 32"><circle part="sweep" cx="16" cy="16" r="8" fill="none" stroke-width="16" pathLength="1" stroke-dasharray={(e.remaining >= e.duration ? 0 : 1 - e.remaining / e.duration) + " 1"} transform="rotate(-90 16 16)"></circle></svg>{/if}
        </div>
        {#if e.stacks > 1 || e.remaining > 0}<div class="details" layout="row" gap="tight">
          {#if e.stacks > 1}<span class="stacks badge" part="tag">{t("component.status-grid.stacks", { count: e.stacks })}</span>{/if}
          {#if e.remaining > 0}<div class="time" part="tag"><span class="number figure" part="figure">{e.remaining < 10 ? t("component.status-grid.seconds", { value: e.remaining }) : formatCountdown(e.remaining)}</span></div>{/if}
        </div>{/if}
      </li>
    {/each}
    {#if more > 0}<li id="More" class="tile" layout="stack" gap="hair"><div class="face cell cover" part="cell" layout="stack" fill gap="none"><span id="Count" class="count figure" part="figure">{"+" + formatNumber(more)}</span></div></li>{/if}
  </ul>
  <ul id="Debuffs" layout="wrap" gap="loose" role="list" aria-label={t("component.status-grid.debuffs")}>
    {#each debuffs as e (e.id)}
      <li class="tile debuff" layout="stack" gap="hair" class:expiring={e.expiring} class:magic={e.type == 'magic'} class:curse={e.type == 'curse'} class:poison={e.type == 'poison'} class:disease={e.type == 'disease'} class:bleed={e.type == 'bleed'} aria-label={e.name}>
        <div class="face cell cover" part="cell" layout="stack" fill gap="none">
          <og-icon class="icon" play={e.expiring ? 'pulse' : null} icon={e.icon}></og-icon>
          {#if e.duration > 0}<svg class="sweep" anchor="fill" viewBox="0 0 32 32"><circle part="sweep" cx="16" cy="16" r="8" fill="none" stroke-width="16" pathLength="1" stroke-dasharray={(e.remaining >= e.duration ? 0 : 1 - e.remaining / e.duration) + " 1"} transform="rotate(-90 16 16)"></circle></svg>{/if}
          <div class="notch" part="dot" anchor="top-start"></div>
        </div>
        {#if e.stacks > 1 || e.remaining > 0}<div class="details" layout="row" gap="tight">
          {#if e.stacks > 1}<span class="stacks badge" part="tag">{t("component.status-grid.stacks", { count: e.stacks })}</span>{/if}
          {#if e.remaining > 0}<div class="time" part="tag"><span class="number figure" part="figure">{e.remaining < 10 ? t("component.status-grid.seconds", { value: e.remaining }) : formatCountdown(e.remaining)}</span></div>{/if}
        </div>{/if}
      </li>
    {/each}
  </ul>
</div>

<style>
.cover { justify-content: center; align-items: center; }
/* Drawn over the scene: the style's tag halo holds its text there. */
#StatusGrid { text-shadow: var(--tag-halo); --scene-halo: var(--tag-halo); }
#Buffs, #Debuffs { max-width: var(--measure-sm); margin: 0; padding: 0; list-style: none; align-items: flex-start; }
.tile { align-items: center; min-width: calc(var(--size-md) + var(--space-xs)); --edge: var(--cell-scene-rim); }
.debuff { --edge: var(--danger); }
.magic { --edge: var(--info); }
.curse { --edge: var(--xp); }
.poison { --edge: var(--success); }
.disease { --edge: var(--warn); }
.bleed { --edge: var(--danger); }
/* A debuff's type is the cell's rim, through its token, so a pack's cell still dresses it. */
.cell { position: relative; width: var(--size-md); height: var(--size-md); overflow: hidden; --cell-fill: var(--cell-scene-fill); --cell-shadow: var(--cell-scene-shadow); --cell-rim: var(--edge); --layout-anchor-inset: 0; }
.icon { width: var(--size-sm); height: var(--size-sm); }
.sweep { width: var(--size-md); height: var(--size-md); }
.notch { width: var(--space-sm); height: var(--space-sm); --dot-fill: var(--edge); --dot-rim-width: 0; clip-path: polygon(0% 0%, 100% 0%, 0% 100%); }
.badge { padding: 0 var(--space-xxs); }
.figure { --figure-size: var(--font-step-sm); white-space: nowrap; }
#Count { --figure-shadow: var(--tag-halo); }
</style>
