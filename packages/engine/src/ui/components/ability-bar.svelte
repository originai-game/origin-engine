<meta name="component" content="AbilityBar">
<meta name="strings" content="./ability-bar.en.ogt">
<meta name="prop:slots" content="list" data-doc="The slots, in order: each { id, icon, name, action, hotkey, cooldown, cooldownMax, cost, unaffordable, locked, active }, cooldown in seconds (0: ready). A slot's full set of states is the AbilitySlot component.">
<meta name="prop:column" content="bool" data-doc="The slots run top to bottom; false is a row.">
<meta name="action:use" content="Use">
<meta name="fixture:default" content='{"column":false,"slots":[{"id":"q","icon":"sword","name":"Cleave","action":"","hotkey":"Q","cooldown":0,"cooldownMax":6,"cost":"","unaffordable":false,"locked":false,"active":false},{"id":"w","icon":"shield","name":"Guard","action":"","hotkey":"W","cooldown":3.4,"cooldownMax":10,"cost":"30","unaffordable":false,"locked":false,"active":false},{"id":"e","icon":"bolt","name":"Dash","action":"","hotkey":"E","cooldown":0,"cooldownMax":5,"cost":"60","unaffordable":true,"locked":false,"active":false},{"id":"r","icon":"fire","name":"Inferno","action":"","hotkey":"R","cooldown":0,"cooldownMax":60,"cost":"","unaffordable":false,"locked":true,"active":false},{"id":"t","icon":"potion","name":"Elixir","action":"","hotkey":"1","cooldown":0,"cooldownMax":0,"cost":"","unaffordable":false,"locked":false,"active":true}]}'>

<ul id="AbilityBar" role="toolbar" aria-label={t("component.ability-bar.name")} class:column={column}>
  {#each slots as s (s.id)}
    <li class="cell">
      <button class="slot" onclick={use} aria-label={s.name} aria-valuetext={s.locked ? t("component.ability-bar.locked") : s.cooldown > 0 ? formatNumber(s.cooldown) : t("component.ability-bar.ready")} disabled={s.locked} class:cooling={s.cooldown > 0} class:active={s.active} class:tinted={s.unaffordable} class:locked={s.locked}>
        <div class="face">
          <div class="art"><icon class="icon" icon={s.icon}></icon></div>
          <svg class="sweep" viewBox="0 0 56 56" hidden={!(s.cooldown > 0)}><circle cx="28" cy="28" r="20" fill="none" stroke="#00000099" stroke-width="40" pathLength="1" stroke-dasharray={(s.cooldown / s.cooldownMax) + " 1"} transform="rotate(-90 28 28) scale(-1 1) translate(-56 0)"></circle></svg>
          <span class="time" hidden={!(s.cooldown > 0)}>{s.cooldown < 10 ? formatNumber(s.cooldown) : formatCountdown(s.cooldown)}</span>
          <icon class="lock" icon="lock" hidden={!s.locked}></icon>
          <span class="cost" class:short={s.unaffordable} hidden={!s.cost || s.cooldown > 0 || s.locked}>{s.cost}</span>
        </div>
      </button>
      <kbd class="hotkey" data-action={s.action} hidden={!s.hotkey && !s.action}>{s.hotkey}</kbd>
    </li>
  {/each}
</ul>

<style>
.face { width: stretch; height: stretch; align-items: center; justify-content: center; }
ul, ol { list-style-type: none; }
#AbilityBar { display: flex; gap: 8px; flex-wrap: wrap; justify-content: center; max-width: 560px; }
#AbilityBar.column { flex-direction: column; flex-wrap: nowrap; }
.cell { align-items: center; gap: 3px; }
.slot { width: 56px; height: 56px; padding: 0; border-radius: 8px; overflow: hidden; border: 1px solid #ffffff4d; background: var(--control-strong); box-shadow: inset 0 2px 4px #00000080, 0 3px 6px #00000080; }
.art { width: stretch; height: stretch; align-items: center; justify-content: center; background: radial-gradient(circle at 50% 30%, #5b8cffff, #1a2a6cff); }
.icon { width: 34px; height: 34px; color: #ffffffff; }
.slot.tinted .art { background: radial-gradient(circle at 50% 30%, #c0392bff, #4a0e0eff); }
.slot.tinted .icon { color: #ff8080ff; }
.slot.locked .art { filter: grayscale(1) brightness(0.35); }
.slot.active { box-shadow: 0 0 12px var(--primary), inset 0 0 0 2px var(--primary); animation: breathe 1400ms ease-in-out infinite alternate; }
.sweep { position: absolute; top: 0; inset-inline-start: 0; width: stretch; height: stretch; }
.time { position: absolute; top: 16px; inset-inline-start: 0; width: stretch; text-align: center; font: var(--text-lg); color: #ffffffff; text-shadow: 0 0 3px #000000ff; }
.lock { position: absolute; top: 16px; inset-inline-start: 16px; width: 24px; height: 24px; color: var(--ink); }
.cost { position: absolute; bottom: 2px; inset-inline-start: 2px; padding: 0 4px; border-radius: 6px; background: #000000b3; font: var(--text-caption); color: var(--mana); }
.cost.short { color: var(--danger); }
.hotkey { font: var(--text-caption); color: var(--ink-muted); }
@keyframes breathe { from { opacity: 1; } to { opacity: 0.8; } }
</style>
