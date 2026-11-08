<meta name="component" content="AbilitySlot">
<meta name="strings" content="./ability-slot.en.ogt">
<meta name="prop:icon" content="string" data-doc="An icon of the atlas, drawn large on a gradient.">
<meta name="prop:name" content="string" data-doc="What a screen reader calls it (Fireball).">
<meta name="prop:action" content="string" data-doc="The action whose glyph the hotkey cap shows on the device used last (it follows a rebinding); empty is none.">
<meta name="prop:hotkey" content="string" data-doc="A label for the cap instead of the action's glyph (Q, LB); empty shows the action's.">
<meta name="prop:cooldown" content="number" data-doc="Seconds of cooldown left; 0 is ready.">
<meta name="prop:cooldownMax" content="number" data-doc="The cooldown's length; the sweep shows cooldown over it.">
<meta name="prop:charges" content="number" data-doc="Charges ready, in a corner badge when maxCharges is over 1.">
<meta name="prop:maxCharges" content="number" data-doc="Charges it holds; 0 or 1 is none.">
<meta name="prop:cost" content="string" data-doc="Its cost (40) at the bottom left while ready; empty is none.">
<meta name="prop:unaffordable" content="bool" data-doc="Not enough resource: the art tinted red and the cost in danger.">
<meta name="prop:outOfRange" content="bool" data-doc="The target is out of range: the art tinted red.">
<meta name="prop:active" content="bool" data-doc="Toggled on or channelling: a breathing glow and a primary edge.">
<meta name="prop:locked" content="bool" data-doc="Not learnt yet: dark art under a lock.">
<meta name="prop:disabled" content="bool" data-doc="Cannot be used now: the art washed out under a red slash.">
<meta name="prop:ultimate" content="number" data-doc="An ultimate's charge 0 to 1: larger, a gold ring filling round it and a percent; negative is a plain slot.">
<meta name="prop:level" content="number" data-doc="Ranks learnt (pips right under it).">
<meta name="prop:maxLevel" content="number" data-doc="Ranks it has; 0 draws no pips.">
<meta name="action:use" content="Use">
<meta name="fixture:default" content='{"icon":"fire","name":"Fireball","action":"","hotkey":"Q","cooldown":4.2,"cooldownMax":8,"charges":2,"maxCharges":3,"cost":"40","unaffordable":false,"outOfRange":false,"active":false,"locked":false,"disabled":false,"ultimate":-1,"level":2,"maxLevel":3}'>

<div id="AbilitySlot" class:ultimate={ultimate >= 0}>
  <button id="Slot" onclick={use} aria-label={name} aria-valuetext={locked ? t("component.ability-slot.locked") : disabled ? t("component.ability-slot.unavailable") : cooldown > 0 ? formatNumber(cooldown) : t("component.ability-slot.ready")} disabled={locked || disabled} class:cooling={cooldown > 0} class:active={active} class:tinted={unaffordable || outOfRange} class:locked={locked} class:off={disabled} class:full={ultimate >= 1}>
    <div id="Face">
      <div id="Art"><icon id="Icon" icon={icon}></icon></div>
      <svg id="Ultimate" viewBox="0 0 64 64" hidden={ultimate < 0}><circle cx="32" cy="32" r="29" fill="none" stroke="#00000080" stroke-width="4"></circle><circle id="UltimateFill" cx="32" cy="32" r="29" fill="none" stroke-width="4" pathLength="1" stroke-dasharray={ultimate + " 1"} transform="rotate(-90 32 32)"></circle></svg>
      <svg id="Sweep" viewBox="0 0 56 56" hidden={!(cooldown > 0)}><circle cx="28" cy="28" r="20" fill="none" stroke="#00000099" stroke-width="40" pathLength="1" stroke-dasharray={(cooldown / cooldownMax) + " 1"} transform="rotate(-90 28 28) scale(-1 1) translate(-56 0)"></circle></svg>
      <span id="Time" hidden={!(cooldown > 0)}>{cooldown < 10 ? formatNumber(cooldown) : formatCountdown(cooldown)}</span>
      <span id="Percent" hidden={!(ultimate >= 0 && ultimate < 1)}>{formatPercent(ultimate)}</span>
      <svg id="Slash" viewBox="0 0 56 56" hidden={!disabled}><path d="M8 48L48 8" stroke="#e5484dff" stroke-width="5" stroke-linecap="round"></path></svg>
      <icon id="Lock" icon="lock" hidden={!locked}></icon>
      <span id="Cost" class:short={unaffordable} hidden={!cost || cooldown > 0 || locked}>{cost}</span>
      <span id="Charges" hidden={!(maxCharges > 1) || cooldown > 0}>{formatNumber(charges)}</span>
    </div>
  </button>
  <div id="Pips" hidden={!(maxLevel > 0)} style:--ranks={maxLevel} style:--level={level}><div id="Ranked"></div></div>
  <kbd id="Hotkey" data-action={action} hidden={!hotkey && !action}>{hotkey}</kbd>
