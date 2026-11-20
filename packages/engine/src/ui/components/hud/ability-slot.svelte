<meta name="component" content="AbilitySlot">
<meta name="strings" content="./ability-slot.en.ogt">
<meta name="parts" content="AbilitySlot.frame AbilitySlot.cell AbilitySlot.well AbilitySlot.icon AbilitySlot.ultimate AbilitySlot.figure AbilitySlot.percent AbilitySlot.badge AbilitySlot.track AbilitySlot.fill AbilitySlot.label AbilitySlot.sweep">
<meta name="prop:icon" content="string" data-doc="An icon of the atlas, drawn large on a gradient.">
<meta name="prop:name" content="string" data-doc="What a screen reader calls it (Fireball).">
<meta name="prop:action" content="string" data-doc="The action whose glyph the hotkey cap shows on the device used last (it follows a rebinding); empty is none.">
<meta name="prop:hotkey" content="string" data-doc="A label for the cap instead of the action's glyph (Q, LB); empty shows the action's.">
<meta name="prop:cooldown" content="number" data-doc="Seconds of cooldown left; 0 is ready.">
<meta name="prop:cooldownMax" content="number" data-default="1" data-doc="The cooldown's length; the sweep shows cooldown over it.">
<meta name="prop:charges" content="number" data-doc="Charges ready, in a corner badge when maxCharges is over 1.">
<meta name="prop:maxCharges" content="number" data-doc="Charges it holds; 0 or 1 is none.">
<meta name="prop:count" content="number" data-doc="How many are left (a quick item: potions, scrolls), at the bottom's end; 0 shows none.">
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
<meta name="fixture:default" content='{"icon":"fire","name":"Fireball","action":"","hotkey":"Q","cooldown":4.2,"cooldownMax":8,"charges":2,"maxCharges":3,"count":0,"cost":"40","unaffordable":false,"outOfRange":false,"active":false,"locked":false,"disabled":false,"ultimate":-1,"level":2,"maxLevel":3}'>

