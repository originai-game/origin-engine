<meta name="component" content="LootLabel">
<meta name="prop:entity" content="number" data-doc="The item, by its bits (0: use at).">
<meta name="prop:at" content="string" data-doc="A world point it sits over without an entity (x y z).">
<meta name="prop:height" content="number" data-doc="World units above it.">
<meta name="prop:name" content="string" data-doc="The item's name.">
<meta name="prop:count" content="number" data-doc="A stack's size, shown as ×n from 2.">
<meta name="prop:rarity" content="string" data-doc="Its tier: common (empty), uncommon, rare, epic or legendary.">
<meta name="action:pick" content="Pick up">
<meta name="fixture:default" content='{"entity":0,"at":"0 0 0","height":0.4,"name":"Ember Blade","count":1,"rarity":"epic"}'>

<anchor id="LootLabel" entity={entity || null} at={at} offset={"0 " + height + " 0"} declutter="stack" occlusion="fade" priority="10" max-distance="30">
  <button id="Tag" onclick={pick} class:uncommon={rarity == 'uncommon'} class:rare={rarity == 'rare'} class:epic={rarity == 'epic'} class:legendary={rarity == 'legendary'}>
    <span id="Name">{name}</span>
    <span id="Count" hidden={!(count > 1)}>{"×" + formatNumber(count)}</span>
  </button>
</anchor>

<style>
#LootLabel { align-items: center; }
#Tag { gap: 4px; padding: 2px 8px; min-height: 0; border-radius: 3px; background: #000000c7; border: 1px solid var(--tier); --tier: #c8c8c8ff; }
#Tag.uncommon { --tier: #1eff00ff; }
#Tag.rare { --tier: #0070ddff; }
#Tag.epic { --tier: #a335eeff; }
#Tag.legendary { --tier: #ff8000ff; }
#Tag:hover { background: #202634e6; }
#Name { font: var(--text-sm); color: var(--tier); white-space: nowrap; }
#Count { font: var(--text-caption); color: var(--ink-muted); }
</style>
