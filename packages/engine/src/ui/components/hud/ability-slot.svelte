<meta name="component" content="AbilitySlot">
<meta name="strings" content="./ability-slot.en.ogt">
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
<meta name="prop:fit" content="bool" data-doc="It fills the place it is given (a strip's), rather than standing --ability-slot-size wide.">
<meta name="prop:maxLevel" content="number" data-doc="Ranks it has; 0 draws no pips.">
<meta name="action:use" content="Use">
<meta name="fixture:default" content='{"icon":"fire","name":"Fireball","action":"","hotkey":"Q","cooldown":4.2,"cooldownMax":8,"charges":2,"maxCharges":3,"count":0,"cost":"40","unaffordable":false,"outOfRange":false,"active":false,"locked":false,"disabled":false,"ultimate":-1,"level":2,"maxLevel":3}'>

<div id="AbilitySlot" part="frame" variant="bare" layout="stack" gap="hair" class:ultimate={ultimate >= 0} class:fit={fit}>
  <button id="Slot" part="cell" variant="hud" shape={ultimate >= 0 ? 'round' : 'square'} state:active={active || ultimate >= 1} state:locked={locked} play={active || ultimate >= 1 ? 'pulse' : null} onclick={use} aria-label={name} aria-valuetext={locked ? t("component.ability-slot.locked") : disabled ? t("component.ability-slot.unavailable") : cooldown > 0 ? formatNumber(cooldown) : t("component.ability-slot.ready")} disabled={locked || disabled} class:cooling={cooldown > 0} class:tinted={unaffordable || outOfRange} class:full={ultimate >= 1}>
    <div id="Face" layout="overlay">
      <div id="Art" part="well" variant="hud" layout="stack" anchor="fill"><og-icon id="Icon" icon={icon}></og-icon></div>
      {#if ultimate >= 0}<svg id="Ultimate" anchor="fill" viewBox="0 0 64 64"><circle id="UltimateTrack" cx="32" cy="32" r="29" stroke-width="4"></circle><circle id="UltimateFill" cx="32" cy="32" r="29" stroke-width="4" pathLength="1" stroke-dasharray={ultimate + " 1"} transform="rotate(-90 32 32)"></circle></svg>{/if}
      {#if cooldown > 0}
        <svg id="Sweep" anchor="fill" viewBox="0 0 56 56"><circle id="SweepArc" cx="28" cy="28" r="20" stroke-width="40" pathLength="1" stroke-dasharray={(cooldown / cooldownMax) + " 1"} transform="rotate(-90 28 28) scale(-1 1) translate(-56 0)"></circle></svg>
        <div class="middle" anchor="fill" layout="center"><span id="Time" part="figure" variant="hud">{cooldown < 10 ? formatNumber(cooldown) : formatCountdown(cooldown)}</span></div>
      {/if}
      {#if ultimate >= 0 && ultimate < 1}<div id="Charge" layout="row" anchor="bottom-edge"><span id="Percent" part="badge">{formatPercent(ultimate)}</span></div>{/if}
      {#if disabled}<svg id="Slash" anchor="fill" viewBox="0 0 56 56"><path id="SlashLine" d="M8 48L48 8" stroke-width="5" stroke-linecap="round"></path></svg>{/if}
      {#if locked}<div class="middle" anchor="fill" layout="center"><og-icon id="Lock" icon="lock"></og-icon></div>{/if}
      {#if cost && cooldown <= 0 && !locked}
        {#if ultimate >= 0}<div id="CostTop" layout="row" anchor="top-edge"><span class="cost" part="badge" variant="hud" class:short={unaffordable}>{cost}</span></div>
        {:else}<div layout="row" anchor="bottom-start"><span class="cost" part="badge" variant="hud" class:short={unaffordable}>{cost}</span></div>{/if}
      {/if}
      {#if count > 0}<span id="Count" part="figure" variant="hud" anchor="bottom-end">{formatNumber(count)}</span>{/if}
      {#if maxCharges > 1 && cooldown <= 0 && !locked}<span id="Charges" part="badge" anchor="top-end">{formatNumber(charges)}</span>{/if}
    </div>
  </button>
  {#if maxLevel > 0}<div id="Pips" part="track" style:--ranks={maxLevel} style:--level={level}><div id="Ranked" part="fill"></div></div>{/if}
  {#if hotkey || action}<kbd id="Hotkey" part="badge" variant="hud" data-action={action}>{hotkey}</kbd>{/if}
</div>

<style>
/* One block of a bar a game lays out. Its side is --ability-slot-size and
   it never outgrows its place: a strip that gives it less shrinks it. The
   slot grows with the player's text scale, as the text in it does, so its
   icon, badges and ring keep their proportions. Alone, a long key cap
   widens the block; in a place of a set width the cap keeps to it. */
#AbilitySlot { inline-size: var(--ability-slot-size); max-inline-size: 100%; align-items: center; text-shadow: var(--tag-halo); }
.ultimate { inline-size: calc(var(--ability-slot-size) * 4 / 3); }
#AbilitySlot.fit { inline-size: 100%; }
/* Over the scene the slot is the style's scene cell (the cell's hud
   variant): a socket in a plate, square, or round for an ultimate. */
#Slot { inline-size: 100%; aspect-ratio: 1; padding: 0; overflow: hidden; }
#Face { inline-size: 100%; block-size: 100%; --layout-anchor-inset: var(--space-xxs); }
#Art { inset: var(--cell-scene-inset); justify-content: center; align-items: center; }
.ultimate #Art { border-radius: var(--radius-capsule); }
#Icon { width: 66%; height: 66%; }
.ultimate #Icon { width: 50%; height: 50%; }
/* While it cools, the sweep alone darkens the share still to run: the art
   is not dimmed under it as well, and the seconds read over both on their
   halo. An unaffordable or out-of-range slot tints its art. */
.tinted #Art { background: var(--well-alert-fill); }
.full { border-color: var(--gold); }
#Ultimate, #Sweep, #Slash { inline-size: 100%; block-size: 100%; --layout-anchor-inset: 0; }
#UltimateTrack { fill: none; stroke: var(--track-stroke); }
#UltimateFill { fill: none; stroke: var(--gold); }
#SweepArc { fill: none; stroke: var(--sweep-ink); }
#SlashLine { fill: none; stroke: var(--tone-danger); }
.middle { --layout-anchor-inset: 0; }
#Time { font-size: var(--font-step-xl); }
/* The percent sits on a small ground of its own, so the ring's arc never
   runs through its digits, whatever size and shape a pack gives the ring. */
#Charge { justify-content: center; }
#Percent { background: var(--scrim); color: var(--gold); }
#Lock { width: 50%; height: 50%; }
/* The cost: at the bottom's leading corner on a square slot; centred at
   the top of an ultimate, whose percent takes the bottom. It hides on
   cooldown and when locked, with the time and the lock. */
#CostTop { justify-content: center; }
.cost { color: var(--mana); }
.cost.short { color: var(--tone-danger); }
#Charges { min-width: var(--size-xs); }
/* The count stands inside the art's well, a step off its rim. */
#Count { font-size: var(--font-step-sm); --layout-anchor-inset: calc(var(--cell-scene-inset) + var(--space-xs)); }
#Pips { width: calc(var(--ranks) * var(--space-md) - var(--space-xs)); height: var(--space-xs); mask-image: repeating-linear-gradient(90deg, var(--ink) 0 var(--space-sm), transparent 0 var(--space-md)); }
#Ranked { width: calc(var(--level) * var(--space-md) - var(--space-xs)); height: var(--space-xs); background: var(--gold); }
/* The cap sits under the slot, over the scene (the badge's hud variant:
   the scene's plate in the scene's ink, so it reads over a bright sky as
   over a dark floor). It never outgrows its block's place, so the caps of
   a strip stay apart: a long binding (Ctrl+Shift+5) scrolls inside it. */
#Hotkey { box-sizing: border-box; max-inline-size: 100%; white-space: nowrap; overflow: hidden; text-overflow: -og-scroll var(--motion-duration-orbit) var(--motion-duration-turn); }
</style>
