<meta name="component" content="EquipmentPage">

<!-- Equipment: the character sheet wide, the worn item's card beside it. -->
<div id="Page" class="rpg-ground">
  <div id="Hero"><RpgCharacter name={$charName} line={$charLine} portrait={$charPortrait} figure={$charFigure} slots={$charSlots} selected={$charSelected} attributes={$charAttributes} /></div>
  <div id="Card"><RpgItemCard name={$cardName} icon={$cardIcon} art={$cardArt} rarity={$cardRarity} kind={$cardKind} lead={$cardLead} stats={$cardStats} lore={$cardLore} primary={$cardSecondaryEquip} secondary={$cardUnequip} canPrimary={true} canSecondary={true} /></div>
</div>

<style>
#Page { display: flex; width: stretch; height: stretch; gap: var(--space-lg); padding: var(--space-xl); }
#Hero { width: 58%; height: stretch; min-height: 0; }
#Card { width: 42%; height: stretch; min-height: 0; }
@media (width < 600px) {
  #Page { flex-direction: column; padding: var(--space-sm); gap: var(--space-sm); }
  #Hero { width: stretch; height: 62%; }
  #Card { width: stretch; height: 38%; }
}
</style>
