<meta name="component" content="AbilityBar">
<meta name="strings" content="./ability-bar.en.ogt">
<meta name="parts" content="AbilityBar.frame">
<meta name="prop:slots" content="list" data-doc="The slots, in order: each { id, icon, name, action, hotkey, cooldown, cooldownMax, cost, unaffordable, locked, active, count }, cooldown in seconds (0: ready); count above 0 shows how many are left (a quick item: potions, scrolls). A slot's full set of states is the AbilitySlot component.">
<meta name="prop:column" content="bool" data-doc="The slots run top to bottom; false is a row.">
<meta name="action:use" content="Use">
<meta name="fixture:default" content='{"column":false,"slots":[{"id":"q","icon":"sword","name":"Cleave","action":"","hotkey":"Q","cooldown":0,"cooldownMax":6,"cost":"","unaffordable":false,"locked":false,"active":false},{"id":"w","icon":"shield","name":"Guard","action":"","hotkey":"W","cooldown":3.4,"cooldownMax":10,"cost":"30","unaffordable":false,"locked":false,"active":false},{"id":"e","icon":"bolt","name":"Dash","action":"","hotkey":"E","cooldown":0,"cooldownMax":5,"cost":"60","unaffordable":true,"locked":false,"active":false},{"id":"r","icon":"fire","name":"Inferno","action":"","hotkey":"R","cooldown":0,"cooldownMax":60,"cost":"","unaffordable":false,"locked":true,"active":false},{"id":"t","icon":"potion","name":"Elixir","action":"","hotkey":"1","cooldown":0,"cooldownMax":0,"cost":"","unaffordable":false,"locked":false,"active":true,"count":9}]}'>

<!-- A strip of AbilitySlot blocks, and nothing else: the slot draws
     itself, the strip sizes them. It is as wide as its slots at
     --ability-bar-slot-size and, where it is given less, every slot
     shrinks alike, down to --ability-bar-slot-min; under that the strip
     breaks into lines, so it never overflows. A game that wants
     another arrangement (two rows, groups, an ultimate at the end) lays
     AbilitySlot out itself in the same way. -->
<ul id="AbilityBar" part="AbilityBar.frame" class="bare" data-layout={column ? 'stack' : 'strip'} data-wraps="true" data-gap="tight" style:--layout-strip-count={slots.length} role="toolbar" aria-label={t("component.ability-bar.name")} class:column={column}>
  {#each slots as s (s.id)}
    <li class="item" data-fit><AbilitySlot --ability-slot-size="100%" icon={s.icon} name={s.name} action={s.action} hotkey={s.hotkey} cooldown={s.cooldown} cooldownMax={s.cooldownMax} charges={0} maxCharges={0} count={s.count} cost={s.cost} unaffordable={s.unaffordable} outOfRange={false} active={s.active} locked={s.locked} disabled={false} ultimate={-1} level={0} maxLevel={0} /></li>
  {/each}
</ul>

<style>
/* The slots grow with the player's text scale (--font-step-px is one CSS
   px at 100 % text). */
:host { margin: 0; padding: 0; list-style-type: none; --layout-strip-item: var(--ability-bar-slot-size); --layout-strip-item-min: var(--ability-bar-slot-min); }
.column { align-items: center; }
.column .item { inline-size: var(--layout-strip-item); }
</style>