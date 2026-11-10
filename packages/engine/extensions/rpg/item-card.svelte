<meta name="component" content="RpgItemCard">
<meta name="strings" content="./rpg.en.ogt">
<meta name="prop:name" content="string" data-doc="The item's name.">
<meta name="prop:icon" content="string" data-doc="Its icon in the style's or the game's atlas, drawn large at the top.">
<meta name="prop:art" content="number" data-doc="A painted picture of the item (og.load.texture) shown large in place of its icon; 0 shows the icon.">
<meta name="prop:rarity" content="string" data-doc="common, uncommon, rare, epic, legendary or mythic; the name and the art take its ink.">
<meta name="prop:kind" content="string" data-doc="The rarity and kind line as the game words it (Legendary · Longsword).">
<meta name="prop:lead" content="list" data-doc="The headline figure, zero or one row of { id, icon, label, value, delta, trend, current }: trend is up, down or none; delta and current are text, empty hides them.">
<meta name="prop:stats" content="list" data-doc="The other figures, keyed rows of { id, icon, label, value, delta, trend, current }.">
<meta name="prop:lore" content="string" data-doc="The flavour text; empty hides it.">
<meta name="prop:primary" content="string" data-doc="The main button's text (Equip, Use); empty hides it.">
<meta name="prop:secondary" content="string" data-doc="The second button's text (Drop, Sell); empty hides it.">
<meta name="prop:canPrimary" content="bool" data-doc="The main button is enabled.">
<meta name="prop:canSecondary" content="bool" data-doc="The second button is enabled.">
<meta name="action:details" content="Equip">
<meta name="fixture:default" content='{"name":"Emberwake Blade","icon":"item-blade","art":0,"rarity":"legendary","kind":"Legendary · Longsword","lead":[{"id":"dmg","icon":"sword","label":"Damage","value":"12–20","delta":"+4","trend":"up","current":"8–16"}],"stats":[{"id":"crit","icon":"star","label":"Critical chance","value":"12%","delta":"+4%","trend":"up","current":"8%"},{"id":"res","icon":"shield","label":"Spell resistance","value":"+10%","delta":"−5%","trend":"down","current":"+15%"}],"lore":"Forged in a dying star.","primary":"Equip","secondary":"Drop","canPrimary":true,"canSecondary":true}'>
<meta name="fixture:plain" content='{"name":"Bread","icon":"item-bread","art":0,"rarity":"common","kind":"Common · Food","lead":[],"stats":[],"lore":"","primary":"Use","secondary":"","canPrimary":true,"canSecondary":false}'>

<!--
  One item in detail: its art, name and kind, a headline figure with its
  change against what is worn now, the other figures as rows, a line of
  lore and up to two buttons. The game words every figure and decides the
  trend; the card reports equip (the main button, X or pad Y) and
  discard (the second button). It fills the height it is given: the
  figures and the lore scroll, the buttons keep the foot.
