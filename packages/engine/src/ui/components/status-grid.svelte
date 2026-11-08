<meta name="component" content="StatusGrid">
<meta name="strings" content="./status-grid.en.ogt">
<meta name="prop:buffs" content="list" data-doc="The buffs, in the game's order: each { id, icon, name, stacks, remaining, duration, expiring }, remaining and duration in seconds (duration 0 lasts until removed), expiring in its last five seconds.">
<meta name="prop:debuffs" content="list" data-doc="The debuffs, in their own row and bordered by type: each as a buff with type magic, curse, poison, disease or bleed.">
<meta name="prop:more" content="number" data-doc="Effects past the shown ones, folded into a +n tile (0: none).">
<meta name="action:pick" content="Cancel">
<meta name="fixture:default" content='{"more":2,"buffs":[{"id":"haste","icon":"bolt","name":"Haste","stacks":0,"remaining":12,"duration":30,"expiring":false},{"id":"shield","icon":"shield","name":"Barrier","stacks":3,"remaining":4.2,"duration":10,"expiring":true},{"id":"regen","icon":"heart","name":"Regrowth","stacks":0,"remaining":0,"duration":0,"expiring":false}],"debuffs":[{"id":"burn","icon":"fire","name":"Burning","stacks":2,"remaining":6,"duration":8,"expiring":false,"type":"magic"},{"id":"venom","icon":"drop","name":"Venom","stacks":0,"remaining":18,"duration":20,"expiring":false,"type":"poison"}]}'>

<div id="StatusGrid">
  <ul id="Buffs" role="list" aria-label={t("component.status-grid.buffs")}>
    {#each buffs as e (e.id)}
      <li class="tile" class:expiring={e.expiring} aria-label={e.name}>
        <div class="face">
          <icon class="icon" icon={e.icon}></icon>
          <svg class="sweep" viewBox="0 0 32 32" hidden={!(e.duration > 0)}><circle cx="16" cy="16" r="8" fill="none" stroke="#0000009e" stroke-width="16" pathLength="1" stroke-dasharray={(1 - e.remaining / e.duration) + " 1"} transform="rotate(-90 16 16)"></circle></svg>
          <span class="stacks" hidden={!(e.stacks > 1)}>{formatNumber(e.stacks)}</span>
        </div>
        <span class="time" hidden={!(e.remaining > 0)}>{e.remaining < 10 ? formatNumber(e.remaining) : formatCountdown(e.remaining)}</span>
      </li>
    {/each}
    <li id="More" class="tile" hidden={!(more > 0)}><div class="face"><span id="Count">{"+" + formatNumber(more)}</span></div></li>
  </ul>
  <ul id="Debuffs" role="list" aria-label={t("component.status-grid.debuffs")}>
    {#each debuffs as e (e.id)}
      <li class="tile debuff" class:expiring={e.expiring} class:magic={e.type == 'magic'} class:curse={e.type == 'curse'} class:poison={e.type == 'poison'} class:disease={e.type == 'disease'} class:bleed={e.type == 'bleed'} aria-label={e.name}>
        <div class="face">
          <icon class="icon" icon={e.icon}></icon>
          <svg class="sweep" viewBox="0 0 32 32" hidden={!(e.duration > 0)}><circle cx="16" cy="16" r="8" fill="none" stroke="#0000009e" stroke-width="16" pathLength="1" stroke-dasharray={(1 - e.remaining / e.duration) + " 1"} transform="rotate(-90 16 16)"></circle></svg>
          <div class="notch"></div>
          <span class="stacks" hidden={!(e.stacks > 1)}>{formatNumber(e.stacks)}</span>
        </div>
        <span class="time" hidden={!(e.remaining > 0)}>{e.remaining < 10 ? formatNumber(e.remaining) : formatCountdown(e.remaining)}</span>
      </li>
    {/each}
  </ul>
</div>

<style>
ul, ol { list-style-type: none; }
#StatusGrid { gap: 6px; }
#Buffs, #Debuffs { display: flex; gap: 6px; flex-wrap: wrap; max-width: 320px; }
.tile { align-items: center; gap: 2px; width: 36px; --edge: #ffffff40; }
.debuff { --edge: var(--danger); }
.magic { --edge: #3d8bffff; }
.curse { --edge: #a335eeff; }
.poison { --edge: #33cc4dff; }
.disease { --edge: #c69b3aff; }
.bleed { --edge: #c41e3aff; }
.face { width: 32px; height: 32px; border-radius: 4px; overflow: hidden; background: var(--control-strong); border: 1px solid var(--edge); align-items: center; justify-content: center; }
.icon { width: 24px; height: 24px; }
.sweep { position: absolute; top: 0; inset-inline-start: 0; width: 32px; height: 32px; }
.notch { position: absolute; top: 0; inset-inline-start: 0; width: 8px; height: 8px; background: var(--edge); clip-path: polygon(0% 0%, 100% 0%, 0% 100%); }
.stacks { position: absolute; bottom: 0; inset-inline-start: 18px; font: var(--text-caption); color: var(--ink); text-shadow: 0 1px 2px #000000ff; }
.time { font: var(--text-caption); color: var(--ink); text-shadow: 0 1px 2px #000000ff; white-space: nowrap; }
.expiring .face { animation: blink 500ms ease-in-out infinite alternate; }
#Count { font: var(--text-sm); color: var(--ink-muted); }
@keyframes blink { from { opacity: 1; } to { opacity: 0.35; } }
</style>
