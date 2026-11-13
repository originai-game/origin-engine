<meta name="component" content="BuilderPage">

<!-- The deck builder: the collection's filter tabs and card grid beside the
     deck list with its size against the limit. A phone: the collection over
     the deck list. -->
<div id="Page">
  <div id="Library"><CardCollection title={$colTitle} filters={$colFilters} filter={$colFilter} cards={$colCards} selected={$colSelected} empty="" /></div>
  <div id="Deck"><CardDeckList title={$deckTitle} entries={$deckEntries} total={$deckCount} limit={$deckLimit} selected={$deckSelected} confirmLabel={$deckConfirm} canConfirm={false} /></div>
</div>

<style>
#Page { display: flex; width: stretch; height: stretch; gap: var(--space-lg); padding: var(--space-xl); background: var(--backdrop); }
#Library { flex: 1; min-width: 0; height: stretch; }
#Deck { width: 30%; height: stretch; }
@media (width < 600px) {
  #Page { flex-direction: column; gap: var(--space-sm); padding: var(--space-sm); }
  #Library { width: stretch; flex: none; height: 58%; }
  #Deck { width: stretch; flex: 1; min-height: 0; }
}
</style>
