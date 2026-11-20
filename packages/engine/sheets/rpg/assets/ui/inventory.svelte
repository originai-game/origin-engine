<meta name="component" content="InventoryPage">

<!-- Desktop: the backpack, the item card and the character side by side,
     the full height of the screen. A phone: the backpack over the card. -->
<div id="Page" class="rpg-ground">
  <div id="Bag"><RpgBackpack title={$bagTitle} kicker={$bagKicker} capacity={$bagCapacity} tabs={$bagTabs} tab={$bagTab} items={$bagItems} selected={$bagSelected} weight={$bagWeight} load={$bagLoad} gold={$bagGold} /></div>
  <div id="Card"><RpgItemCard name={$cardName} icon={$cardIcon} art={$cardArt} rarity={$cardRarity} kind={$cardKind} lead={$cardLead} stats={$cardStats} lore={$cardLore} primary={$cardPrimary} secondary={$cardSecondary} line={$cardLine} canPrimary={$cardCanPrimary} canSecondary={true} reason={$cardReason} /></div>
  <div id="Hero"><RpgCharacter title="" name={$charName} line={$charLine} portrait={$charPortrait} figure={$charFigure} slots={$charSlots} selected={$charSelected} attributes={$charAttributes} /></div>
</div>

<style>
#Page { display: flex; width: stretch; height: stretch; gap: var(--space-lg); padding: var(--space-xl); }
#Bag { flex: 38; min-width: 0; height: stretch; min-height: 0; }
#Card { flex: 30; min-width: 0; height: stretch; min-height: 0; }
#Hero { flex: 32; min-width: 0; height: stretch; min-height: 0; }
#Page:size(compact) { flex-direction: column; gap: var(--space-sm); margin: calc(env(safe-area-inset-top) * -1) 0 calc(env(safe-area-inset-bottom) * -1); padding: calc(env(safe-area-inset-top) + var(--space-sm)) var(--space-sm) calc(env(safe-area-inset-bottom) + var(--space-sm)); height: calc(100% + env(safe-area-inset-top) + env(safe-area-inset-bottom)); }
#Bag:size(compact) { flex: 58; width: stretch; height: auto; min-height: 0; }
#Card:size(compact) { flex: 42; width: stretch; height: auto; }
#Hero:size(compact) { display: none; }
#Page:size(short) { gap: var(--space-sm); padding: var(--space-sm) var(--space-md); }
#Bag:size(short) { flex: 58; }
#Card:size(short) { flex: 42; }
#Hero:size(short) { display: none; }
</style>