-->
<article id="RpgItemCard" class="rpg-item-card" data-rarity={rarity} aria-label={name} ondetails={equip}>
  <div class="hero"><div class="hero-glow"></div><img class="hero-picture" src={art} alt="" hidden={!art}><icon class="hero-art" icon={icon} hidden={!!art}></icon></div>
  <header class="head">
    <h2 class="item-name">{name}</h2>
    <p class="item-kind">{kind}</p>
  </header>
  <div class="rpg-rule" role="separator"></div>
  <scroll class="body">
    {#each lead as s (s.id)}
      <div class="lead-row" aria-label={s.label}>
        <icon class="stat-mark" icon={s.icon}></icon>
        <div class="lead-main">
          <span class="caption">{s.label}</span>
          <div class="lead-figures"><span class="lead-value">{s.value}</span><span class="delta" data-trend={s.trend} hidden={!s.delta}>{s.delta}</span></div>
        </div>
        <div class="lead-current" hidden={!s.current}>
          <span class="caption">{t("rpg.card.current")}</span>
          <span class="current-value">{s.current}</span>
        </div>
      </div>
    {/each}
    <div class="stats" role="table">
      {#each stats as s (s.id)}
        <div class="stat-row" role="row">
          <icon class="stat-mark" icon={s.icon}></icon>
          <span class="stat-label" role="rowheader">{s.label}</span>
          <span class="stat-value" role="cell">{s.value}</span>
          <span class="delta" role="cell" data-trend={s.trend}>{s.delta}</span>
          <span class="stat-current" role="cell">{s.current}</span>
        </div>
      {/each}
    </div>
    <p class="lore" hidden={!lore}>{lore}</p>
  </scroll>
  <div class="rpg-rule" role="separator" hidden={!primary && !secondary}></div>
  <footer class="actions" hidden={!primary && !secondary}>
    <button id="Primary" class="primary act" hidden={!primary} disabled={!canPrimary} onclick={equip}>{primary}</button>
    <button id="Secondary" class="secondary act" hidden={!secondary} disabled={!canSecondary} onclick={discard}>{secondary}</button>
  </footer>
</article>

<style>
.rpg-item-card[hidden], .rpg-item-card [hidden] { display: none; }
.rpg-item-card { display: flex; flex-direction: column; width: stretch; height: stretch; gap: var(--space-xs); container-type: inline-size; }
.hero { position: relative; display: flex; width: stretch; height: calc(var(--size-xxl) * 1.25); flex: none; align-items: center; justify-content: center; }
.hero-glow { position: absolute; left: 6%; top: 0; width: 88%; height: stretch; }
.hero-picture { position: relative; width: stretch; height: stretch; object-fit: contain; }
.hero-art { position: relative; width: calc(var(--size-xxl) * 1.8); height: calc(var(--size-xxl) * 1.8); }
.head { display: flex; flex-direction: column; width: stretch; gap: var(--space-xxs); align-items: center; text-align: center; }
.item-name, .item-kind { margin: 0; }
.body { width: stretch; flex: 1; min-height: 0; gap: var(--space-sm); }
.lead-row { display: flex; flex-wrap: wrap; width: stretch; align-items: center; row-gap: var(--space-xs); gap: var(--space-sm); padding: 0 0 var(--space-xxs); }
.lead-main { display: flex; flex-direction: column; width: auto; flex: 1 0 auto; gap: var(--space-xxs); }
.lead-figures { display: flex; align-items: baseline; gap: var(--space-sm); }
.lead-current { display: flex; flex-direction: column; flex: none; gap: var(--space-xxs); padding-inline-start: var(--space-md); align-items: flex-start; }
.stat-mark { width: var(--size-sm); height: var(--size-sm); flex: none; }
.stats { width: stretch; }
.stat-row { display: grid; grid-template-columns: var(--size-sm) 1fr auto 4.5em 3.5em; column-gap: var(--space-sm); align-items: center; width: stretch; padding: var(--space-xxs) 0; }
.stat-value, .delta, .stat-current, .lead-value, .current-value { font-variant-numeric: tabular-nums; white-space: nowrap; }
.stat-value, .stat-current { text-align: end; }
.lore { margin: 0; padding: var(--space-xs) var(--space-xs); flex: none; }
@media (width < 600px), (height < 500px) { .lore { display: none; } }
.actions { display: flex; flex-direction: column; width: stretch; gap: var(--space-sm); flex: none; }
.act { width: stretch; min-height: calc(var(--size-touch) * 1.25); }
@media (width < 600px), (height < 500px) {
  .hero { height: calc(var(--size-xxl) * 0.55); }
  .hero-art { width: calc(var(--size-xxl) * 0.6); height: calc(var(--size-xxl) * 0.6); }
  .actions { flex-direction: row; }
  .act { min-height: var(--size-touch); }
}
@container (width < 360px) { .stat-row { grid-template-columns: var(--size-sm) 1fr auto 4em; } .stat-current { display: none; } }
</style>
