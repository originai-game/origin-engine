<meta name="component" content="AbilitySlot">
<meta name="strings" content="./ability-slot.en.ogt">
<meta name="parts" content="AbilitySlot.frame AbilitySlot.cell AbilitySlot.well AbilitySlot.icon AbilitySlot.ultimate AbilitySlot.figure AbilitySlot.percent AbilitySlot.badge AbilitySlot.track AbilitySlot.fill AbilitySlot.label">
<meta name="prop:icon" content="string" data-doc="An icon of the atlas, drawn large on a gradient.">
<meta name="prop:name" content="string" data-doc="What a screen reader calls it (Fireball).">
<meta name="prop:action" content="string" data-doc="The action whose glyph the hotkey cap shows on the device used last (it follows a rebinding); empty is none.">
<meta name="prop:hotkey" content="string" data-doc="A label for the cap instead of the action's glyph (Q, LB); empty shows the action's.">
<meta name="prop:cooldown" content="number" data-doc="Seconds of cooldown left; 0 is ready.">
<meta name="prop:cooldownMax" content="number" data-default="1" data-doc="The cooldown's length; the sweep shows cooldown over it.">
<meta name="prop:charges" content="number" data-doc="Charges ready, in a corner badge when maxCharges is over 1.">
<meta name="prop:maxCharges" content="number" data-doc="Charges it holds; 0 or 1 is none.">
<meta name="prop:cost" content="string" data-doc="Its cost (40) at the bottom left while ready; empty is none.">
<meta name="prop:unaffordable" content="bool" data-doc="Not enough resource: the art tinted red and the cost in danger.">
<meta name="prop:outOfRange" content="bool" data-doc="The target is out of range: the art tinted red.">
<meta name="prop:active" content="bool" data-doc="Toggled on or channelling: a breathing glow and a primary edge.">
<meta name="prop:locked" content="bool" data-doc="Not learnt yet: dark art under a lock.">
<meta name="prop:disabled" content="bool" data-doc="Cannot be used now: the art washed out under a red slash.">
<meta name="prop:ultimate" content="number" data-default="-1" data-doc="An ultimate's charge 0 to 1: larger, a gold ring filling round it and a percent; negative is a plain slot.">
<meta name="prop:level" content="number" data-doc="Ranks learnt (pips right under it).">
<meta name="prop:maxLevel" content="number" data-doc="Ranks it has; 0 draws no pips.">
<meta name="action:use" content="Use">
<meta name="fixture:default" content='{"icon":"fire","name":"Fireball","action":"","hotkey":"Q","cooldown":4.2,"cooldownMax":8,"charges":2,"maxCharges":3,"cost":"40","unaffordable":false,"outOfRange":false,"active":false,"locked":false,"disabled":false,"ultimate":-1,"level":2,"maxLevel":3}'>

<div id="AbilitySlot" part="AbilitySlot.frame" class="bare" class:ultimate={ultimate >= 0}>
  <button id="Slot" part="AbilitySlot.cell" onclick={use} aria-label={name} aria-valuetext={locked ? t("component.ability-slot.locked") : disabled ? t("component.ability-slot.unavailable") : cooldown > 0 ? formatNumber(cooldown) : t("component.ability-slot.ready")} disabled={locked || disabled} class:cooling={cooldown > 0} class:active={active} class:tinted={unaffordable || outOfRange} class:locked={locked} class:off={disabled} class:full={ultimate >= 1}>
    <div id="Face">
      <div id="Art" part="AbilitySlot.well"><og-icon id="Icon" part="AbilitySlot.icon" icon={icon}></og-icon></div>
      <svg id="Ultimate" part="AbilitySlot.ultimate" viewBox="0 0 64 64" hidden={ultimate < 0}><circle cx="32" cy="32" r="29" fill="none" stroke-width="4"></circle><circle id="UltimateFill" cx="32" cy="32" r="29" fill="none" stroke-width="4" pathLength="1" stroke-dasharray={ultimate + " 1"} transform="rotate(-90 32 32)"></circle></svg>
      <svg id="Sweep" viewBox="0 0 56 56" hidden={!(cooldown > 0)}><circle cx="28" cy="28" r="20" fill="none" stroke-width="40" pathLength="1" stroke-dasharray={(cooldown / cooldownMax) + " 1"} transform="rotate(-90 28 28) scale(-1 1) translate(-56 0)"></circle></svg>
      <span id="Time" part="AbilitySlot.figure" hidden={!(cooldown > 0)}>{cooldown < 10 ? formatNumber(cooldown) : formatCountdown(cooldown)}</span>
      <span id="Percent" part="AbilitySlot.percent" hidden={!(ultimate >= 0 && ultimate < 1)}>{formatPercent(ultimate)}</span>
      <svg id="Slash" viewBox="0 0 56 56" hidden={!disabled}><path d="M8 48L48 8" stroke-width="5" stroke-linecap="round"></path></svg>
      <og-icon id="Lock" icon="lock" hidden={!locked}></og-icon>
      <span id="Cost" part="AbilitySlot.badge" class="cost" class:short={unaffordable} hidden={!cost || cooldown > 0 || locked}>{cost}</span>
      <span id="Charges" part="AbilitySlot.badge" class="charges" hidden={!(maxCharges > 1) || cooldown > 0}>{formatNumber(charges)}</span>
    </div>
  </button>
  <div id="Pips" part="AbilitySlot.track" hidden={!(maxLevel > 0)} style:--ranks={maxLevel} style:--level={level}><div id="Ranked" part="AbilitySlot.fill"></div></div>
  <kbd id="Hotkey" part="AbilitySlot.label" data-action={action} hidden={!hotkey && !action}>{hotkey}</kbd>
