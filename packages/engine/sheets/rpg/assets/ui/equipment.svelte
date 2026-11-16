<meta name="component" content="EquipmentPage">

<!-- Equipment, as on the board: the menu bar with its screens and close;
     the figure with an equipment column either side; the stats in two
     columns; a footer of Back, Unequip, Auto-equip and Details. -->
<div id="Page" class="rpg-ground">
  <div id="Menu"><RpgMenuBar tabs={$menuTabs} tab={$menuTab} /></div>
  <div id="Body">
    <div id="Hero"><RpgCharacter title={$equipTitle} name={$charName} line={$charLine} portrait={0} figure={$charFigure} slots={$equipSlots} selected={$charSelected} attributes={$noAttributes} /></div>
    <div id="Stats"><RpgStats title={$statsTitle} groups={$statGroups} rows={$statRows} /></div>
  </div>
  <div id="Foot">
    <button id="Back" class="primary foot" onclick={back}>{$footBack}</button>
    <div id="FootGap"></div>
    <button id="Unequip" class="secondary foot" onclick={unequip}>{$footUnequip}</button>
    <button id="Auto" class="primary foot" onclick={autoequip}>{$footAuto}</button>
    <button id="Details" class="secondary foot" onclick={details}>{$footDetails}</button>
  </div>
</div>

<style>
#Page { display: flex; flex-direction: column; width: stretch; height: stretch; gap: var(--space-md); padding: 0 var(--space-xl) var(--space-lg); }
#Menu { width: stretch; flex: none; }
#Body { display: flex; width: stretch; flex: 1; min-height: 0; gap: var(--space-lg); }
#Hero { flex: 52; min-width: 0; height: stretch; min-height: 0; }
#Stats { flex: 48; min-width: 0; height: stretch; min-height: 0; }
#Foot { display: flex; width: stretch; flex: none; gap: var(--space-md); align-items: center; }
#Foot button { width: calc(var(--size-xxl) * 2.6); }
#FootGap { flex: 1; }
@media (width < 600px) {
  #Page { padding: 0 var(--space-sm) var(--space-sm); gap: var(--space-sm); }
  #Body { flex-direction: column; gap: var(--space-sm); }
  #Hero { flex: 60; width: stretch; height: auto; }
  #Stats { flex: 40; width: stretch; height: auto; }
  #Foot { flex-wrap: wrap; gap: var(--space-xs); }
  #Foot button { width: calc(50% - var(--space-xs) / 2); }
  /* Two to a row on a phone: a step smaller and tighter, so AUTO-EQUIP fits whole at 150 % text. */
  button.foot::part(base) { padding: 0 var(--space-sm); font-size: 0.9286em; letter-spacing: 0.04em; }
  #FootGap { display: none; }
}
@media (height < 500px) {
  #Page { display: grid; grid-template-columns: minmax(0, 1fr) auto; grid-template-rows: auto minmax(0, 1fr); grid-template-areas: "menu menu" "body foot"; column-gap: var(--space-sm); row-gap: var(--space-xs); padding: 0 var(--space-md) var(--space-sm); }
  #Menu { grid-area: menu; }
  #Body { grid-area: body; height: stretch; gap: var(--space-sm); }
  #Foot { grid-area: foot; flex-direction: column; justify-content: flex-end; gap: var(--space-xs); }
  #Foot button { width: calc(var(--size-xxl) * 2); }
  #FootGap { display: none; }
}
</style>
