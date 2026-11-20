<meta name="component" content="RpgItemCard">
<meta name="strings" content="./item-card.en.ogt">
<meta name="prop:name" content="string" data-doc="The item's name.">
<meta name="prop:icon" content="string" data-doc="Its icon in the style's or the game's atlas, drawn large at the top.">
<meta name="prop:art" content="number" data-doc="A painted picture of the item (og.load.texture) shown large in place of its icon; 0 shows the icon.">
<meta name="prop:rarity" content="string" data-doc="common, uncommon, rare, epic, legendary or mythic; the name and the art take its ink.">
<meta name="prop:kind" content="string" data-doc="The rarity and kind line as the game words it (Legendary · Longsword), a kicker over the name in the rarity's ink.">
<meta name="prop:line" content="string" data-default="" data-doc="A line under the name as the game words it (One-handed · Item level 60); empty hides it.">
<meta name="prop:lead" content="list" data-doc="The headline figure, zero or one row of { id, icon, label, value, delta, trend, current }: trend is up, down or none; delta and current are text, empty hides them.">
<meta name="prop:stats" content="list" data-doc="The other figures, keyed rows of { id, icon, label, value, delta, trend, current }.">
<meta name="prop:lore" content="string" data-doc="The flavour text; empty hides it.">
<meta name="prop:primary" content="string" data-doc="The main button's text (Equip, Use); empty hides it.">
<meta name="prop:secondary" content="string" data-doc="The second button's text (Drop, Sell); empty hides it.">
<meta name="prop:canPrimary" content="bool" data-doc="The main button is enabled.">
<meta name="prop:canSecondary" content="bool" data-doc="The second button is enabled.">
<meta name="prop:reason" content="string" data-default="" data-doc="Why the main button is refused, as the game words it (Requires level 40); shown in red over the buttons while canPrimary is false.">
<meta name="action:details" content="Equip">
<meta name="fixture:default" content='{"name":"Emberwake Blade","icon":"sword","art":0,"rarity":"legendary","kind":"Legendary · Longsword","lead":[{"id":"dmg","icon":"sword","label":"Damage","value":"12–20","delta":"+4","trend":"up","current":"8–16"}],"stats":[{"id":"crit","icon":"star","label":"Critical chance","value":"12%","delta":"+4%","trend":"up","current":"8%"},{"id":"res","icon":"shield","label":"Spell resistance","value":"+10%","delta":"−5%","trend":"down","current":"+15%"}],"line":"One-handed · Item level 60","lore":"Forged in a dying star.","primary":"Equip","secondary":"Drop","canPrimary":true,"canSecondary":true}'>
<meta name="fixture:plain" content='{"name":"Bread","icon":"apple","art":0,"rarity":"common","kind":"Common · Food","lead":[],"stats":[],"lore":"","primary":"Use","secondary":"","canPrimary":true,"canSecondary":false}'>
<meta name="fixture:refused" content='{"name":"Staff of the Drowned Choir and the Last Hymn","icon":"drop","art":0,"rarity":"epic","kind":"Epic · Staff","line":"Two-handed","lead":[{"id":"sp","icon":"star","label":"Spell power","value":"38","delta":"−6","trend":"down","current":"44"}],"stats":[{"id":"int","icon":"star","label":"Intellect","value":"+18","delta":"+2","trend":"up","current":"+16"}],"lore":"","primary":"Equip","secondary":"Drop","canPrimary":false,"canSecondary":true,"reason":"Requires level 40"}'>

<!--
  One item in detail: its art large over a glow of its rarity, the rarity
  and kind as a kicker in the rarity's ink over the name, the headline
  figure large with its change against what is equipped, the other figures
  as rows (this, change, now), the lore under a rule of the rarity, and up
  to two buttons, with the reason the main one is refused in words. The
  game words every figure and decides the trend; the card reports equip
  (the main button, X or pad Y) and discard (the second button). It fills
  the height it is given: the figures and the lore scroll when needed; the
  buttons stay at its foot; the art and the name scroll with the figures. Narrow, the art sits in a socket beside the
  name and the rows drop the equipped column.
