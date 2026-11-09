<meta name="component" content="rpgSheet">
<meta name="prop:page" content="string" data-doc="The page shown: inventory (the normal page) or journal (the stress page).">
<meta name="prop:inv" content="object" data-doc="RpgInventory's props.">
<meta name="prop:eq" content="object" data-doc="RpgEquipment's props.">
<meta name="prop:jr" content="object" data-doc="RpgJournal's props.">
<meta name="prop:tr" content="object" data-doc="RpgTracker's props.">
<meta name="prop:dl" content="object" data-doc="RpgDialogue's props.">
<meta name="prop:pa" content="object" data-doc="RpgParty's props.">
<meta name="prop:to" content="object" data-doc="RpgTurnOrder's props.">
<meta name="prop:mp" content="object" data-doc="RpgWorldMap's props.">
<meta name="prop:cp" content="object" data-doc="RpgCompass's props.">
<meta name="fixture:default" content='{"page":"inventory","inv":{},"eq":{},"jr":{},"tr":{},"dl":{},"pa":{},"to":{},"mp":{},"cp":{}}'>

<!--
  The rpg kit's review sheet (docs/ui.md §6.2). inventory, the normal
  page: the bag beside the equipment. journal, the stress page: the party
  and turn order, the world map under the compass and tracker with a
  conversation below it, and the journal; on a phone they stack and
  scroll. Each member takes its props from one store (`$inv.items`).
-->
<div id="Page">
  {#if $page == "inventory"}
    <scroll id="InventoryPage">
      <div id="Inventory" class="spread">
        <div id="Bag"><RpgInventory title={$inv.title} filter={$inv.filter} filters={$inv.filters} items={$inv.items} selected={$inv.selected} weight={$inv.weight} load={$inv.load} gold={$inv.gold} detailName={$inv.detailName} detailIcon={$inv.detailIcon} detailRarity={$inv.detailRarity} detailKind={$inv.detailKind} detailText={$inv.detailText} detailStats={$inv.detailStats} detailValue={$inv.detailValue} canUse={$inv.canUse} canEquip={$inv.canEquip} equipLabel={$inv.equipLabel} canDrop={$inv.canDrop} /></div>
        <div id="Doll"><RpgEquipment name={$eq.name} subtitle={$eq.subtitle} portrait={$eq.portrait} selected={$eq.selected} slots={$eq.slots} stats={$eq.stats} /></div>
      </div>
    </scroll>
  {:else}
    <scroll id="JournalPage">
      <div id="Stress" class="spread">
        <div id="Party"><RpgParty selected={$pa.selected} orientation={$ui.width < 700 ? "horizontal" : $pa.orientation} compact={$pa.compact} members={$pa.members} /></div>
        <div id="World">
          <RpgTurnOrder round={$to.round} selected={$to.selected} entries={$to.entries} />
          <div id="MapWrap">
            <RpgWorldMap map={$mp.map} region={$mp.region} selected={$mp.selected} detailName={$mp.detailName} detailText={$mp.detailText} canTravel={$mp.canTravel} canTrack={$mp.canTrack} markers={$mp.markers} filters={$mp.filters} />
          </div>
        </div>
        <div id="Hud"><div id="Tracker"><RpgTracker quest={$tr.quest} distance={$tr.distance} objectives={$tr.objectives} updated={$tr.updated} /></div><div id="Compass"><RpgCompass facing={$cp.facing} points={$cp.points} markers={$cp.markers} /></div></div>
        <div id="Talk"><RpgDialogue speaker={$dl.speaker} epithet={$dl.epithet} portrait={$dl.portrait} line={$dl.line} choices={$dl.choices} selected={$dl.selected} canContinue={$dl.canContinue} canLog={$dl.canLog} /></div>
        <div id="Quests"><RpgJournal title={$jr.title} filter={$jr.filter} filters={$jr.filters} quests={$jr.quests} selected={$jr.selected} detailName={$jr.detailName} detailMeta={$jr.detailMeta} detailText={$jr.detailText} objectives={$jr.objectives} rewards={$jr.rewards} tracked={$jr.tracked} canTrack={$jr.canTrack} canLocate={$jr.canLocate} /></div>
      </div>
    </scroll>
  {/if}
</div>

<style>
#Page { width: stretch; height: stretch; background: var(--backdrop); container-type: inline-size; }
#InventoryPage, #JournalPage { width: stretch; height: stretch; }
.spread { width: stretch; padding: calc(env(safe-area-inset-top) + var(--space-lg)) var(--space-lg) calc(env(safe-area-inset-bottom) + var(--space-lg)); gap: var(--space-lg); }
#Inventory { display: flex; align-items: start; }
#Bag { flex: 3 1 0; min-width: 0; }
#Doll { flex: 2 1 0; min-width: 0; }
#Stress { display: grid; grid-template-columns: calc(14 * var(--space-lg)) 1fr calc(26 * var(--space-lg)); grid-template-rows: 1fr auto; height: stretch; }
#Party { grid-column: 1; grid-row: 1; gap: var(--space-md); }
#Talk { grid-column: 2; grid-row: 2; }
#Hud { grid-column: 1; grid-row: 2; width: stretch; gap: var(--space-lg); align-items: center; }
#Tracker, #Compass { width: stretch; }
#World { grid-column: 2; grid-row: 1; min-width: 0; gap: var(--space-md); }
#MapWrap { position: relative; width: stretch; flex: 1 1 0; min-height: calc(13 * var(--space-lg)); }
#Quests { grid-column: 3; grid-row: 1 / 3; min-height: 0; }
@container (width < 1100px) {
  #Inventory { flex-direction: column; align-items: stretch; }
  #Stress { display: flex; flex-direction: column; align-items: stretch; height: auto; }
  #Bag, #Doll { flex: none; width: stretch; }
  #Party, #Quests { width: stretch; }
  #World { flex: none; width: stretch; }
  #MapWrap { flex: none; height: auto; }
}
@container (width < 600px) {
  .spread { padding-inline: var(--space-sm); gap: var(--space-md); }
    }
</style>
