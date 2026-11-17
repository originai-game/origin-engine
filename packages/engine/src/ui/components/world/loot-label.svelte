<meta name="component" content="LootLabel">
<meta name="parts" content="LootLabel.tag LootLabel.name LootLabel.count LootLabel.figure LootLabel.cell">
<meta name="prop:entity" content="number" data-doc="The item, by its bits (0: use at).">
<meta name="prop:at" content="string" data-default="0 0 0" data-doc="A world point it sits over without an entity (x y z).">
<meta name="prop:height" content="number" data-doc="World units above it.">
<meta name="prop:name" content="string" data-doc="The item's name.">
<meta name="prop:count" content="number" data-doc="A stack's size, shown as ×n from 2.">
<meta name="prop:rarity" content="string" data-doc="Its tier: common (empty), uncommon, rare, epic or legendary.">
<meta name="action:pick" content="Pick up">
<meta name="fixture:default" content='{"entity":0,"at":"0 0 0","height":0.4,"name":"Ember Blade","count":1,"rarity":"epic"}'>

<anchor id="LootLabel" entity={entity || null} at={at} offset={"0 " + height + " 0"} declutter="stack" occlusion="fade" priority="10" max-distance="30">
  <button id="Tag" part="LootLabel.tag LootLabel.cell" onclick={pick} data-rarity={rarity || 'common'} class:uncommon={rarity == 'uncommon'} class:rare={rarity == 'rare'} class:epic={rarity == 'epic'} class:legendary={rarity == 'legendary'}>
    <div class="face" part="LootLabel.tag" data-layout="row" data-gap="tight">
    <span id="Name" part="LootLabel.name">{name}</span>
    {#if count > 1}<span id="Count" part="LootLabel.count LootLabel.figure">{"×" + formatNumber(count)}</span>{/if}
    </div>
  </button>
</anchor>

<style>
:host { text-shadow: var(--tag-halo); }
[part~="LootLabel.cell"] { padding: var(--space-xxs) var(--space-sm); min-height: 0; --tag-fill: var(--scene-plate); --cell-rim: var(--common); }
[part~="LootLabel.tag"].uncommon { --name-ink: var(--uncommon); --cell-rim: var(--uncommon); }
[part~="LootLabel.tag"].rare { --name-ink: var(--rare); --cell-rim: var(--rare); }
[part~="LootLabel.tag"].epic { --name-ink: var(--epic); --cell-rim: var(--epic); }
[part~="LootLabel.tag"].legendary { --name-ink: var(--legendary); --cell-rim: var(--legendary); }
[part="LootLabel.name"], [part~="LootLabel.count"] { white-space: nowrap; }
</style>
