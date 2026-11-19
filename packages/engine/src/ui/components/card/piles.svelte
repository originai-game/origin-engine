<meta name="component" content="CardPiles">
<meta name="strings" content="./card.en.ogt">
<meta name="parts" content="CardPiles.frame CardPiles.pile CardPiles.stack CardPiles.count CardPiles.label">
<meta name="prop:piles" content="list" data-doc="The piles in the game's order, keyed rows of { id, kind, label, icon, count, enabled }: kind is draw, discard, exhaust or the game's own word (it reaches the pile as data-kind); count the game's number; enabled false when the game will not open it.">
<meta name="prop:label" content="string" data-doc="What a screen reader calls the group; empty is Piles.">
<meta name="fixture:default" content='{"label":"","piles":[{"id":"draw","kind":"draw","label":"Draw","icon":"bag","count":14,"enabled":true},{"id":"discard","kind":"discard","label":"Discard","icon":"back","count":3,"enabled":true},{"id":"exhaust","kind":"exhaust","label":"Exhaust","icon":"fire","count":1,"enabled":true}]}'>
<meta name="fixture:empty" content='{"label":"","piles":[{"id":"draw","kind":"draw","label":"Draw","icon":"bag","count":0,"enabled":true},{"id":"discard","kind":"discard","label":"Discard","icon":"back","count":0,"enabled":true},{"id":"exhaust","kind":"exhaust","label":"Exhaust","icon":"fire","count":0,"enabled":false}]}'>
<meta name="fixture:overflow" content='{"label":"","piles":[{"id":"draw","kind":"draw","label":"Remaining in the draw pile","icon":"bag","count":1284,"enabled":true},{"id":"discard","kind":"discard","label":"Discarded this combat","icon":"back","count":312,"enabled":true},{"id":"exhaust","kind":"exhaust","label":"Exhausted for good","icon":"fire","count":96,"enabled":false}]}'>

<!--
  The draw, discard and exhaust piles as buttons: a stack of card backs
  (one when it is empty) with the pile's icon on top, the game's count on
  a tag at its corner and a label under it. A pile at 0 carries data-empty;
  one the game will not open is disabled. Reports open (a pile's key). The
  piles sit in a row spread across the width they are given; a page that
  wants the draw pile in one corner and the discard in the other mounts two.
-->
<nav id="CardPiles" part="CardPiles.frame" class="bare" data-layout="row" aria-label={label || t("card.piles.name")}>
  {#each piles as p (p.id)}
    <button part="CardPiles.pile" data-kind={p.kind} data-empty={p.count == 0} disabled={!p.enabled} aria-label={t("card.piles.count", { label: p.label, count: p.count })} onclick={open}><div class="face" data-layout="stack" data-gap="hair">
      <div part="CardPiles.stack" class="stack">
        {#if p.count > 1}<div class="back under-2"></div>{/if}
        {#if p.count > 0}<div class="back under-1"></div>{/if}
        <div class="back top" data-layout="row" data-gap="none"><og-icon class="icon" icon={p.icon}></og-icon></div>
        <span part="CardPiles.count" class="count">{formatNumber(p.count)}</span>
      </div>
      <span part="CardPiles.label" class="label">{p.label}</span>
    </div></button>
  {/each}
</nav>

<style>
:host { inline-size: stretch; align-items: flex-end; justify-content: space-between; gap: var(--space-sm); }
[part="CardPiles.pile"] { flex: 0 1 auto; min-width: var(--size-touch); min-height: var(--size-touch); padding: var(--space-xs); align-items: center; justify-content: center; border: none; border-radius: 0; background: var(--inv-bare-fill); box-shadow: none; }
.face { align-items: center; min-width: 0; }
/* The stack: up to three backs, each a step up and in from the last. */
.stack { position: relative; inline-size: var(--size-lg); block-size: calc(var(--size-lg) * 1.4); margin-inline-end: var(--space-xs); margin-block-start: var(--space-xs); }
.back { position: absolute; inset: 0; background: var(--tcg-back-fill); box-shadow: var(--tcg-back-rim); }
.under-2 { translate: calc(var(--space-xxs) * -2) calc(var(--space-xxs) * -2); }
.under-1 { translate: calc(var(--space-xxs) * -1) calc(var(--space-xxs) * -1); }
.top { justify-content: center; align-items: center; }
[data-empty=true] .top { background: var(--inv-bare-fill); box-shadow: var(--tcg-back-empty-rim); }
.icon { width: 56%; height: 56%; color: var(--tcg-label-ink); }
/* The count on a cut tag over the stack's foot corner. */
[part="CardPiles.count"] { position: absolute; inset-inline-end: calc(var(--space-xs) * -1); bottom: calc(var(--space-xxs) * -1); min-width: var(--size-xs); padding: 0 var(--space-xxs); text-align: center; white-space: nowrap; clip-path: polygon(var(--inv-tag-cut) 0, 100% 0, 100% calc(100% - var(--inv-tag-cut)), calc(100% - var(--inv-tag-cut)) 100%, 0 100%, 0 var(--inv-tag-cut)); background: var(--tcg-count-fill); color: var(--tcg-count-ink); font-size: var(--tcg-count-size); font-weight: var(--inv-figure-weight); font-stretch: var(--inv-figure-stretch); }
[data-kind=discard] [part="CardPiles.count"] { background: var(--tcg-count-discard-fill); }
[data-empty=true] [part="CardPiles.count"] { background: var(--tcg-count-empty-fill); color: var(--tcg-count-empty-ink); }
[part="CardPiles.label"] { max-width: calc(var(--size-xxl) * 1.5); white-space: nowrap; text-overflow: ellipsis; overflow: hidden; text-align: center; font-size: var(--inv-kicker-size); font-weight: var(--inv-name-weight); letter-spacing: var(--inv-kicker-tracking); text-transform: var(--inv-kicker-transform); color: var(--tcg-label-ink); }
</style>