</div>

<style>
#Face { width: stretch; height: stretch; align-items: center; justify-content: center; }
#AbilitySlot { align-items: center; gap: 3px; }
#Slot { width: 56px; height: 56px; padding: 0; border-radius: 8px; overflow: hidden; border: 1px solid #ffffff4d; background: var(--control-strong); box-shadow: inset 0 2px 4px #00000080, 0 3px 6px #00000080; transition: scale 160ms ease-out; }
.ultimate #Slot { width: 64px; height: 64px; border-radius: 32px; }
#Art { width: stretch; height: stretch; align-items: center; justify-content: center; background: radial-gradient(circle at 50% 30%, #ff9a3cff, #7a2a0eff); }
#Icon { width: 34px; height: 34px; color: #ffffffff; }
#Slot.tinted #Art { background: radial-gradient(circle at 50% 30%, #c0392bff, #4a0e0eff); }
#Slot.tinted #Icon { color: #ff8080ff; }
#Slot.locked #Art { filter: grayscale(1) brightness(0.35); }
#Slot.off #Art { filter: grayscale(1) brightness(0.7); }
#Slot.active { box-shadow: 0 0 12px var(--primary), inset 0 0 0 2px var(--primary); animation: breathe 1400ms ease-in-out infinite alternate; }
#Slot.full { box-shadow: 0 0 14px var(--gold), inset 0 0 0 2px var(--gold); animation: breathe 1400ms ease-in-out infinite alternate; }
#Ultimate { position: absolute; top: 0; inset-inline-start: 0; width: 64px; height: 64px; }
#UltimateFill { stroke: var(--gold); }
#Sweep { position: absolute; top: 0; inset-inline-start: 0; width: stretch; height: stretch; }
#Time { position: absolute; top: 16px; inset-inline-start: 0; width: stretch; text-align: center; font: var(--text-lg); color: #ffffffff; text-shadow: 0 0 3px #000000ff; }
#Percent { position: absolute; bottom: 2px; inset-inline-start: 0; width: stretch; text-align: center; font: var(--text-caption); color: var(--gold); text-shadow: 0 1px 2px #000000ff; }
#Slash { position: absolute; top: 0; inset-inline-start: 0; width: stretch; height: stretch; }
#Lock { position: absolute; top: 16px; inset-inline-start: 16px; width: 24px; height: 24px; color: var(--ink); }
#Cost { position: absolute; bottom: 2px; inset-inline-start: 2px; padding: 0 4px; border-radius: 6px; background: #000000b3; font: var(--text-caption); color: var(--mana); }
#Cost.short { color: var(--danger); }
#Charges { position: absolute; top: 2px; inset-inline-start: 38px; min-width: 16px; padding: 0 3px; border-radius: 8px; background: #000000cc; font: var(--text-caption); color: var(--ink); align-items: center; }
#Pips { width: calc(var(--ranks) * 9px - 3px); height: 6px; background: repeating-linear-gradient(90deg, #00000099 0px, #00000099 6px, #00000000 6px, #00000000 9px); }
#Ranked { width: calc(var(--level) * 9px - 3px); height: 6px; background: repeating-linear-gradient(90deg, #ffc93cff 0px, #ffc93cff 6px, #00000000 6px, #00000000 9px); }
#Hotkey { font: var(--text-caption); color: var(--ink-muted); }
@keyframes breathe { from { opacity: 1; } to { opacity: 0.8; } }
</style>
