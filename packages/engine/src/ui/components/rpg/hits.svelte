<meta name="component" content="RpgHits">
<meta name="strings" content="./rpg.en.ogt">
<meta name="parts" content="RpgHits.frame RpgHits.hit RpgHits.call RpgHits.amount RpgHits.kind">
<meta name="prop:hits" content="list" data-doc="The latest damage and healing figures, newest first, keyed rows of { id, amount, kind, call, word }: kind normal, critical (gold and slanted, the largest), burn (damage over time in an ember ink, its word beside it), heal (green with a plus) or miss (on data-kind); call the words over a critical as the game words them (Critical · Weak point), empty for none; word the figure's kind as the game words it (burn), empty for none. The newest stands largest; when there are older ones it keeps room before it, where they step down and away (a phone shows the newest alone); the game trims the list and places it.">
<meta name="fixture:default" content='{"hits":[{"id":"h3","amount":2481,"kind":"critical","call":"Critical · Weak point","word":""},{"id":"h2","amount":642,"kind":"normal","call":"","word":""},{"id":"h1","amount":118,"kind":"burn","call":"","word":"burn"}]}'>
<meta name="fixture:heal" content='{"hits":[{"id":"r1","amount":320,"kind":"heal","call":"","word":""}]}'>
<meta name="fixture:miss" content='{"hits":[{"id":"m1","amount":0,"kind":"miss","call":"","word":"Miss"}]}'>
<meta name="fixture:empty" content='{"hits":[]}'>
<meta name="fixture:overflow" content='{"hits":[{"id":"o1","amount":12345678,"kind":"critical","call":"Critical · Weak point · Overkill · Backstab","word":""},{"id":"o2","amount":999999,"kind":"burn","call":"","word":"searing flame"}]}'>

<!--
  Damage and healing figures beside the fight: a critical hit's figure
  largest, gold and slanted under its call, older figures stepping down
  and away, a burn in an ember ink with its word, a heal in green. The
  game owns every figure and where the page puts them; read only.
-->
<section id="RpgHits" part="RpgHits.frame" class="hud bare" data-layout="stack" data-gap="none" role="log" aria-label={t("rpg.hits.name")}>
  {#each hits as h (h.id)}
  <div part="RpgHits.hit" class="hit" data-layout="stack" data-gap="none" data-kind={h.kind} data-motion="pop" out:fade={{ duration: 200 }}>
    {#if h.call}<span part="RpgHits.call" class="call">{h.call}</span>{/if}
    <div class="line" data-layout="row" data-gap="tight">
      {#if h.kind != 'miss'}<span part="RpgHits.amount" class="amount">{h.kind == 'heal' ? t("rpg.hits.heal", { amount: formatNumber(h.amount) }) : formatNumber(h.amount)}</span>{/if}
      {#if h.word}<span part="RpgHits.kind" class="word">{h.word}</span>{/if}
    </div>
  </div>
  {/each}
</section>

<style>
:host { align-items: flex-start; max-inline-size: stretch; pointer-events: none; color: var(--hud-ink); text-shadow: var(--hud-shadow); -webkit-text-stroke-width: var(--hud-stroke-width); -webkit-text-stroke-color: var(--hud-stroke-color); paint-order: stroke fill; }
.hit { align-items: flex-start; max-inline-size: 100%; min-inline-size: 0; }
.line { align-items: baseline; min-inline-size: 0; max-inline-size: 100%; }
/* The call is a short label, not a name: when the room beside the fight is
   narrower than it (a large text scale, a phone) it breaks between its
   words rather than scrolling half of itself out of sight. */
.call { max-inline-size: 100%; min-inline-size: 0; font-size: var(--hud-size-feed); font-weight: var(--hud-label-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); color: var(--hud-gold); }
.amount { flex: none; font-size: var(--hud-size-lg); line-height: 1; font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); font-variant-numeric: var(--hud-figure-numeric); white-space: nowrap; }
.word { min-inline-size: 0; font-size: var(--hud-size-feed); font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); white-space: nowrap; text-overflow: -og-scroll var(--motion-duration-orbit) var(--motion-duration-turn); overflow: hidden; }
[data-kind=critical] .amount { font-size: var(--hud-size-crit); font-style: var(--hud-figure-slant); color: var(--hud-gold); }
[data-kind=burn] .amount, [data-kind=burn] .word { color: var(--hud-burn-ink); }
[data-kind=heal] .amount { color: var(--hud-boon); }
[data-kind=miss] .word { font-size: var(--hud-size-md); color: var(--hud-ink-dim); }
/* Older figures step down and away: when there are older ones the newest
   keeps room before it, and the next figure stands in it, under and left
   of the newest; the third under the newest, past the room's end. A size
   smaller and dimmed. */
.hit:first-child:not(:last-child) { margin-inline-start: calc(150 * var(--font-step-px)); }
.hit + .hit { margin-block-start: calc(28 * var(--font-step-px)); opacity: 0.85; }
.hit + .hit + .hit { margin-inline-start: calc(220 * var(--font-step-px)); margin-block-start: 0; opacity: 0.7; }
.hit + .hit + .hit .amount { font-size: var(--hud-size-md); }
@media (width < 600px), (height < 500px) {
  .amount { font-size: var(--hud-size-md); }
  [data-kind=critical] .amount { font-size: var(--hud-size-crit-compact); }
  .call { font-size: var(--hud-tag-size); letter-spacing: normal; }
  /* A phone shows the newest alone. */
  .hit:first-child:not(:last-child) { margin-inline-start: 0; }
  .hit + .hit { display: none; }
}
</style>