</div>

<style>
#Face { width: stretch; height: stretch; align-items: center; justify-content: center; }
#AbilitySlot { align-items: center; gap: var(--space-xxs); }
#Slot { width: var(--size-lg); height: var(--size-lg); padding: 0; overflow: hidden; transition-property: scale; transition-duration: var(--motion-duration-fast); transition-timing-function: var(--motion-easing-out); }
.ultimate #Slot { width: var(--size-xl); height: var(--size-xl); border-radius: var(--radius-capsule); }
#Art { width: stretch; height: stretch; align-items: center; justify-content: center; }
#Icon { width: var(--size-md); height: var(--size-md); color: var(--ink); }
#Slot.tinted #Art { background: var(--danger-45); }
#Slot.tinted #Icon { color: var(--danger); }
#Slot.locked #Art { filter: var(--filter-locked); }
#Slot.off #Art { filter: var(--filter-spent); }
#Slot.active { box-shadow: 0 0 var(--space-md) var(--primary), inset 0 0 0 var(--stroke-regular) var(--primary); animation-name: breathe; animation-duration: var(--motion-duration-bounce); animation-timing-function: var(--motion-easing-in-out); animation-iteration-count: infinite; animation-direction: alternate; }
#Slot.full { box-shadow: 0 0 var(--space-md) var(--gold), inset 0 0 0 var(--stroke-regular) var(--gold); animation-name: breathe; animation-duration: var(--motion-duration-bounce); animation-timing-function: var(--motion-easing-in-out); animation-iteration-count: infinite; animation-direction: alternate; }
#Ultimate { position: absolute; top: 0; inset-inline-start: 0; width: stretch; height: stretch; stroke: var(--scrim); }
#UltimateFill { stroke: var(--gold); }
#Sweep { position: absolute; top: 0; inset-inline-start: 0; width: stretch; height: stretch; stroke: var(--scrim); }
#Time { position: absolute; top: var(--space-md); inset-inline-start: 0; width: stretch; text-align: center; font-size: var(--font-step-xl); }
#Percent { position: absolute; bottom: var(--space-xxs); inset-inline-start: 0; width: stretch; text-align: center; font: var(--text-caption); color: var(--gold); }
#Slash { position: absolute; top: 0; inset-inline-start: 0; width: stretch; height: stretch; stroke: var(--danger); }
#Lock { position: absolute; top: var(--space-md); inset-inline-start: var(--space-md); width: var(--size-sm); height: var(--size-sm); color: var(--ink); }
#Cost { position: absolute; bottom: var(--space-xxs); inset-inline-start: var(--space-xxs); padding: 0 var(--space-xs); font-size: var(--font-step-sm); color: var(--mana); }
#Cost.short { color: var(--danger); }
#Charges { position: absolute; top: var(--space-xxs); inset-inline-end: var(--space-xxs); min-width: var(--size-xs); padding: 0 var(--space-xxs); font-size: var(--font-step-sm); align-items: center; }
#Pips { width: calc(var(--ranks) * var(--space-md) - var(--space-xs)); height: var(--space-xs); background: repeating-linear-gradient(90deg, var(--scrim) 0, var(--scrim) var(--space-sm), transparent var(--space-sm), transparent var(--space-md)); }
#Ranked { width: calc(var(--level) * var(--space-md) - var(--space-xs)); height: var(--space-xs); background: repeating-linear-gradient(90deg, var(--gold) 0, var(--gold) var(--space-sm), transparent var(--space-sm), transparent var(--space-md)); }
#Hotkey { font-size: var(--font-step-sm); }
@keyframes breathe { from { opacity: 1; } to { opacity: 0.8; } }
</style>
