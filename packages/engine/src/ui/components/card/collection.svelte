<meta name="component" content="CardCollection">
<meta name="strings" content="./card.en.ogt">
<meta name="parts" content="CardCollection.frame CardCollection.band CardCollection.title CardCollection.filter CardCollection.cell CardCollection.badge">
<meta name="prop:title" content="string" data-doc="The collection's heading (Collection, Library); empty hides the band.">
<meta name="prop:filters" content="list" data-doc="The filters the game offers (classes, costs, types), keyed rows of { id, label }; empty hides the row.">
<meta name="prop:filter" content="string" data-doc="The id of the filter applied; the game filters the cards it passes.">
<meta name="prop:cards" content="list" data-doc="The cards shown, already filtered and sorted by the game: keyed rows of { id, name, cost, icon, art, type, typeLabel, rarity, text, owned, inDeck, canAdd, locked, reason }. owned and inDeck are the game's counts (owned above 1 is shown, inDeck above 0); canAdd is the game's answer (limit, rarity rule, not owned); locked marks a card not owned yet; reason is why the game will not add it, in the game's words (spoken and marked when canAdd is false; empty says Not owned for a locked card, Cannot be added otherwise).">
<meta name="prop:selected" content="string" data-doc="The card shown in detail; empty is none.">
<meta name="prop:empty" content="string" data-doc="What an empty result says; empty is No cards match.">
<meta name="action:tabPrev" content="Previous filter">
<meta name="action:tabNext" content="Next filter">
<meta name="action:details" content="Inspect">
<meta name="fixture:default" content='{"title":"Collection","filter":"all","filters":[{"id":"all","label":"All"},{"id":"attack","label":"Attacks"},{"id":"skill","label":"Skills"},{"id":"power","label":"Powers"}],"selected":"guard","empty":"","cards":[{"id":"strike","name":"Ember Strike","cost":"1","icon":"fire","art":0,"type":"attack","typeLabel":"Attack","rarity":"common","text":"Deal 6 damage.","owned":3,"inDeck":2,"canAdd":true,"locked":false,"reason":""},{"id":"guard","name":"Iron Guard","cost":"1","icon":"shield","art":0,"type":"skill","typeLabel":"Skill","rarity":"common","text":"Gain 5 Block.","owned":2,"inDeck":2,"canAdd":false,"locked":false,"reason":"Both copies are in the deck"},{"id":"surge","name":"Arcane Surge","cost":"3","icon":"bolt","art":0,"type":"power","typeLabel":"Power","rarity":"rare","text":"Draw 2 cards each turn.","owned":1,"inDeck":0,"canAdd":true,"locked":false,"reason":""},{"id":"crown","name":"Hollow Crown","cost":"2","icon":"crown","art":0,"type":"power","typeLabel":"Power","rarity":"legendary","text":"Your attacks deal double damage.","owned":0,"inDeck":0,"canAdd":false,"locked":true,"reason":""}]}'>
<meta name="fixture:empty" content='{"title":"Collection","filter":"power","filters":[{"id":"all","label":"All"},{"id":"power","label":"Powers"}],"selected":"","empty":"","cards":[]}'>

