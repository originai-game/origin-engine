<meta name="component" content="StatusGrid">
<meta name="strings" content="./status-grid.en.ogt">
<meta name="parts" content="StatusGrid.cell StatusGrid.badge StatusGrid.figure">
<meta name="prop:buffs" content="list" data-doc="The buffs, in the game's order: each { id, icon, name, stacks, remaining, duration, expiring }, remaining and duration in seconds (duration 0 lasts until removed), expiring in its last five seconds.">
<meta name="prop:debuffs" content="list" data-doc="The debuffs, in their own row and bordered by type: each as a buff with type magic, curse, poison, disease or bleed.">
<meta name="prop:more" content="number" data-doc="Effects past the shown ones, folded into a +n tile (0: none).">
<meta name="action:pick" content="Cancel">
<meta name="fixture:default" content='{"more":2,"buffs":[{"id":"haste","icon":"bolt","name":"Haste","stacks":0,"remaining":12,"duration":30,"expiring":false},{"id":"shield","icon":"shield","name":"Barrier","stacks":3,"remaining":4.2,"duration":10,"expiring":true},{"id":"regen","icon":"heart","name":"Regrowth","stacks":0,"remaining":0,"duration":0,"expiring":false}],"debuffs":[{"id":"burn","icon":"fire","name":"Burning","stacks":2,"remaining":6,"duration":8,"expiring":false,"type":"magic"},{"id":"venom","icon":"drop","name":"Venom","stacks":0,"remaining":18,"duration":20,"expiring":false,"type":"poison"}]}'>

<div id="StatusGrid">
  <ul id="Buffs" role="list" aria-label={t("component.status-grid.buffs")}>
    {#each buffs as e (e.id)}
      <li class="tile" class:expiring={e.expiring} aria-label={e.name}>
        <div class="face" part="StatusGrid.cell">
          <og-icon class="icon" icon={e.icon}></og-icon>
          <svg class="sweep" viewBox="0 0 32 32" hidden={!(e.duration > 0)}><circle cx="16" cy="16" r="8" fill="none" stroke-width="16" pathLength="1" stroke-dasharray={(1 - e.remaining / e.duration) + " 1"} transform="rotate(-90 16 16)"></circle></svg>
          <span class="stacks" part="StatusGrid.badge" hidden={!(e.stacks > 1)}>{formatNumber(e.stacks)}</span>
        </div>
        <span class="time" part="StatusGrid.figure" hidden={!(e.remaining > 0)}>{e.remaining < 10 ? formatNumber(e.remaining) : formatCountdown(e.remaining)}</span>
      </li>
    {/each}
    <li id="More" class="tile" hidden={!(more > 0)}><div class="face" part="StatusGrid.cell"><span id="Count">{"+" + formatNumber(more)}</span></div></li>
  </ul>
  <ul id="Debuffs" role="list" aria-label={t("component.status-grid.debuffs")}>
    {#each debuffs as e (e.id)}
      <li class="tile debuff" class:expiring={e.expiring} class:magic={e.type == 'magic'} class:curse={e.type == 'curse'} class:poison={e.type == 'poison'} class:disease={e.type == 'disease'} class:bleed={e.type == 'bleed'} aria-label={e.name}>
        <div class="face" part="StatusGrid.cell">
          <og-icon class="icon" icon={e.icon}></og-icon>
          <svg class="sweep" viewBox="0 0 32 32" hidden={!(e.duration > 0)}><circle cx="16" cy="16" r="8" fill="none" stroke-width="16" pathLength="1" stroke-dasharray={(1 - e.remaining / e.duration) + " 1"} transform="rotate(-90 16 16)"></circle></svg>
          <div class="notch"></div>
          <span class="stacks" part="StatusGrid.badge" hidden={!(e.stacks > 1)}>{formatNumber(e.stacks)}</span>
        </div>
        <span class="time" part="StatusGrid.figure" hidden={!(e.remaining > 0)}>{e.remaining < 10 ? formatNumber(e.remaining) : formatCountdown(e.remaining)}</span>
      </li>
    {/each}
  </ul>
</div>

<style>
ul, ol { list-style-type: none; }
#StatusGrid { gap: var(--space-xs); }
#Buffs, #Debuffs { display: flex; gap: var(--space-xs); flex-wrap: wrap; max-width: var(--measure-sm); }
.tile { align-items: center; gap: var(--space-xxs); width: calc(var(--size-md) + var(--space-xs)); --edge: var(--line-strong); }
.debuff { --edge: var(--danger); }
.magic { --edge: var(--info); }
.curse { --edge: var(--xp); }
.poison { --edge: var(--success); }
.disease { --edge: var(--warn); }
.bleed { --edge: var(--danger); }
/* A debuff's type is the cell's rim, through its token, so a pack's cell still dresses it. */
[part="StatusGrid.cell"] { width: var(--size-md); height: var(--size-md); overflow: hidden; align-items: center; justify-content: center; --cell-rim: var(--edge); }
.icon { width: var(--size-sm); height: var(--size-sm); }
.sweep { position: absolute; top: 0; inset-inline-start: 0; width: var(--size-md); height: var(--size-md); stroke: var(--scrim); }
.notch { position: absolute; top: 0; inset-inline-start: 0; width: var(--space-sm); height: var(--space-sm); background: var(--edge); clip-path: polygon(0% 0%, 100% 0%, 0% 100%); }
[part="StatusGrid.badge"] { position: absolute; bottom: 0; inset-inline-end: 0; padding: 0 var(--space-xxs); font: var(--text-caption); font-family: var(--figure-font); }
[part="StatusGrid.figure"] { font: var(--text-caption); font-family: var(--figure-font); white-space: nowrap; text-shadow: 0 var(--stroke-hairline) var(--stroke-regular) var(--scene-shadow); }
.expiring [part="StatusGrid.cell"] { animation-name: blink; animation-duration: var(--motion-duration-spring); animation-timing-function: var(--motion-easing-in-out); animation-iteration-count: infinite; animation-direction: alternate; }
#Count { font: var(--text-sm); color: var(--ink-muted); }
@keyframes blink { from { opacity: 1; } to { opacity: 0.35; } }
</style>
