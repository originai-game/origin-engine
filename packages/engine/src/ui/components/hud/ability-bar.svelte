<meta name="component" content="AbilityBar">
<meta name="strings" content="./ability-bar.en.ogt">
<meta name="parts" content="AbilityBar.frame AbilityBar.cell AbilityBar.well AbilityBar.icon AbilityBar.figure AbilityBar.badge AbilityBar.label AbilityBar.sweep">
<meta name="prop:slots" content="list" data-doc="The slots, in order: each { id, icon, name, action, hotkey, cooldown, cooldownMax, cost, unaffordable, locked, active, count }, cooldown in seconds (0: ready); count above 0 shows how many are left (a quick item: potions, scrolls). A slot's full set of states is the AbilitySlot component.">
<meta name="prop:column" content="bool" data-doc="The slots run top to bottom; false is a row.">
<meta name="action:use" content="Use">
<meta name="fixture:default" content='{"column":false,"slots":[{"id":"q","icon":"sword","name":"Cleave","action":"","hotkey":"Q","cooldown":0,"cooldownMax":6,"cost":"","unaffordable":false,"locked":false,"active":false},{"id":"w","icon":"shield","name":"Guard","action":"","hotkey":"W","cooldown":3.4,"cooldownMax":10,"cost":"30","unaffordable":false,"locked":false,"active":false},{"id":"e","icon":"bolt","name":"Dash","action":"","hotkey":"E","cooldown":0,"cooldownMax":5,"cost":"60","unaffordable":true,"locked":false,"active":false},{"id":"r","icon":"fire","name":"Inferno","action":"","hotkey":"R","cooldown":0,"cooldownMax":60,"cost":"","unaffordable":false,"locked":true,"active":false},{"id":"t","icon":"potion","name":"Elixir","action":"","hotkey":"1","cooldown":0,"cooldownMax":0,"cost":"","unaffordable":false,"locked":false,"active":true,"count":9}]}'>

<ul id="AbilityBar" part="AbilityBar.frame" class="bare" data-layout={column ? 'stack' : 'cluster'} data-gap="tight" role="toolbar" aria-label={t("component.ability-bar.name")} class:column={column}>
  {#each slots as s (s.id)}
    <li class="cell" data-layout="stack" data-gap="hair">
      <button part="AbilityBar.cell" class="slot" data-active={s.active} data-locked={s.locked} data-motion={s.active ? 'pulse' : null} onclick={use} aria-label={s.name} aria-valuetext={s.locked ? t("component.ability-bar.locked") : s.cooldown > 0 ? formatNumber(s.cooldown) : t("component.ability-bar.ready")} disabled={s.locked} class:cooling={s.cooldown > 0} class:active={s.active} class:tinted={s.unaffordable} class:locked={s.locked}>
        <div class="face" data-layout="overlay">
          <div part="AbilityBar.well" class="art" data-layout="cover" data-gap="none" data-anchor="fill"><og-icon part="AbilityBar.icon" class="icon" icon={s.icon}></og-icon></div>
          {#if s.cooldown > 0}
            <svg class="sweep" data-anchor="fill" viewBox="0 0 56 56"><circle part="AbilityBar.sweep" cx="28" cy="28" r="20" fill="none" stroke-width="40" pathLength="1" stroke-dasharray={(s.cooldown / s.cooldownMax) + " 1"} transform="rotate(-90 28 28) scale(-1 1) translate(-56 0)"></circle></svg>
            <span part="AbilityBar.figure" class="time" data-anchor="center">{s.cooldown < 10 ? formatNumber(s.cooldown) : formatCountdown(s.cooldown)}</span>
          {/if}
          {#if s.locked}<og-icon class="lock" data-anchor="center" icon="lock"></og-icon>{/if}
          {#if s.count > 0}<span part="AbilityBar.figure" class="count" data-anchor="bottom-end">{formatNumber(s.count)}</span>{/if}
          {#if s.cost && s.cooldown <= 0 && !s.locked}<span part="AbilityBar.badge" class="cost" data-anchor="bottom-start" class:short={s.unaffordable}>{s.cost}</span>{/if}
        </div>
      </button>
      {#if s.hotkey || s.action}<kbd part="AbilityBar.label AbilityBar.badge" class="hotkey" data-action={s.action}>{s.hotkey}</kbd>{/if}
    </li>
  {/each}
</ul>

<style>
:host { text-shadow: var(--tag-halo); --scene-halo: var(--tag-halo); }
.face { inline-size: 100%; block-size: 100%; --layout-anchor-inset: var(--space-xxs); }
/* The slots grow with the player's text scale, as AbilitySlot's do:
   --text-px is one CSS px at 100 % text. */
:host { --text-px: var(--font-step-px); margin: 0; padding: 0; list-style-type: none; justify-content: center; max-inline-size: min(100%, var(--measure-xl)); }
.cell { align-items: center; }
/* Over the scene the slot is the style's scene cell: a socket in a plate. */
.slot { width: calc(48 * var(--text-px)); height: calc(48 * var(--text-px)); padding: 0; overflow: hidden; --cell-fill: var(--cell-scene-fill); --cell-rim: var(--cell-scene-rim); --cell-shadow: var(--cell-scene-shadow); --cell-empty-shadow: var(--cell-scene-shadow); }
.art { inset: var(--cell-scene-inset); --well-radius: var(--cell-scene-well-radius); }
.icon { width: calc(32 * var(--text-px)); height: calc(32 * var(--text-px)); }
.slot.tinted .art { --well-fill: var(--well-alert-fill); }
.sweep { inline-size: 100%; block-size: 100%; --layout-anchor-inset: 0; }
.time { --figure-size: var(--font-step-xl); --figure-ink: var(--scene-ink); --figure-shadow: var(--tag-halo); }
.lock { width: calc(24 * var(--text-px)); height: calc(24 * var(--text-px)); }
.cost { --badge-ink: var(--mana); --badge-fill: var(--scene-plate); }
.cost.short { --badge-ink: var(--tone-danger); }
.count { --figure-size: var(--font-step-sm); --figure-ink: var(--scene-ink); --figure-shadow: var(--tag-halo); }
/* The cap sits under the slot, over the scene: it is the scene's plate in
   the scene's ink, the pair a pack sets for text over the scene, so it reads
   over a bright sky as over a dark floor. */
.hotkey { --badge-fill: var(--scene-plate); --badge-ink: var(--scene-ink); }
</style>
