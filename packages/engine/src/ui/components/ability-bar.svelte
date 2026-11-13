<meta name="component" content="AbilityBar">
<meta name="strings" content="./ability-bar.en.ogt">
<meta name="parts" content="AbilityBar.frame AbilityBar.cell AbilityBar.well AbilityBar.icon AbilityBar.figure AbilityBar.badge AbilityBar.label">
<meta name="prop:slots" content="list" data-doc="The slots, in order: each { id, icon, name, action, hotkey, cooldown, cooldownMax, cost, unaffordable, locked, active }, cooldown in seconds (0: ready). A slot's full set of states is the AbilitySlot component.">
<meta name="prop:column" content="bool" data-doc="The slots run top to bottom; false is a row.">
<meta name="action:use" content="Use">
<meta name="fixture:default" content='{"column":false,"slots":[{"id":"q","icon":"sword","name":"Cleave","action":"","hotkey":"Q","cooldown":0,"cooldownMax":6,"cost":"","unaffordable":false,"locked":false,"active":false},{"id":"w","icon":"shield","name":"Guard","action":"","hotkey":"W","cooldown":3.4,"cooldownMax":10,"cost":"30","unaffordable":false,"locked":false,"active":false},{"id":"e","icon":"bolt","name":"Dash","action":"","hotkey":"E","cooldown":0,"cooldownMax":5,"cost":"60","unaffordable":true,"locked":false,"active":false},{"id":"r","icon":"fire","name":"Inferno","action":"","hotkey":"R","cooldown":0,"cooldownMax":60,"cost":"","unaffordable":false,"locked":true,"active":false},{"id":"t","icon":"potion","name":"Elixir","action":"","hotkey":"1","cooldown":0,"cooldownMax":0,"cost":"","unaffordable":false,"locked":false,"active":true}]}'>

<ul id="AbilityBar" part="AbilityBar.frame" class="bare" role="toolbar" aria-label={t("component.ability-bar.name")} class:column={column}>
  {#each slots as s (s.id)}
    <li class="cell">
      <button part="AbilityBar.cell" class="slot" onclick={use} aria-label={s.name} aria-valuetext={s.locked ? t("component.ability-bar.locked") : s.cooldown > 0 ? formatNumber(s.cooldown) : t("component.ability-bar.ready")} disabled={s.locked} class:cooling={s.cooldown > 0} class:active={s.active} class:tinted={s.unaffordable} class:locked={s.locked}>
        <div class="face">
          <div part="AbilityBar.well" class="art"><og-icon part="AbilityBar.icon" class="icon" icon={s.icon}></og-icon></div>
          <svg class="sweep" viewBox="0 0 56 56" hidden={!(s.cooldown > 0)}><circle cx="28" cy="28" r="20" fill="none" stroke-width="40" pathLength="1" stroke-dasharray={(s.cooldown / s.cooldownMax) + " 1"} transform="rotate(-90 28 28) scale(-1 1) translate(-56 0)"></circle></svg>
          <span part="AbilityBar.figure" class="time" hidden={!(s.cooldown > 0)}>{s.cooldown < 10 ? formatNumber(s.cooldown) : formatCountdown(s.cooldown)}</span>
          <og-icon class="lock" icon="lock" hidden={!s.locked}></og-icon>
          <span part="AbilityBar.badge" class="cost" class:short={s.unaffordable} hidden={!s.cost || s.cooldown > 0 || s.locked}>{s.cost}</span>
        </div>
      </button>
      <kbd part="AbilityBar.label" class="hotkey" data-action={s.action} hidden={!s.hotkey && !s.action}>{s.hotkey}</kbd>
    </li>
  {/each}
</ul>

<style>
.face { width: stretch; height: stretch; align-items: center; justify-content: center; }
ul, ol { list-style-type: none; }
#AbilityBar { display: flex; gap: var(--space-sm); flex-wrap: wrap; justify-content: center; max-width: var(--measure-xl); }
#AbilityBar.column { flex-direction: column; flex-wrap: nowrap; }
.cell { align-items: center; gap: var(--space-xxs); }
.slot { width: var(--size-lg); height: var(--size-lg); padding: 0; overflow: hidden; }
.art { width: stretch; height: stretch; align-items: center; justify-content: center; }
.icon { width: var(--size-md); height: var(--size-md); color: var(--ink); }
.slot.tinted .art { background: var(--danger-45); }
.slot.tinted .icon { color: var(--danger); }
.slot.locked .art { filter: var(--filter-locked); }
.slot.active { box-shadow: 0 0 var(--space-md) var(--primary), inset 0 0 0 var(--stroke-regular) var(--primary); animation-name: breathe; animation-duration: var(--motion-duration-bounce); animation-timing-function: var(--motion-easing-in-out); animation-iteration-count: infinite; animation-direction: alternate; }
.sweep { position: absolute; top: 0; inset-inline-start: 0; width: stretch; height: stretch; stroke: var(--scrim); }
.time { position: absolute; top: var(--space-md); inset-inline-start: 0; width: stretch; text-align: center; font-size: var(--font-step-xl); }
.lock { position: absolute; top: var(--space-md); inset-inline-start: var(--space-md); width: var(--size-sm); height: var(--size-sm); color: var(--ink); }
.cost { position: absolute; bottom: var(--space-xxs); inset-inline-start: var(--space-xxs); padding: 0 var(--space-xs); font-size: var(--font-step-sm); color: var(--mana); }
.cost.short { color: var(--danger); }
.hotkey { font-size: var(--font-step-sm); }
@keyframes breathe { from { opacity: 1; } to { opacity: 0.8; } }
</style>
