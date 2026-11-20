<meta name="component" content="StrategyTower">
<meta name="strings" content="./strategy.en.ogt">
<meta name="parts" content="StrategyTower.frame StrategyTower.band StrategyTower.title StrategyTower.badge StrategyTower.portrait StrategyTower.row StrategyTower.path StrategyTower.name StrategyTower.cost StrategyTower.reason StrategyTower.picture StrategyTower.figure StrategyTower.caption StrategyTower.headline StrategyTower.card StrategyTower.well StrategyTower.track StrategyTower.fill">
<meta name="prop:name" content="string" data-doc="The tower's name; empty takes the kit's (Tower).">
<meta name="prop:icon" content="string" data-doc="An icon of the atlas for the tower, drawn when portrait is 0.">
<meta name="prop:portrait" content="number" data-doc="A texture of the tower (og.load.texture); 0 draws icon.">
<meta name="prop:level" content="number" data-doc="The tower's level as the game counts it; 0 hides it.">
<meta name="prop:stats" content="list" data-doc="Its current stats, keyed rows of { id, label, value, next }: value as the game words it (42); next the value after the upgrade in focus (51), shown after an arrow, empty none.">
<meta name="prop:paths" content="list" data-doc="Upgrade paths, keyed rows of { id, name, icon, detail, tier, tiers, cost, state, reason }: tier of tiers bought; cost the next tier's price as the game words it (250 gold); state affordable, unaffordable, locked or maxed (data-state and a word; only affordable can be bought); reason why it is locked (Requires tier 3 on another path), empty none.">
<meta name="prop:sellText" content="string" data-doc="What selling returns, as the game words it (+180 gold), on the Sell button.">
<meta name="prop:canSell" content="bool" data-doc="The game allows selling now; false disables Sell.">
<meta name="fixture:default" content='{"name":"Arcane Spire","icon":"bolt","portrait":0,"level":3,"canSell":true,"sellText":"+180 gold","stats":[{"id":"dmg","label":"Damage","value":"42","next":"51"},{"id":"range","label":"Range","value":"6.5","next":""},{"id":"rate","label":"Attacks/s","value":"1.2","next":"1.4"}],"paths":[{"id":"focus","name":"Focusing Lens","icon":"gem","detail":"More damage to one target","tier":2,"tiers":4,"cost":"250 gold","state":"affordable","reason":""},{"id":"chain","name":"Chain Lightning","icon":"bolt","detail":"Arcs to nearby enemies","tier":1,"tiers":4,"cost":"420 gold","state":"unaffordable","reason":""},{"id":"frost","name":"Frost Ward","icon":"drop","detail":"Slows what it hits","tier":0,"tiers":4,"cost":"300 gold","state":"locked","reason":"Choose two paths at most"}]}'>
<meta name="fixture:maxed" content='{"name":"Cannon","icon":"bomb","portrait":0,"level":8,"canSell":true,"sellText":"+1,240 gold","stats":[{"id":"dmg","label":"Damage","value":"310","next":""},{"id":"splash","label":"Splash","value":"2.5","next":""}],"paths":[{"id":"big","name":"Big Bertha","icon":"bomb","detail":"","tier":4,"tiers":4,"cost":"","state":"maxed","reason":""},{"id":"cluster","name":"Cluster shot","icon":"fire","detail":"","tier":4,"tiers":4,"cost":"","state":"maxed","reason":""}]}'>
<meta name="fixture:disabled" content='{"name":"Watchtower","icon":"flag","portrait":0,"level":1,"canSell":false,"sellText":"+40 gold","stats":[{"id":"dmg","label":"Damage","value":"8","next":""}],"paths":[{"id":"a","name":"Longbows","icon":"bolt","detail":"","tier":0,"tiers":3,"cost":"90 gold","state":"unaffordable","reason":""},{"id":"b","name":"Hot oil","icon":"fire","detail":"","tier":0,"tiers":3,"cost":"120 gold","state":"locked","reason":"Build a Forge first"}]}'>
<meta name="fixture:empty" content='{"name":"Wall","icon":"shield","portrait":0,"level":0,"canSell":true,"sellText":"+5 gold","stats":[],"paths":[]}'>
<meta name="fixture:overflow" content='{"name":"Grand Celestial Observatory of the Seventh Astral Concord","icon":"star","portrait":0,"level":128,"canSell":true,"sellText":"+1,284,000,000 gold","stats":[{"id":"dmg","label":"Damage per celestial alignment","value":"1,240,500","next":"1,860,750"},{"id":"range","label":"Range","value":"14.0","next":""},{"id":"rate","label":"Attacks/s","value":"0.25","next":"0.3"},{"id":"crit","label":"Critical chance","value":"35%","next":"40%"},{"id":"pierce","label":"Pierce","value":"12","next":""}],"paths":[{"id":"p1","name":"Starfall Convergence of the Eternal Night","icon":"star","detail":"Every fifth shot calls down a meteor that splits into lesser stars","tier":7,"tiers":10,"cost":"12,800,000 gold","state":"affordable","reason":""},{"id":"p2","name":"Nebula","icon":"drop","detail":"Clouds that slow","tier":3,"tiers":10,"cost":"900,000,000 gold","state":"unaffordable","reason":""},{"id":"p3","name":"Gravity Well","icon":"gem","detail":"","tier":0,"tiers":10,"cost":"5,000,000 gold","state":"locked","reason":"Requires Starfall tier 8 and the Astral Library on the eastern isle"},{"id":"p4","name":"Lens","icon":"gem","detail":"","tier":10,"tiers":10,"cost":"","state":"maxed","reason":""}]}'>