<!--
  The card library of a deck builder: a heading, filter tabs and a grid
  of cards that fills the height it is given and scrolls inside it. The
  game filters and sorts what it passes; the grid reports add (a card's
  key: tap, Enter, pad A), inspect (X or pad Y on the focused card) and
  choosefilter (a filter's key), tabNext and tabPrev (E/Q, bumpers). A
  card the game will not add never reports add: it stays in reach, so it
  can be focused and read, reports inspect from a tap too, and says why
  in the game's words, with a lock when it is not owned and a cross when
  it is. The core DataGrid is a table of text rows, so the cards are a
  grid of their own.
-->
<section id="CardCollection" part="CardCollection.frame" aria-label={title || t("card.collection.cards")}>
  <header part="CardCollection.band" hidden={!title}><h2 part="CardCollection.title">{title}</h2></header>
  <div class="filters" role="tablist" aria-label={t("card.collection.filters")} hidden={!filters.length} ontabNext={tabNext} ontabPrev={tabPrev}>
    <scroll class="filters-scroll">
      <div class="filters-row">
        {#each filters as f (f.id)}
          <button part="CardCollection.filter" variant="quiet" role="tab" aria-selected={f.id == filter} onclick={choosefilter}>{f.label}</button>
        {/each}
      </div>
    </scroll>
  </div>
  {#if cards.length}
    <scroll class="grid-scroll">
      <div class="grid" role="grid" aria-label={title || t("card.collection.cards")}>
        {#each cards as c (c.id)}
          {#if c.canAdd}
            <button part="CardCollection.cell" role="gridcell" data-type={c.type} data-rarity={c.rarity} data-can-add={c.canAdd} data-locked={c.locked} data-in-deck={c.inDeck > 0} aria-selected={c.id == selected} aria-label={c.name} aria-description={c.text} onclick={add} ondetails={inspect}><div class="cell-face">
              <div class="face"><CardFace name={c.name} cost={c.cost} icon={c.icon} art={c.art} type={c.type} typeLabel={c.typeLabel} rarity={c.rarity} text={c.text} /></div>
              <div class="badges">
                <span part="CardCollection.badge" class="owned" hidden={!(c.owned > 1)}>{t("card.collection.owned", { count: c.owned })}</span>
                <span part="CardCollection.badge" class="in-deck" hidden={!(c.inDeck > 0)}>{t("card.collection.inDeck", { count: c.inDeck })}</span>
              </div>
            </div></button>
          {:else}
            <button part="CardCollection.cell" role="gridcell" data-type={c.type} data-rarity={c.rarity} data-can-add={c.canAdd} data-locked={c.locked} data-in-deck={c.inDeck > 0} aria-selected={c.id == selected} aria-label={c.name} aria-description={c.reason || (c.locked ? t("card.collection.locked") : t("card.collection.refused"))} onclick={inspect} ondetails={inspect}><div class="cell-face">
              <div class="face"><CardFace name={c.name} cost={c.cost} icon={c.icon} art={c.art} type={c.type} typeLabel={c.typeLabel} rarity={c.rarity} text={c.text} /></div>
              <div class="badges">
                <span part="CardCollection.badge" class="owned" hidden={!(c.owned > 1)}>{t("card.collection.owned", { count: c.owned })}</span>
                <span part="CardCollection.badge" class="in-deck" hidden={!(c.inDeck > 0)}>{t("card.collection.inDeck", { count: c.inDeck })}</span>
              </div>
              <div part="CardCollection.badge" class="state" aria-label={c.reason || (c.locked ? t("card.collection.locked") : t("card.collection.refused"))}><og-icon icon={c.locked ? "lock" : "close"}></og-icon></div>
            </div></button>
          {/if}
        {/each}
      </div>
    </scroll>
  {:else}
    <div class="empty" role="status"><p class="empty-text">{empty || t("card.collection.empty")}</p></div>
  {/if}
</section>

<style>
[part="CardCollection.frame"][hidden], [part="CardCollection.frame"] [hidden] { display: none; }
[part="CardCollection.frame"] { display: flex; flex-direction: column; width: stretch; height: stretch; gap: var(--space-sm); container-type: inline-size; }
[part="CardCollection.band"] { display: flex; width: stretch; align-items: center; justify-content: center; flex: none; }
[part="CardCollection.title"] { margin: 0; }
.filters { display: flex; width: stretch; flex: none; }
.filters-scroll { width: stretch; overflow-x: auto; overflow-y: hidden; }
.filters-row { display: flex; gap: var(--space-xs); }
[part="CardCollection.filter"] { flex: none; min-height: var(--size-touch); padding: 0 var(--space-md); }
.grid-scroll { width: stretch; flex: 1; min-height: 0; }
.grid { display: grid; grid-template-columns: repeat(5, 1fr); gap: var(--space-md); width: stretch; padding: var(--space-xs); }
[part="CardCollection.cell"] { width: stretch; aspect-ratio: 0.7; padding: 0; align-items: stretch; justify-content: stretch; }
.cell-face { position: relative; width: stretch; height: stretch; }
.face { width: stretch; height: stretch; }
/* The counts sit on the foot of the art window, clear of the rules text. */
.badges { position: absolute; left: 0; top: calc(46% - var(--size-sm)); display: flex; width: stretch; justify-content: center; gap: var(--space-xxs); }
[part="CardCollection.badge"] { padding: 0 var(--space-xs); white-space: nowrap; font-variant-numeric: tabular-nums; }
.state { position: absolute; right: var(--space-xxs); top: var(--space-xxs); display: flex; width: var(--size-sm); height: var(--size-sm); padding: 0; align-items: center; justify-content: center; }
.state:dir(rtl) { right: auto; left: var(--space-xxs); }
.state og-icon { width: 70%; height: 70%; }
.empty { display: flex; width: stretch; flex: 1; align-items: center; justify-content: center; }
.empty-text { margin: 0; }
@container (width < 720px) { .grid { grid-template-columns: repeat(4, 1fr); gap: var(--space-sm); } }
@container (width < 520px) { .grid { grid-template-columns: repeat(3, 1fr); } }
</style>
