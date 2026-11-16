<meta name="component" content="AbilityBar">
<meta name="strings" content="./ability-bar.en.ogt">
<meta name="parts" content="AbilityBar.frame AbilityBar.cell AbilityBar.well AbilityBar.icon AbilityBar.figure AbilityBar.badge AbilityBar.label">
<meta name="prop:slots" content="list" data-doc="The slots, in order: each { id, icon, name, action, hotkey, cooldown, cooldownMax, cost, unaffordable, locked, active, count }, cooldown in seconds (0: ready); count above 0 shows how many are left (a quick item: potions, scrolls). A slot's full set of states is the AbilitySlot component.">
<meta name="prop:column" content="bool" data-doc="The slots run top to bottom; false is a row.">
<meta name="action:use" content="Use">
<meta name="fixture:default" content='{"column":false,"slots":[{"id":"q","icon":"sword","name":"Cleave","action":"","hotkey":"Q","cooldown":0,"cooldownMax":6,"cost":"","unaffordable":false,"locked":false,"active":false},{"id":"w","icon":"shield","name":"Guard","action":"","hotkey":"W","cooldown":3.4,"cooldownMax":10,"cost":"30","unaffordable":false,"locked":false,"active":false},{"id":"e","icon":"bolt","name":"Dash","action":"","hotkey":"E","cooldown":0,"cooldownMax":5,"cost":"60","unaffordable":true,"locked":false,"active":false},{"id":"r","icon":"fire","name":"Inferno","action":"","hotkey":"R","cooldown":0,"cooldownMax":60,"cost":"","unaffordable":false,"locked":true,"active":false},{"id":"t","icon":"potion","name":"Elixir","action":"","hotkey":"1","cooldown":0,"cooldownMax":0,"cost":"","unaffordable":false,"locked":false,"active":true,"count":9}]}'>

<ul id="AbilityBar" part="AbilityBar.frame" class="bare" role="toolbar" aria-label={t("component.ability-bar.name")} class:column={column}>
  {#each slots as s (s.id)}
    <li class="cell">
      <button part="AbilityBar.cell" class="slot" onclick={use} aria-label={s.name} aria-valuetext={s.locked ? t("component.ability-bar.locked") : s.cooldown > 0 ? formatNumber(s.cooldown) : t("component.ability-bar.ready")} disabled={s.locked} class:cooling={s.cooldown > 0} class:active={s.active} class:tinted={s.unaffordable} class:locked={s.locked}>
        <div class="face">
          <div part="AbilityBar.well" class="art"><og-icon part="AbilityBar.icon" class="icon" icon={s.icon}></og-icon></div>
          {#if s.cooldown > 0}
            <svg class="sweep" viewBox="0 0 56 56"><circle cx="28" cy="28" r="20" fill="none" stroke-width="40" pathLength="1" stroke-dasharray={(s.cooldown / s.cooldownMax) + " 1"} transform="rotate(-90 28 28) scale(-1 1) translate(-56 0)"></circle></svg>
            <span part="AbilityBar.figure" class="time">{s.cooldown < 10 ? formatNumber(s.cooldown) : formatCountdown(s.cooldown)}</span>
          {/if}
          {#if s.locked}<og-icon class="lock" icon="lock"></og-icon>{/if}
          {#if s.count > 0}<span part="AbilityBar.figure" class="count">{formatNumber(s.count)}</span>{/if}
          {#if s.cost && s.cooldown <= 0 && !s.locked}<span part="AbilityBar.badge" class="cost" class:short={s.unaffordable}>{s.cost}</span>{/if}
        </div>
      </button>
      {#if s.hotkey || s.action}<kbd part="AbilityBar.label" class="hotkey" data-action={s.action}>{s.hotkey}</kbd>{/if}
    </li>
  {/each}
</ul>

<style>
:host { text-shadow: var(--tag-halo); }
.face { position: relative; display: flex; flex-direction: column; inline-size: 100%; block-size: 100%; align-items: center; justify-content: center; }
/* The slots grow with the player's text scale, as AbilitySlot's do:
   --text-px is one CSS px at 100 % text. */
:host { --text-px: var(--font-step-px); display: flex; margin: 0; padding: 0; list-style-type: none; gap: calc(8 * var(--text-px)); flex-wrap: wrap; justify-content: center; max-inline-size: min(100%, var(--measure-xl)); }
.column { flex-direction: column; flex-wrap: nowrap; }
.cell { display: flex; flex-direction: column; align-items: center; gap: var(--space-xxs); }
.slot { width: calc(48 * var(--text-px)); height: calc(48 * var(--text-px)); padding: 0; overflow: hidden; }
.art { display: flex; flex-direction: column; inline-size: 100%; block-size: 100%; align-items: center; justify-content: center; }
.icon { width: calc(32 * var(--text-px)); height: calc(32 * var(--text-px)); color: var(--ink); }
.slot.tinted .art { background: var(--danger-45); }
.slot.tinted .icon { color: var(--danger); }
.slot.locked .art { filter: var(--filter-locked); }
.slot.active { box-shadow: 0 0 var(--space-md) var(--primary), inset 0 0 0 var(--stroke-regular) var(--primary); animation-name: breathe; animation-duration: var(--motion-duration-bounce); animation-timing-function: var(--motion-easing-in-out); animation-iteration-count: infinite; animation-direction: alternate; }
.sweep { position: absolute; inset: 0; inline-size: 100%; block-size: 100%; stroke: var(--scrim); }
.time { position: absolute; top: calc(12 * var(--text-px)); inset-inline: 0; text-align: center; font-size: var(--font-step-xl); font-family: var(--figure-font); color: var(--scene-ink); text-shadow: var(--tag-halo); }
.lock { position: absolute; top: calc(12 * var(--text-px)); inset-inline-start: calc(12 * var(--text-px)); width: calc(24 * var(--text-px)); height: calc(24 * var(--text-px)); color: var(--ink); }
.cost { position: absolute; bottom: var(--space-xxs); inset-inline-start: var(--space-xxs); padding: 0 var(--space-xs); font-size: var(--font-step-sm); color: var(--mana); }
.cost.short { color: var(--danger); }
.count { position: absolute; bottom: var(--space-xxs); inset-inline-end: var(--space-xxs); font-size: var(--font-step-sm); font-family: var(--figure-font); color: var(--scene-ink); text-shadow: var(--tag-halo); }
.hotkey { font-size: var(--font-step-sm); }
/* The cap sits under the slot, over the scene: it is the scene's plate in
   the scene's ink, the pair a pack sets for text over the scene, so it reads
   over a bright sky as over a dark floor. */
[part="AbilityBar.label"] { background: var(--scene-plate); color: var(--scene-ink); }
@keyframes breathe { from { opacity: 1; } to { opacity: 0.8; } }
</style>