<div id="AbilitySlot" part="AbilitySlot.frame" data-layout="stack" data-gap="hair" class="bare" class:ultimate={ultimate >= 0}>
  <button id="Slot" part="AbilitySlot.cell" data-shape={ultimate >= 0 ? 'round' : 'square'} data-active={active || ultimate >= 1} data-locked={locked} data-motion={active || ultimate >= 1 ? 'pulse' : null} onclick={use} aria-label={name} aria-valuetext={locked ? t("component.ability-slot.locked") : disabled ? t("component.ability-slot.unavailable") : cooldown > 0 ? formatNumber(cooldown) : t("component.ability-slot.ready")} disabled={locked || disabled} class:cooling={cooldown > 0} class:active={active} class:tinted={unaffordable || outOfRange} class:locked={locked} class:off={disabled} class:full={ultimate >= 1}>
    <div id="Face" data-layout="overlay">
      <div id="Art" part="AbilitySlot.well" data-layout="cover" data-gap="none" data-anchor="fill"><og-icon id="Icon" part="AbilitySlot.icon" icon={icon}></og-icon></div>
      {#if ultimate >= 0}<svg id="Ultimate" part="AbilitySlot.ultimate" data-anchor="fill" viewBox="0 0 64 64"><circle part="AbilitySlot.track" cx="32" cy="32" r="29" fill="none" stroke-width="4"></circle><circle id="UltimateFill" part="AbilitySlot.fill" cx="32" cy="32" r="29" fill="none" stroke-width="4" pathLength="1" stroke-dasharray={ultimate + " 1"} transform="rotate(-90 32 32)"></circle></svg>{/if}
      {#if cooldown > 0}
        <svg id="Sweep" data-anchor="fill" viewBox="0 0 56 56"><circle part="AbilitySlot.sweep" cx="28" cy="28" r="20" fill="none" stroke-width="40" pathLength="1" stroke-dasharray={(cooldown / cooldownMax) + " 1"} transform="rotate(-90 28 28) scale(-1 1) translate(-56 0)"></circle></svg>
        <span id="Time" class="time" part="AbilitySlot.figure" data-anchor="center">{cooldown < 10 ? formatNumber(cooldown) : formatCountdown(cooldown)}</span>
      {/if}
      <div id="Charge" data-layout="row" data-anchor="bottom">{#if ultimate >= 0 && ultimate < 1}<span id="Percent" class="percent" part="AbilitySlot.percent AbilitySlot.badge">{formatPercent(ultimate)}</span>{/if}</div>
      {#if disabled}<svg id="Slash" data-anchor="fill" viewBox="0 0 56 56"><path part="AbilitySlot.fill" data-tone="danger" d="M8 48L48 8" stroke-width="5" stroke-linecap="round"></path></svg>{/if}
      {#if locked}<og-icon id="Lock" data-anchor="center" icon="lock"></og-icon>{/if}
      <div id="Costs" data-layout="row" data-anchor={ultimate >= 0 ? 'top' : 'bottom-start'}>{#if cost && cooldown <= 0 && !locked}<span id="Cost" part="AbilitySlot.badge" class="cost" class:short={unaffordable}>{cost}</span>{/if}</div>
      {#if count > 0}<span id="Count" part="AbilitySlot.figure" class="count" data-anchor="bottom-end">{formatNumber(count)}</span>{/if}
      {#if maxCharges > 1 && cooldown <= 0 && !locked}<span id="Charges" part="AbilitySlot.badge" class="charges" data-anchor="top-end">{formatNumber(charges)}</span>{/if}
    </div>
  </button>
  {#if maxLevel > 0}<div id="Pips" part="AbilitySlot.well AbilitySlot.track" style:--ranks={maxLevel} style:--level={level}><div id="Ranked" part="AbilitySlot.fill"></div></div>{/if}
  {#if hotkey || action}<kbd id="Hotkey" part="AbilitySlot.label AbilitySlot.badge" data-action={action}>{hotkey}</kbd>{/if}
</div>

<style>
:host { text-shadow: var(--tag-halo); --scene-halo: var(--tag-halo); }
#Face { inline-size: 100%; block-size: 100%; --layout-anchor-inset: var(--space-xxs); }
/* The slot grows with the player's text scale, as the text in it does,
   so its icon, badges and ring keep their proportions:
   --text-px is one CSS px at 100 % text (rem is the body type step, 14 px
   at 100 %, grown by the scale). It is one block of a bar a game lays
   out: --ability-slot-size is its side (100 % fills the place a strip
   gives it), and it never outgrows its place. Alone, a long key cap
   widens the block; in a place of a set width the cap keeps to it. */
:host { --text-px: var(--font-step-px); --slot: var(--ability-slot-size); max-inline-size: 100%; align-items: center; }
/* Over the scene the slot is the style's scene cell: a socket in a plate. */
#Slot { container-type: inline-size; inline-size: var(--slot); max-inline-size: 100%; block-size: auto; aspect-ratio: 1; padding: 0; overflow: hidden; --cell-fill: var(--cell-scene-fill); --cell-rim: var(--cell-scene-rim); --cell-shadow: var(--cell-scene-shadow); --cell-empty-shadow: var(--cell-scene-shadow); }
.ultimate #Slot { inline-size: calc(var(--slot) * 4 / 3); }
#Ultimate, #Sweep, #Slash { --layout-anchor-inset: 0; }
#Art { inset: var(--cell-scene-inset); --well-radius: var(--cell-scene-well-radius); }
.ultimate #Art { --well-radius: var(--radius-capsule); }
#Icon { width: 66%; height: 66%; }
.ultimate #Icon { width: 50%; height: 50%; }
/* While it cools the art steps back, so the seconds left read over it. */
#Slot.cooling #Icon { opacity: var(--opacity-dim); }
#Slot.tinted #Art { --well-fill: var(--well-alert-fill); }
#Slot.full { --cell-active-rim: var(--gold); }
#Ultimate, #Sweep, #Slash { inline-size: 100%; block-size: 100%; }
#UltimateFill { --fill-color: var(--gold); }
#Time { --figure-size: min(var(--font-step-xl), 38cqi); --figure-ink: var(--scene-ink); --figure-shadow: var(--tag-halo); }
/* The percent sits on a small ground of its own (a pack sets it with
   --ability-slot-percent-ground), so the ring's arc never runs through its
   digits, whatever size and shape a pack gives the ring. */
#Charge { justify-content: center; }
#Percent { --badge-fill: var(--ability-slot-percent-ground, var(--scrim)); --badge-ink: var(--gold); }
#Lock { width: 50%; height: 50%; }
/* The cost sits in a row of its own: at the bottom's leading corner on a
   square slot; on an ultimate, whose percent takes the bottom, centred at
   the top, where the round slot is wide enough for it and where nothing
   else shows while it does (it hides on cooldown and when locked, with the
   time and the lock). */
#Cost { --badge-ink: var(--mana); --badge-fill: var(--scene-plate); }
#Cost.short { --badge-ink: var(--tone-danger); }
#Charges { min-width: var(--size-xs); }
#Count { --figure-size: var(--font-step-sm); --figure-ink: var(--scene-ink); --figure-shadow: var(--tag-halo); }
#Pips { width: calc(var(--ranks) * var(--space-md) - var(--space-xs)); height: var(--space-xs); mask-image: repeating-linear-gradient(90deg, #000 0 var(--space-sm), transparent 0 var(--space-md)); }
#Ranked { width: calc(var(--level) * var(--space-md) - var(--space-xs)); height: var(--space-xs); --fill-color: var(--gold); }
/* The cap sits under the slot, over the scene: it is the scene's plate in
   the scene's ink, the pair a pack sets for text over the scene, so it reads
   over a bright sky as over a dark floor. */
#Hotkey { --badge-fill: var(--scene-plate); --badge-ink: var(--scene-ink); }
/* A cap never outgrows its block's place, so the caps of a strip stay
   apart: there a long binding (Ctrl+Shift+5) scrolls inside it. */
#Hotkey { box-sizing: border-box; max-inline-size: 100%; white-space: nowrap; overflow: hidden; text-overflow: -og-scroll var(--motion-duration-orbit) var(--motion-duration-turn); }
</style>
