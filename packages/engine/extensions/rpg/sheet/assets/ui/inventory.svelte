<meta name="component" content="InventoryPage">

<!-- Desktop: the backpack, the item card and the character side by side,
     the full height of the screen. A phone: the backpack over the card. -->
<div id="Page" class="rpg-ground">
  <div id="Bag"><RpgBackpack title={$bagTitle} tabs={$bagTabs} tab={$bagTab} items={$bagItems} selected={$bagSelected} weight={$bagWeight} load={$bagLoad} gold={$bagGold} /></div>
  <div id="Card"><RpgItemCard name={$cardName} icon={$cardIcon} art={$cardArt} rarity={$cardRarity} kind={$cardKind} lead={$cardLead} stats={$cardStats} lore={$cardLore} primary={$cardPrimary} secondary={$cardSecondary} canPrimary={true} canSecondary={true} /></div>
  <div id="Hero"><RpgCharacter name={$charName} line={$charLine} portrait={$charPortrait} figure={$charFigure} slots={$charSlots} selected={$charSelected} attributes={$charAttributes} /></div>
</div>

<style>
#Page { display: flex; width: stretch; height: stretch; gap: var(--space-lg); padding: var(--space-xl); }
#Bag { width: 40%; height: stretch; min-height: 0; }
#Card { width: 28%; height: stretch; min-height: 0; }
#Hero { width: 32%; height: stretch; min-height: 0; }
@media (width < 600px) {
  #Page { flex-direction: column; gap: var(--space-sm); margin: calc(env(safe-area-inset-top) * -1) 0 calc(env(safe-area-inset-bottom) * -1); padding: calc(env(safe-area-inset-top) + var(--space-sm)) var(--space-sm) calc(env(safe-area-inset-bottom) + var(--space-sm)); height: calc(100% + env(safe-area-inset-top) + env(safe-area-inset-bottom)); }
  #Bag { width: stretch; height: 54%; }
  #Card { width: stretch; height: 46%; }
  #Hero { display: none; }
}
@media (height < 600px) and (orientation: landscape) {
  #Page { gap: var(--space-sm); padding: var(--space-sm) var(--space-md); }
  #Bag { width: 58%; }
  #Card { width: 42%; }
  #Hero { display: none; }
}
</style>
