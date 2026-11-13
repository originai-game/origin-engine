<meta name="component" content="LootLabel">
<meta name="parts" content="LootLabel.tag LootLabel.name LootLabel.count">
<meta name="prop:entity" content="number" data-doc="The item, by its bits (0: use at).">
<meta name="prop:at" content="string" data-doc="A world point it sits over without an entity (x y z).">
<meta name="prop:height" content="number" data-doc="World units above it.">
<meta name="prop:name" content="string" data-doc="The item's name.">
<meta name="prop:count" content="number" data-doc="A stack's size, shown as ×n from 2.">
<meta name="prop:rarity" content="string" data-doc="Its tier: common (empty), uncommon, rare, epic or legendary.">
<meta name="action:pick" content="Pick up">
<meta name="fixture:default" content='{"entity":0,"at":"0 0 0","height":0.4,"name":"Ember Blade","count":1,"rarity":"epic"}'>

<anchor id="LootLabel" entity={entity || null} at={at} offset={"0 " + height + " 0"} declutter="stack" occlusion="fade" priority="10" max-distance="30">
  <button id="Tag" part="LootLabel.tag" onclick={pick} class:uncommon={rarity == 'uncommon'} class:rare={rarity == 'rare'} class:epic={rarity == 'epic'} class:legendary={rarity == 'legendary'}>
    <span id="Name" part="LootLabel.name">{name}</span>
    <span id="Count" part="LootLabel.count" hidden={!(count > 1)}>{"×" + formatNumber(count)}</span>
  </button>
</anchor>

<style>
#LootLabel { align-items: center; }
[part="LootLabel.tag"] { gap: var(--space-xs); padding: var(--space-xxs) var(--space-sm); min-height: 0; border-radius: var(--radius-control); background: var(--scene-plate); border: var(--stroke-hairline) solid var(--tier); --tier: var(--common, var(--scene-ink)); }
[part="LootLabel.tag"].uncommon { --tier: var(--uncommon, var(--success)); }
[part="LootLabel.tag"].rare { --tier: var(--rare, var(--info)); }
[part="LootLabel.tag"].epic { --tier: var(--epic, var(--xp)); }
[part="LootLabel.tag"].legendary { --tier: var(--legendary, var(--warn)); }
[part="LootLabel.tag"]:hover { background: var(--panel-hover); }
[part="LootLabel.name"] { font: var(--text-sm); color: var(--tier); white-space: nowrap; }
[part="LootLabel.count"] { font: var(--text-caption); color: var(--ink-muted); }
</style>