<!--
  A tower's inspector: its picture, name and level, its current stats with
  the value after the next upgrade where the game gives one, the upgrade
  paths (tier of tiers, the next tier's cost, a state word: affordable,
  too costly, locked with the reason, maxed) and Sell with what it returns.
  Reports upgrade (the path's key; only an affordable path is enabled) and
  sell. The game owns prices, refunds, stats and what is unlocked.
-->
<section id="StrategyTower" part="StrategyTower.frame" data-layout="shell" data-gap="loose" aria-label={name != '' ? name : t("strategy.tower.name")}>
  <header part="StrategyTower.band" data-layout="row">
    <div part="StrategyTower.portrait StrategyTower.picture" class="picture" data-layout="ratio">{#if portrait}<img class="art" src={portrait} alt="">{:else}<og-icon class="mark" icon={icon}></og-icon>{/if}</div>
    <h2 part="StrategyTower.title" data-fill>{name != '' ? name : t("strategy.tower.name")}</h2>
    {#if level > 0}<span part="StrategyTower.badge" class="level">{t("strategy.tower.level", { level: level })}</span>{/if}
  </header>
  {#if stats.length}
  <div class="stats" part="StrategyTower.well" data-layout="stack" data-gap="tight" role="list" aria-label={t("strategy.tower.stats")}>
    {#each stats as s (s.id)}
      <div part="StrategyTower.row" data-layout="row" role="listitem" data-next={!!s.next}><span class="label" part="StrategyTower.caption" data-fill>{s.label}</span><span class="value" part="StrategyTower.figure">{s.value}</span>{#if s.next}<span class="next" part="StrategyTower.figure" aria-label={t("strategy.tower.nextName", { next: s.next })}>{t("strategy.tower.next", { next: s.next })}</span>{/if}</div>
    {/each}
  </div>
  {/if}
  {#if paths.length}
    <div class="paths" data-layout="scroll" role="list" aria-label={t("strategy.tower.paths")}>
      {#each paths as p (p.id)}
        <div part="StrategyTower.path StrategyTower.card" data-layout="row" role="listitem" aria-label={p.name} data-state={p.state}>
          <og-icon class="path-mark" icon={p.icon}></og-icon>
          <div class="path-main" data-layout="stack" data-gap="hair" data-fill>
            <span part="StrategyTower.name StrategyTower.headline" class="path-name">{p.name}</span>
            {#if p.detail}<span class="detail" part="StrategyTower.caption">{p.detail}</span>{/if}
            <div class="pips" data-layout="row" data-gap="tight" role="meter" aria-label={t("strategy.tower.tierName", { tier: p.tier, tiers: p.tiers })} aria-valuemin="0" aria-valuemax={p.tiers} aria-valuenow={p.tier}><span class="tier" part="StrategyTower.figure">{t("strategy.tower.tier", { tier: p.tier, tiers: p.tiers })}</span><div class="track" part="StrategyTower.track" data-fill style:--fill={p.tiers > 0 ? p.tier / p.tiers : 0}><div class="fill" part="StrategyTower.fill"></div></div></div>
            {#if p.state == 'locked' && p.reason}<span part="StrategyTower.reason StrategyTower.caption" class="reason">{p.reason}</span>{/if}
          </div>
          <button class="buy" variant={p.state == 'affordable' ? 'primary' : 'secondary'} disabled={p.state != 'affordable'} aria-label={t("strategy.tower.upgrade", { name: p.name })} aria-description={p.state == 'locked' ? p.reason : p.cost} data-state={p.state} onclick={upgrade}><div class="buy-face" data-layout="stack" data-gap="hair">
            {#if p.state != 'affordable'}<span part="StrategyTower.badge" class="state" data-state={p.state}>{p.state == 'maxed' ? t("strategy.tower.maxed") : (p.state == 'locked' ? t("strategy.tower.locked") : t("strategy.tower.short"))}</span>{/if}
            {#if p.state != 'maxed' && p.cost}<span part="StrategyTower.cost StrategyTower.figure" class="cost" data-state={p.state}>{p.cost}</span>{/if}
          </div></button>
        </div>
      {/each}
    </div>
  {:else}
    <p class="empty-text" part="StrategyTower.caption" role="status">{t("strategy.tower.empty")}</p>
  {/if}
  <div class="actions" data-layout="row">
    <button id="Sell" variant="ghost" disabled={!canSell} onclick={sell}><span>{t("strategy.tower.sell")}</span>{#if sellText != ''}<span class="refund">{sellText}</span>{/if}</button>
  </div>
</section>

<style>
:host { inline-size: stretch; block-size: auto; min-inline-size: 0; max-block-size: stretch; min-block-size: 0; }
[part="StrategyTower.band"] { width: stretch; flex: none; }
[part="StrategyTower.portrait StrategyTower.picture"] { flex: none; width: var(--size-lg); height: var(--size-lg); overflow: hidden; }
.art { width: stretch; height: stretch; object-fit: cover; }
.mark { width: 70%; height: 70%; }
[part="StrategyTower.title"] { margin: 0; flex: 1; min-width: 0; white-space: nowrap; text-overflow: -og-scroll var(--motion-duration-orbit) var(--motion-duration-turn); overflow: hidden; }
.level { flex: none; white-space: nowrap; padding: 0 var(--space-xs); }
.stats { padding: var(--space-sm); }
[part="StrategyTower.row"] { width: stretch; align-items: baseline; }
.label { min-width: 0; white-space: nowrap; text-overflow: -og-scroll var(--motion-duration-orbit) var(--motion-duration-turn); overflow: hidden; }
.value, .next { flex: none; white-space: nowrap; }
.paths { min-block-size: 0; }
[part~="StrategyTower.path"] { padding: var(--space-sm); }
.path-mark { flex: none; width: var(--size-md); height: var(--size-md); }
.path-main { min-width: 0; overflow: hidden; }
.path-name { min-width: 0; white-space: nowrap; text-overflow: -og-scroll var(--motion-duration-orbit) var(--motion-duration-turn); overflow: hidden; }
.detail { overflow-wrap: anywhere; }
/* A reason wraps: it is the one line that says why, so it is never cut. */
.reason { white-space: normal; }
.pips { width: stretch; }
.tier { flex: none; white-space: nowrap; }
.track { position: relative; min-width: 0; block-size: var(--space-xs); overflow: hidden; }
.fill { position: absolute; inset-inline-start: 0; top: 0; bottom: 0; inline-size: calc(var(--fill) * 100%); }
/* The buy plate fits its cost and state word, up to two fifths of the row. */
.buy { flex: none; min-height: var(--size-touch); min-width: calc(var(--size-xxl) * 1.3); max-width: 40%; padding: var(--space-xxs) var(--space-md); }
.buy-face { width: stretch; align-items: center; --figure-ink: currentColor; }
.state, .cost { max-width: stretch; white-space: nowrap; text-overflow: ellipsis; overflow: hidden; }
/* The cost is a badge on the buy plate: on an affordable path the plate is
   the primary, so the cost takes its ink rather than the badge's. */
.state { padding: 0 var(--space-xxs); }
.actions { flex: none; width: stretch; justify-content: flex-end; }
#Sell { min-height: var(--size-touch); gap: var(--space-xs); max-width: stretch; }
.refund { white-space: nowrap; text-overflow: ellipsis; overflow: hidden; min-width: 0; }
.empty-text { margin: 0; padding: var(--space-md); text-align: center; }
/* Smoked glass and lit edges, as the strategy HUD's other plates: the
   tower's portrait in a lit tile, its name large, its stats on a dark
   well, each path a feed row edged by its state, the buy plates and Sell
   on glass, an affordable one lit in the accent. */
:host { padding: calc(12 * var(--font-step-px)) calc(14 * var(--font-step-px)); border: 0; color: var(--hud-ink); background: var(--hud-plate-fill); box-shadow: var(--hud-plate-rim); border-radius: var(--hud-plate-cut) 0; corner-shape: var(--cut-shape); clip-path: border-box; }
[part="StrategyTower.band"] { align-items: center; column-gap: calc(10 * var(--font-step-px)); min-height: 0; padding: 0; border-bottom: 0; background: var(--hud-bare-fill); box-shadow: none; }
[part="StrategyTower.portrait StrategyTower.picture"] { display: flex; align-items: center; justify-content: center; box-sizing: border-box; padding: calc(2 * var(--font-step-px)); background: var(--hud-ready-edge); border-radius: var(--hud-tag-cut) 0; corner-shape: var(--cut-shape); clip-path: border-box; }
.mark { color: var(--hud-glyph); filter: var(--hud-glyph-shadow); }
[part="StrategyTower.title"] { font-family: inherit; font-size: var(--hud-size-unit); line-height: 1.15; font-weight: var(--hud-figure-weight); color: var(--hud-ink); text-shadow: var(--hud-shadow); }
.level, .state { font-size: var(--hud-tag-size); line-height: 1.6; font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); font-variant-numeric: var(--hud-figure-numeric); color: var(--hud-ink); background: var(--hud-tag-fill); box-shadow: var(--hud-tag-rim); border: 0; border-radius: var(--hud-tag-cut) 0; corner-shape: var(--cut-shape); clip-path: border-box; }
.level { color: var(--hud-gold); }
.stats { border-radius: 0; background: var(--hud-tag-fill); box-shadow: var(--hud-tag-rim); }
.label { font: inherit; font-size: var(--hud-label-size); font-weight: var(--hud-label-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); color: var(--hud-label-ink); }
.value, .next { font-family: inherit; font-size: var(--hud-size-sm); font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); font-variant-numeric: var(--hud-figure-numeric); color: var(--hud-ink); }
.next { color: var(--hud-boon); }
[part~="StrategyTower.path"] { column-gap: calc(10 * var(--font-step-px)); border: 0; border-radius: 0; background: var(--hud-feed-fill-start); box-shadow: none; }
.path-mark { color: var(--hud-glyph); }
[data-state=affordable] > .path-mark { color: var(--hud-accent); }
[data-state=unaffordable] > .path-mark { color: var(--hud-harm); }
[data-state=maxed] > .path-mark { color: var(--hud-gold); }
.path-name { font: inherit; font-size: var(--hud-size-name); font-weight: var(--hud-figure-weight); color: var(--hud-ink); }
.detail, .reason { font: inherit; font-size: var(--hud-size-xs); line-height: 1.3; color: var(--hud-ink-dim); }
.reason { font-weight: var(--hud-label-weight); color: var(--hud-harm); }
.tier { font-family: inherit; font-size: var(--hud-tag-size); font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); font-variant-numeric: var(--hud-figure-numeric); color: var(--hud-ink-dim); }
.track { border-radius: 0; transform: skewX(var(--bar-skew)); background: var(--bar-well); box-shadow: var(--bar-rim); }
.fill { background: var(--hud-accent-fill); }
[data-state=maxed] .fill { background: var(--hud-stagger-fill); }
.buy, #Sell { color: var(--hud-ink); }
.buy::part(base), #Sell::part(base) { border: 0; background: var(--hud-plate-fill); box-shadow: var(--hud-plate-rim); border-radius: var(--hud-plate-cut) 0; corner-shape: var(--cut-shape); clip-path: border-box; }
.buy[data-state=affordable]::part(base) { box-shadow: var(--hud-plate-lit-rim); }
.cost { font-family: inherit; font-size: var(--hud-size-sm); font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); font-variant-numeric: var(--hud-figure-numeric); color: var(--hud-ink); }
.cost[data-state=unaffordable] { color: var(--hud-harm); }
.state[data-state=maxed] { color: var(--hud-gold); }
.state[data-state=unaffordable], .state[data-state=locked] { color: var(--hud-harm); }
#Sell { font-size: var(--hud-label-size); font-weight: var(--hud-figure-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); }
.refund { color: var(--hud-gold); letter-spacing: normal; text-transform: none; }
.empty-text { font-size: var(--hud-label-size); font-weight: var(--hud-label-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); color: var(--hud-label-ink); }
@container (width < 600px) {
  [part~="StrategyTower.path"] { display: grid; grid-template-columns: auto minmax(0, 1fr); padding: var(--space-sm); }
  .buy { grid-column: 2; min-width: 0; max-width: none; padding: var(--space-xs) var(--space-sm); }
  .actions { justify-content: stretch; }
  #Sell { width: stretch; }
}
</style>
