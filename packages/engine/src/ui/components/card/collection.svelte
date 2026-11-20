<meta name="component" content="CardCollection">
<meta name="strings" content="./card.en.ogt">
<meta name="prop:title" content="string" data-doc="The collection's heading (Collection, Library); empty hides the band.">
<meta name="prop:filters" content="list" data-doc="The filters the game offers (classes, costs, types), keyed rows of { id, label, count } (count above 0 shows beside the label); empty hides the row.">
<meta name="prop:filter" content="string" data-doc="The id of the filter applied; the game filters the cards it passes.">
<meta name="prop:cards" content="list" data-doc="The cards shown, already filtered and sorted by the game: keyed rows of { id, name, cost, icon, art, type, typeLabel, rarity, text, owned, inDeck, canAdd, locked, reason }. owned and inDeck are the game's counts (owned above 1 is shown, inDeck above 0); canAdd is the game's answer (limit, rarity rule, not owned); locked marks a card not owned yet; reason is why the game will not add it, in the game's words (spoken and marked when canAdd is false; empty says Not owned for a locked card, Cannot be added otherwise).">
<meta name="prop:selected" content="string" data-doc="The card shown in detail; empty is none.">
<meta name="prop:empty" content="string" data-doc="What an empty result says; empty is No cards match.">
<meta name="action:tabPrev" content="Previous filter">
<meta name="action:tabNext" content="Next filter">
<meta name="action:details" content="Inspect">
<meta name="fixture:default" content='{"title":"Collection","filter":"all","filters":[{"id":"all","label":"All","count":42},{"id":"attack","label":"Attacks","count":18},{"id":"skill","label":"Skills","count":15},{"id":"power","label":"Powers","count":6}],"selected":"guard","empty":"","cards":[{"id":"strike","name":"Ember Strike","cost":"1","icon":"fire","art":0,"type":"attack","typeLabel":"Attack","rarity":"common","text":"Deal 6 damage.","owned":3,"inDeck":2,"canAdd":true,"locked":false,"reason":""},{"id":"guard","name":"Iron Guard","cost":"1","icon":"shield","art":0,"type":"skill","typeLabel":"Skill","rarity":"common","text":"Gain 5 Block.","owned":2,"inDeck":2,"canAdd":false,"locked":false,"reason":"Both copies are in the deck"},{"id":"surge","name":"Arcane Surge","cost":"3","icon":"bolt","art":0,"type":"power","typeLabel":"Power","rarity":"rare","text":"Draw 2 cards each turn.","owned":1,"inDeck":0,"canAdd":true,"locked":false,"reason":""},{"id":"crown","name":"Hollow Crown","cost":"2","icon":"crown","art":0,"type":"power","typeLabel":"Power","rarity":"legendary","text":"Your attacks deal double damage.","owned":0,"inDeck":0,"canAdd":false,"locked":true,"reason":""}]}'>
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
<section id="CardCollection" part="frame" layout="stack" fill aria-label={title || t("card.collection.cards")}>
  {#if title}<header class="band" part="band" layout="row"><h2 class="title" part="title">{title}</h2></header>{/if}
  {#if filters.length}<div class="filters" role="tablist" aria-label={t("card.collection.filters")} ontabNext={tabNext} ontabPrev={tabPrev}>
    <div class="filters-scroll">
      <div class="filters-row" layout="row" gap="tight">
        {#each filters as f (f.id)}
          <button class="filter" part="filter" variant="quiet" role="tab" aria-selected={f.id == filter} aria-label={f.label} onclick={choosefilter} state:selected={f.id == filter}><span class="filter-label">{f.label}</span>{#if f.count > 0}<span part="count" class="count">{formatNumber(f.count)}</span>{/if}<div class="line"></div></button>
        {/each}
      </div>
    </div>
  </div>{/if}
  {#if cards.length}
    <div class="grid-scroll" layout="scroll">
      <div class="grid" gap="loose" role="grid" aria-label={title || t("card.collection.cards")}>
        {#each cards as c (c.id)}
          <div class="slot" layout="stack" gap="hair">
            {#if c.canAdd}
              <button class="cell" part="cell" role="gridcell" data-type={c.type} tier={c.rarity} data-can-add={c.canAdd} data-locked={c.locked} aria-selected={c.id == selected} aria-label={c.name} aria-description={c.text} onclick={add} ondetails={inspect} state:selected={c.id == selected} class:in-deck={c.inDeck > 0}><div class="cell-face">
                <div class="face"><CardFace name={c.name} cost={c.cost} icon={c.icon} art={c.art} type={c.type} typeLabel={c.typeLabel} rarity={c.rarity} text={c.text} dim={false} /></div>
                {#if c.owned > 0}<span part="owned" class="owned">{t("card.collection.owned", { count: c.owned })}</span>{/if}
                {#if c.id == selected}<div class="bracket"></div>{/if}
              </div></button>
            {:else}
              <button class="cell" part="cell" role="gridcell" data-type={c.type} tier={c.rarity} data-can-add={c.canAdd} data-locked={c.locked} aria-selected={c.id == selected} aria-label={c.name} aria-description={c.reason || (c.locked ? t("card.collection.locked") : t("card.collection.refused"))} onclick={inspect} ondetails={inspect} state:selected={c.id == selected} class:in-deck={c.inDeck > 0}><div class="cell-face">
                <div class="face"><CardFace name={c.name} cost={c.cost} icon={c.icon} art={c.art} type={c.type} typeLabel={c.typeLabel} rarity={c.rarity} text={c.text} dim={c.locked} /></div>
                {#if c.locked}<div class="hatch"></div>{/if}
                {#if c.owned > 0}<span part="owned" class="owned">{t("card.collection.owned", { count: c.owned })}</span>{/if}
                {#if c.id == selected}<div class="bracket"></div>{/if}
              </div></button>
            {/if}
            <span part="deck" class="deck" class:in-deck={c.inDeck > 0}>{c.inDeck > 0 ? t("card.collection.inDeck", { count: c.inDeck }) : t("card.collection.notInDeck")}</span>
            {#if !c.canAdd}<span part="reason" class="reason">{c.reason || (c.locked ? t("card.collection.locked") : t("card.collection.refused"))}</span>{/if}
          </div>
        {/each}
      </div>
    </div>
  {:else}
    <div class="empty cover" layout="stack" fill role="status"><p class="empty-text">{empty || t("card.collection.empty")}</p></div>
  {/if}
</section>

<style>
.cover { justify-content: center; align-items: center; }
.grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(min(var(--layout-tile), 100%), 1fr)); }
#CardCollection { position: relative; border-radius: var(--inv-frame-cut) 0; corner-shape: var(--cut-shape); clip-path: border-box; }
.band { width: stretch; justify-content: flex-start; flex: none; }
.title { margin: 0; }
/* Filters: an underline strip, each with its count; the strip scrolls and fades. */
.filters { width: stretch; flex: none; border-block-end: var(--inv-rule); }
.filters-scroll { width: stretch; overflow-x: auto; overflow-y: hidden; min-width: 0; scrollbar-width: none; mask-image: var(--tcg-fade); }
.filters-row { padding-inline-end: var(--size-lg); }
.filter { position: relative; flex: none; min-height: var(--size-touch); min-width: var(--size-touch); padding: 0 var(--space-sm); gap: var(--space-xs); border-radius: 0; background: var(--inv-bare-fill); box-shadow: none; color: var(--inv-tab-ink); }
.filter:state(selected) { color: var(--inv-tab-on-ink); }
.filter-label { white-space: nowrap; }
.count { padding: 0 var(--space-xs); border-radius: var(--inv-chip-radius); font-size: var(--inv-tag-size); font-weight: var(--inv-figure-weight); background: var(--inv-tab-count-fill); color: var(--tcg-count-empty-ink); }
:state(selected) .count { background: var(--inv-tab-count-on-fill); color: var(--inv-accent-ink); }
.line { position: absolute; inset-inline: var(--space-xs); bottom: 0; block-size: var(--stroke-thick); background: var(--inv-tab-line); box-shadow: var(--inv-tab-glow); opacity: 0; }
:state(selected) .line { opacity: 1; }
/* A card is at least the tile wide: five to a desktop panel, two on a phone. */
.grid { --layout-tile: var(--size-3xl); width: stretch; padding: var(--space-xs); }
.slot { min-width: 0; align-items: center; }
.cell { width: stretch; aspect-ratio: 0.7; padding: 0; align-items: stretch; justify-content: stretch; min-width: 0; border: none; border-radius: 0; background: var(--inv-bare-fill); box-shadow: none; }
.cell-face { position: relative; width: stretch; height: stretch; }
.face { width: stretch; height: stretch; }
.hatch { position: absolute; inset: 0; background: var(--tcg-hatch); pointer-events: none; }
.owned { position: absolute; inset-inline-end: var(--space-xxs); top: var(--space-xxs); padding: 0 var(--space-xxs); border-radius: var(--inv-tag-cut) 0; corner-shape: var(--cut-shape); clip-path: border-box; background: var(--inv-tag-fill); box-shadow: var(--inv-tag-rim); color: var(--ink-bright); font-size: var(--inv-tag-size); font-weight: var(--inv-figure-weight); font-stretch: var(--inv-figure-stretch); }
.bracket { position: absolute; inset: calc(var(--space-xxs) * -1); background: var(--inv-bracket); filter: var(--inv-bracket-glow); pointer-events: none; }
.deck { max-width: stretch; text-align: center; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; font-size: var(--inv-tag-size); font-weight: var(--inv-figure-weight); color: var(--tcg-count-empty-ink); }
.deck.in-deck { color: var(--inv-accent); }
.reason { max-width: stretch; text-align: center; overflow-wrap: anywhere; font-size: var(--inv-tag-size); font-weight: var(--inv-figure-weight); color: var(--tcg-reason-ink); }
.empty { width: stretch; }
.empty-text { margin: 0; }
</style>
