<meta name="component" content="TablePage">

<!-- A combat turn: the draw pile bottom left, the discard and exhaust piles
     bottom right and the hand between them, a card picked up with Play on
     it. A phone: the piles in a row over a full-width hand. -->
<div id="Page">
  <div id="Field"></div>
  <div id="Foot">
    <div id="Draw"><CardPiles piles={$drawPiles} label="" /></div>
    <div id="Hand"><CardHand cards={$handCards} selected={$handSelected} fan={$handFan} label="" /></div>
    <div id="Spent"><CardPiles piles={$spentPiles} label="" /></div>
  </div>
</div>

<style>
#Page { display: flex; flex-direction: column; width: stretch; height: stretch; padding: var(--space-xl); background: var(--backdrop); }
#Field { width: stretch; flex: 1; min-height: 0; }
#Foot { display: flex; width: stretch; height: calc(var(--size-xxl) * 3.6); align-items: flex-end; gap: var(--space-lg); }
#Draw { flex: none; }
#Hand { flex: 1; min-width: 0; height: stretch; }
#Spent { flex: none; }
@media (width < 600px) {
  #Page { padding: var(--space-sm); }
  #Foot { display: grid; grid-template-columns: auto 1fr; grid-template-rows: auto 1fr; grid-template-areas: "draw spent" "hand hand"; height: calc(var(--size-xxl) * 3.2); gap: var(--space-xs); }
  #Draw { grid-area: draw; }
  #Spent { grid-area: spent; justify-self: end; }
  #Hand { grid-area: hand; width: stretch; }
}
</style>
