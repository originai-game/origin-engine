<meta name="component" content="StrategyTower">
<meta name="strings" content="./strategy.en.ogt">
<meta name="parts" content="StrategyTower.frame StrategyTower.band StrategyTower.title StrategyTower.badge StrategyTower.portrait StrategyTower.row StrategyTower.path StrategyTower.name StrategyTower.cost StrategyTower.reason">
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
<section id="StrategyTower" part="StrategyTower.frame" aria-label={name != '' ? name : t("strategy.tower.name")}>
  <header part="StrategyTower.band">
    <div part="StrategyTower.portrait" class="picture"><img class="art" src={portrait} alt="" hidden={!portrait}><og-icon class="mark" icon={icon} hidden={!!portrait}></og-icon></div>
    <h2 part="StrategyTower.title">{name != '' ? name : t("strategy.tower.name")}</h2>
    <span part="StrategyTower.badge" class="level" hidden={!(level > 0)}>{t("strategy.tower.level", { level: level })}</span>
  </header>
  <div class="stats" role="list" aria-label={t("strategy.tower.stats")} hidden={!stats.length}>
    {#each stats as s (s.id)}
      <div part="StrategyTower.row" role="listitem" data-next={s.next != ''}><span class="label">{s.label}</span><span class="value">{s.value}</span><span class="next" hidden={s.next == ''} aria-label={t("strategy.tower.nextName", { next: s.next })}>{t("strategy.tower.next", { next: s.next })}</span></div>
    {/each}
  </div>
  {#if paths.length}
    <div class="paths" role="list" aria-label={t("strategy.tower.paths")}>
      {#each paths as p (p.id)}
        <div part="StrategyTower.path" role="listitem" aria-label={p.name} data-state={p.state}>
          <og-icon class="path-mark" icon={p.icon}></og-icon>
          <div class="path-main">
            <span part="StrategyTower.name" class="path-name">{p.name}</span>
            <span class="detail" hidden={p.detail == ''}>{p.detail}</span>
            <div class="pips" role="meter" aria-label={t("strategy.tower.tierName", { tier: p.tier, tiers: p.tiers })} aria-valuemin="0" aria-valuemax={p.tiers} aria-valuenow={p.tier}><span class="tier">{t("strategy.tower.tier", { tier: p.tier, tiers: p.tiers })}</span><div class="track" style:--fill={p.tiers > 0 ? p.tier / p.tiers : 0}><div class="fill"></div></div></div>
            <span part="StrategyTower.reason" class="reason" hidden={p.state != 'locked' || p.reason == ''}>{p.reason}</span>
          </div>
          <button class="buy" variant={p.state == 'affordable' ? 'primary' : 'secondary'} disabled={p.state != 'affordable'} aria-label={t("strategy.tower.upgrade", { name: p.name })} aria-description={p.state == 'locked' ? p.reason : p.cost} data-state={p.state} onclick={upgrade}><div class="buy-face">
            <span part="StrategyTower.badge" class="state" data-state={p.state} hidden={p.state == 'affordable'}>{p.state == 'maxed' ? t("strategy.tower.maxed") : (p.state == 'locked' ? t("strategy.tower.locked") : t("strategy.tower.short"))}</span>
            <span part="StrategyTower.cost" class="cost" data-state={p.state} hidden={p.state == 'maxed' || p.cost == ''}>{p.cost}</span>
          </div></button>
        </div>
      {/each}
    </div>
  {:else}
    <p class="empty-text" role="status">{t("strategy.tower.empty")}</p>
  {/if}
  <div class="actions">
    <button id="Sell" variant="ghost" disabled={!canSell} onclick={sell}><span>{t("strategy.tower.sell")}</span><span class="refund" hidden={sellText == ''}>{sellText}</span></button>
  </div>
</section>

<style>
[part="StrategyTower.frame"][hidden], [part="StrategyTower.frame"] [hidden] { display: none; }
[part="StrategyTower.frame"] { display: flex; flex-direction: column; width: stretch; min-width: 0; gap: var(--space-sm); container-type: inline-size; }
[part="StrategyTower.band"] { display: flex; width: stretch; align-items: center; gap: var(--space-sm); flex: none; }
[part="StrategyTower.portrait"] { flex: none; width: var(--size-lg); height: var(--size-lg); align-items: center; justify-content: center; overflow: hidden; }
.art { width: stretch; height: stretch; object-fit: cover; }
.mark { width: 70%; height: 70%; }
[part="StrategyTower.title"] { margin: 0; flex: 1; min-width: 0; white-space: nowrap; text-overflow: ellipsis; overflow: hidden; }
.level { flex: none; white-space: nowrap; padding: 0 var(--space-xs); }
.stats { display: flex; flex-direction: column; width: stretch; gap: var(--space-xxs); }
[part="StrategyTower.row"] { display: flex; width: stretch; align-items: baseline; gap: var(--space-sm); }
.label { flex: 1; min-width: 0; white-space: nowrap; text-overflow: ellipsis; overflow: hidden; }
.value, .next { flex: none; white-space: nowrap; font-variant-numeric: tabular-nums; }
.paths { display: flex; flex-direction: column; width: stretch; gap: var(--space-xs); }
[part="StrategyTower.path"] { display: flex; width: stretch; align-items: center; gap: var(--space-sm); padding: var(--space-xs) var(--space-sm); }
.path-mark { flex: none; width: var(--size-md); height: var(--size-md); }
.path-main { display: flex; flex-direction: column; flex: 1; min-width: 0; overflow: hidden; gap: var(--space-xxs); }
.path-name, .detail { width: stretch; white-space: nowrap; text-overflow: ellipsis; overflow: hidden; }
/* A reason wraps: it is the one line that says why, so it is never cut. */
.reason { width: stretch; min-width: 0; white-space: normal; }
.pips { display: flex; width: stretch; align-items: center; gap: var(--space-xs); }
.tier { flex: none; white-space: nowrap; font-variant-numeric: tabular-nums; }
.track { flex: 1; min-width: 0; height: var(--space-xs); overflow: hidden; }
.fill { position: absolute; inset-inline-start: 0; top: 0; width: calc(var(--fill) * 100%); height: stretch; }
/* The buy plate fits its cost and state word, up to two fifths of the row. */
.buy { flex: none; min-height: var(--size-touch); min-width: calc(var(--size-xxl) * 1.3); max-width: 40%; padding: var(--space-xxs) var(--space-md); }
.buy-face { display: flex; flex-direction: column; width: stretch; align-items: center; gap: 0; }
.state, .cost { max-width: stretch; white-space: nowrap; text-overflow: ellipsis; overflow: hidden; }
.state { padding: 0 var(--space-xxs); }
.actions { display: flex; width: stretch; justify-content: flex-end; }
#Sell { min-height: var(--size-touch); gap: var(--space-xs); max-width: stretch; }
.refund { white-space: nowrap; text-overflow: ellipsis; overflow: hidden; min-width: 0; }
.empty-text { margin: 0; }
@container (width < 600px) {
  [part="StrategyTower.path"] { padding: var(--space-xxs) var(--space-xs); gap: var(--space-xs); }
  .buy { min-width: var(--size-xxl); max-width: 45%; padding: var(--space-xxs) var(--space-sm); }
  .actions { justify-content: stretch; }
  #Sell { width: stretch; }
}
</style>