-->
<article class="frame" id="RpgItemCard" part="frame" layout="stack" fill tier={rarity || 'common'} aria-label={name} ondetails={equip}>
  <div class="lit"></div>
  <div class="body" layout="scroll" gap="tight">
    <div class="top" layout="stack" gap="none">
      <div part="hero" class="hero cover" layout="stack" fill><div class="hero-glow"></div>{#if art}<img class="hero-art picture" src={art} alt="">{:else}<og-icon class="hero-art" icon={icon}></og-icon>{/if}</div>
      <header part="band" class="band" layout="stack" gap="hair">
        {#if kind}<span part="kicker" class="kicker" aria-label={kind}>{kind}</span>{/if}
        <h2 part="name" class="name" aria-label={name}>{name}</h2>
        {#if line}<p part="caption" class="line" aria-label={line}>{line}</p>{/if}
      </header>
    </div>
    {#each lead as s (s.id)}
      <div part="lead" variant="lead" layout="row" aria-label={s.label}>
        <div class="lead-main" layout="stack" gap="hair" fill>
          <span class="kicker">{s.label}</span>
          <div class="lead-figures" layout="row" gap="tight"><span part="figure" class="big">{s.value}</span>{#if s.delta}<div part="delta" class="delta" layout="row" gap="hair" trend={s.trend} class:trend-down={(s.trend) == 'down'} class:trend-up={(s.trend) == 'up'}><div class="arrow"></div><span>{s.delta}</span></div>{/if}</div>
        </div>
        {#if s.current}<div part="current" class="lead-current" layout="stack" gap="hair">
          <span class="kicker">{t("rpg.card.current")}</span>
          <span part="figure" class="now">{s.current}</span>
        </div>{/if}
      </div>
    {/each}
    {#if stats.length}<div part="stats" class="stats" role="table" aria-label={name}>
      <div class="head-row" layout="row" role="row"><span class="kicker label" role="columnheader" aria-label={t("rpg.card.stat")}></span><div class="figs" layout="row"><span class="kicker value" role="columnheader">{t("rpg.card.this")}</span><div class="change"></div><span class="kicker was wide" role="columnheader">{t("rpg.card.current")}</span></div></div>
      {#each stats as s (s.id)}
        <div part="row" class="row" layout="row" role="row">
          <div class="label" layout="row" gap="tight"><og-icon class="mark" icon={s.icon}></og-icon><span role="rowheader">{s.label}</span></div>
          <div class="figs" layout="row">
          <span part="figure" class="value" role="cell">{s.value}</span>
          <div class="change" layout="row" role="cell">{#if s.delta}<div part="delta" class="delta" layout="row" gap="hair" trend={s.trend} class:trend-down={(s.trend) == 'down'} class:trend-up={(s.trend) == 'up'}><div class="arrow"></div><span>{s.delta}</span></div>{/if}</div>
          <span part="figure" class="was wide" role="cell">{s.current}</span>
          </div>
        </div>
      {/each}
    </div>{/if}
    {#if lore}<div part="lore" class="lore" layout="stack"><p part="caption" class="lore-text">{lore}</p></div>{/if}
  </div>
  {#if primary || secondary}<footer part="actions" class="actions" layout="stack" gap="tight">
    {#if primary && !canPrimary && reason}<p part="reason" class="reason" role="status">{reason}</p>{/if}
    <div class="buttons" layout="row" gap="tight">
      {#if primary}<button id="Primary" class="go act main" variant="primary" disabled={!canPrimary} aria-label={primary} onclick={equip}><kbd class="cap" data-action="details"></kbd><span>{primary}</span></button>{/if}
      {#if secondary}<button id="Secondary" class="secondary act" disabled={!canSecondary} onclick={discard}>{secondary}</button>{/if}
    </div>
  </footer>{/if}
</article>

<style>
.cover { justify-content: center; align-items: center; }
/* The frame: two corners cut, a lit top edge; the rarity tints the hero's
   glow, the kicker, the name and the lore's rule. */
.frame { --inv-rarity: var(--common); position: relative; min-block-size: 0; border-radius: var(--inv-frame-cut) 0; corner-shape: var(--cut-shape); clip-path: border-box; }
.frame.rarity-uncommon { --inv-rarity: var(--uncommon); --inv-socket-fill: var(--cell-uncommon-fill); --inv-socket-rim: var(--cell-uncommon-shadow); }
.frame.rarity-rare { --inv-rarity: var(--rare); --inv-socket-fill: var(--cell-rare-fill); --inv-socket-rim: var(--cell-rare-shadow); }
.frame.rarity-epic { --inv-rarity: var(--epic); --inv-socket-fill: var(--cell-epic-fill); --inv-socket-rim: var(--cell-epic-shadow); }
.frame.rarity-legendary { --inv-rarity: var(--legendary); --inv-socket-fill: var(--cell-legendary-fill); --inv-socket-rim: var(--cell-legendary-shadow); }
.frame.rarity-mythic { --inv-rarity: var(--mythic); --inv-socket-fill: var(--cell-mythic-fill); --inv-socket-rim: var(--cell-mythic-shadow); }
.lit { position: absolute; inset-inline: var(--inv-frame-cut) 30%; top: 0; block-size: var(--stroke-hairline); background: var(--inv-lit-edge); pointer-events: none; }
h2, p { margin: 0; }
.top { flex: none; width: stretch; }
.hero { position: relative; flex: none; width: stretch; block-size: var(--icard-hero-size); color: var(--inv-rarity); border-block-end: var(--icard-hero-rule); }
.hero-glow { position: absolute; inset: 0; background: var(--inv-hero-glow); opacity: 0.4; }
.hero-art { position: relative; inline-size: var(--icard-art-size); block-size: var(--icard-art-size); filter: var(--inv-slot-art-shadow); }
.picture { object-fit: contain; }
.band { width: stretch; padding-block-start: var(--space-sm); }
.kicker { font-size: var(--inv-kicker-size); font-weight: var(--inv-name-weight); letter-spacing: var(--inv-kicker-tracking); text-transform: var(--inv-kicker-transform); color: var(--inv-kicker-ink); }
.band .kicker { color: var(--inv-rarity); overflow-wrap: anywhere; }
.name { font-size: var(--inv-figure-xl-size); color: var(--inv-rarity); text-shadow: var(--icard-name-glow); overflow-wrap: anywhere; }
.rarity-common .name { color: var(--ink-bright); text-shadow: none; }
/* The art and the name scroll with the figures, so a long name at large
   text never leaves the figures without room. */
.body { flex: 0 1 auto; min-block-size: 0; width: stretch; }
/* The headline figure, large, with its change; what is equipped beside it. */
.lead { flex: none; flex-wrap: wrap; align-items: flex-end; column-gap: var(--space-md); row-gap: var(--space-xxs); padding: var(--space-sm) var(--space-md); background: var(--inv-row-fill); box-shadow: var(--inv-row-rim); border-radius: var(--inv-slot-cut) 0; corner-shape: var(--cut-shape); clip-path: border-box; }
.lead-main { flex: 1 0 auto; }
.lead-figures { align-items: center; flex-wrap: wrap; }
.big { font-size: var(--icard-lead-size); line-height: 1; font-weight: var(--inv-figure-weight); font-stretch: var(--inv-figure-stretch); font-variant-numeric: var(--hud-figure-numeric); white-space: nowrap; }
.lead-current { flex: none; margin-inline-start: auto; align-items: flex-end; text-align: end; }
.now { font-size: var(--inv-figure-lg-size); font-weight: var(--inv-figure-weight); font-stretch: var(--inv-figure-stretch); color: var(--ink-label); white-space: nowrap; }
/* A change: a chip in the trend's colour with an arrow. */
.delta { flex: none; align-items: center; padding: 0 var(--space-xs); font-weight: var(--inv-figure-weight); font-stretch: var(--inv-figure-stretch); white-space: nowrap; color: var(--ink-label); border-radius: var(--inv-tag-cut) 0; corner-shape: var(--cut-shape); clip-path: border-box; }
.delta.trend-up { color: var(--icard-up-ink); background: var(--icard-up-fill); }
.delta.trend-down { color: var(--icard-down-ink); background: var(--icard-down-fill); }
.arrow { display: none; flex: none; inline-size: var(--icard-arrow-size); block-size: var(--icard-arrow-size); background: currentColor; }
.delta.trend-up .arrow { display: flex; clip-path: polygon(50% 10%, 100% 90%, 0 90%); }
.delta.trend-down .arrow { display: flex; clip-path: polygon(0 10%, 100% 10%, 50% 90%); }
/* The other figures: this, its change and what is equipped now, in
   columns of fixed width at the line's end; when the line is too short
   for the label beside them, they drop under it. */
.stats { flex: none; }
.head-row, .row { flex-wrap: wrap; align-items: center; column-gap: var(--space-xs); row-gap: var(--space-xxs); }
/* The header names only the figure columns, on one line over them even
   when a row's figures drop under its label. */
.head-row { flex-wrap: nowrap; padding-block: var(--space-xxs); }
.head-row .label { flex: 1 1 0; }
.row { min-block-size: calc(40 * var(--font-step-px)); padding-block: var(--space-xxs); border-block-start: var(--inv-rule); }
.label { flex: 1 1 var(--icard-label-col); min-width: 0; align-items: center; color: var(--ink-label); overflow-wrap: anywhere; }
.head-row .label { padding-inline-start: calc(var(--size-sm) + var(--space-xs)); }
.mark { flex: none; width: var(--size-sm); height: var(--size-sm); }
.figs { flex: none; margin-inline-start: auto; align-items: center; gap: var(--space-xs); }
.value { inline-size: var(--icard-value-col); font-weight: var(--inv-figure-weight); font-stretch: var(--inv-figure-stretch); text-align: end; white-space: nowrap; }
.change { inline-size: var(--icard-change-col); }
.was { inline-size: var(--icard-now-col); color: var(--ink-label); text-align: end; white-space: nowrap; }
/* A header's label can be wider than its figure column (EQUIPPED, a
   translation). The header's figures keep the rows' total width and the
   empty change column between the two labels gives up its room, so each
   label ends over its column and none runs past the row, where it made
   the body scroll sideways. */
.head-row .figs { inline-size: calc(var(--icard-value-col) + var(--icard-change-col) + var(--icard-now-col) + 2 * var(--space-xs)); }
.head-row .value { flex: none; inline-size: auto; min-inline-size: var(--icard-value-col); }
.head-row .change { flex: 1 1 0; inline-size: auto; min-inline-size: 0; }
.head-row .was { flex: 0 1 auto; inline-size: auto; min-inline-size: var(--icard-now-col); overflow: hidden; text-overflow: ellipsis; }
.lore { flex: none; padding-inline-start: var(--space-sm); border-inline-start: var(--icard-lore-rule); border-inline-start-color: var(--inv-rarity); }
.lore-text { font-style: var(--icard-lore-style); }
/* The buttons at the foot, the refusal in words over them. */
.actions { flex: none; width: stretch; padding-block-start: var(--space-sm); border-block-start: var(--inv-rule); }
.reason { color: var(--inv-broken-ink); overflow-wrap: anywhere; }
.buttons { width: stretch; }
.act { flex: 1 1 0; min-width: 0; min-block-size: calc(var(--size-touch) * 1.15); gap: var(--space-xs); }
.main { flex-grow: 1.4; }
.cap { flex: none; }
@media (width < 600px), (height < 500px) {
  /* Narrow or short: the art in a socket beside the name, no lore, the
     rows without the equipped column. */
  .top { flex-direction: row; align-items: center; gap: var(--space-sm); }
  .hero { inline-size: var(--icard-socket-size); block-size: var(--icard-socket-size); width: auto; flex: none; border: none; background: var(--inv-socket-fill); box-shadow: var(--inv-socket-rim); border-radius: var(--inv-slot-cut) 0; corner-shape: var(--cut-shape); clip-path: border-box; }
  .hero-glow { display: none; }
  .hero-art { inline-size: 80%; block-size: 80%; }
  .band { min-width: 0; flex: 1 1 0; padding: 0; }
  /* The kicker and the line keep one line each, so the figures keep the
     card's height at large text. */
  .band .kicker, .line { white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .name { font-size: var(--inv-figure-lg-size); line-clamp: 2; overflow: hidden; }
  .big { font-size: var(--inv-figure-xl-size); }
  .head-row { display: none; }
  .wide { display: none; }
  .lore { display: none; }
  .act { min-block-size: var(--size-touch); }
}
</style>
