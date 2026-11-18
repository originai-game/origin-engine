<meta name="component" content="StrategySelection">
<meta name="strings" content="./strategy.en.ogt">
<meta name="parts" content="StrategySelection.frame StrategySelection.portrait StrategySelection.health StrategySelection.fill StrategySelection.vital StrategySelection.name StrategySelection.count StrategySelection.line StrategySelection.status StrategySelection.stat StrategySelection.roster StrategySelection.cell StrategySelection.art StrategySelection.meter StrategySelection.badge StrategySelection.more">
<meta name="prop:name" content="string" data-doc="The selected unit's or building's name, or the focused subgroup's (Men-at-Arms); empty shows Nothing selected.">
<meta name="prop:count" content="number" data-doc="How many of it are selected (×12, shown above 1).">
<meta name="prop:kind" content="string" data-doc="unit or building (data-kind on the frame).">
<meta name="prop:line" content="string" data-doc="Its class line as the game words it (Heavy infantry · Veteran II), under the name; empty none.">
<meta name="prop:status" content="string" data-doc="What it is doing as the game words it (Holding, Under construction), on a tag before the stats; empty none.">
<meta name="prop:portrait" content="number" data-doc="Its portrait (og.load.texture) in a tile with a lit edge; 0 draws icon instead.">
<meta name="prop:icon" content="string" data-doc="An icon of the atlas drawn when portrait is 0.">
<meta name="prop:health" content="number" data-doc="Its health now (the subgroup's together), under the portrait as a bar and in figures.">
<meta name="prop:healthMax" content="number" data-default="1" data-doc="Its whole health; 0 hides the bar and the figures.">
<meta name="prop:stats" content="list" data-doc="Its stats on tags, keyed rows of { id, label, value }: label short as the game words it (ATK), value its figure (12).">
<meta name="prop:units" content="list" data-doc="Each unit or subgroup selected, as cards in the game's order (shown when there are two or more; two rows, then they scroll), keyed rows of { id, name, kind, portrait, icon, count, health, healthMax }: portrait a texture, 0 draws icon from the atlas; count shown above 1 on a tag; health over healthMax a bar on the card, green, gold when hurt and red when low (data-tone).">
<meta name="prop:focused" content="string" data-doc="The card whose unit or subgroup the panel shows (its edge lit, aria-selected); empty none.">
<meta name="fixture:default" content='{"name":"Men-at-Arms","count":12,"kind":"unit","line":"Heavy infantry · Veteran II","status":"","portrait":0,"icon":"sword","health":1476,"healthMax":1800,"stats":[{"id":"atk","label":"ATK","value":"12"},{"id":"arm","label":"ARM","value":"4"},{"id":"spd","label":"SPD","value":"3.2"}],"focused":"m1","units":[{"id":"m1","name":"Man-at-Arms","kind":"unit","portrait":0,"icon":"sword","count":1,"health":90,"healthMax":100},{"id":"m2","name":"Man-at-Arms","kind":"unit","portrait":0,"icon":"sword","count":1,"health":55,"healthMax":100},{"id":"m3","name":"Man-at-Arms","kind":"unit","portrait":0,"icon":"sword","count":1,"health":100,"healthMax":100},{"id":"m4","name":"Man-at-Arms","kind":"unit","portrait":0,"icon":"sword","count":1,"health":25,"healthMax":100},{"id":"m5","name":"Man-at-Arms","kind":"unit","portrait":0,"icon":"sword","count":1,"health":80,"healthMax":100},{"id":"m6","name":"Man-at-Arms","kind":"unit","portrait":0,"icon":"sword","count":1,"health":100,"healthMax":100},{"id":"m7","name":"Man-at-Arms","kind":"unit","portrait":0,"icon":"sword","count":1,"health":100,"healthMax":100},{"id":"a1","name":"Archer","kind":"unit","portrait":0,"icon":"bolt","count":1,"health":70,"healthMax":100},{"id":"a2","name":"Archer","kind":"unit","portrait":0,"icon":"bolt","count":1,"health":100,"healthMax":100},{"id":"a3","name":"Archer","kind":"unit","portrait":0,"icon":"bolt","count":1,"health":45,"healthMax":100},{"id":"a4","name":"Archer","kind":"unit","portrait":0,"icon":"bolt","count":1,"health":100,"healthMax":100},{"id":"a5","name":"Archer","kind":"unit","portrait":0,"icon":"bolt","count":1,"health":95,"healthMax":100}]}'>
<meta name="fixture:single" content='{"name":"Stone Keep","count":1,"kind":"building","line":"Town centre","status":"Training","portrait":0,"icon":"home","health":1850,"healthMax":2400,"stats":[{"id":"arm","label":"ARM","value":"18"},{"id":"gar","label":"GARRISON","value":"10"}],"focused":"","units":[]}'>
<meta name="fixture:empty" content='{"name":"","count":0,"kind":"unit","line":"","status":"","portrait":0,"icon":"","health":0,"healthMax":0,"stats":[],"focused":"","units":[]}'>
<meta name="fixture:groups" content='{"name":"Spearmen","count":6,"kind":"unit","line":"Light infantry","status":"Holding","portrait":0,"icon":"sword","health":340,"healthMax":420,"stats":[{"id":"atk","label":"ATK","value":"6"},{"id":"arm","label":"ARM","value":"1"}],"focused":"spear","units":[{"id":"spear","name":"Spearmen","kind":"unit","portrait":0,"icon":"sword","count":6,"health":340,"healthMax":420},{"id":"archer","name":"Archers","kind":"unit","portrait":0,"icon":"bolt","count":2,"health":150,"healthMax":160},{"id":"captain","name":"Captain Ilse","kind":"unit","portrait":0,"icon":"crown","count":1,"health":96,"healthMax":300}]}'>
<meta name="fixture:dense" content='{"name":"Men-at-Arms","count":40,"kind":"unit","line":"Heavy infantry","status":"","portrait":0,"icon":"sword","health":3610,"healthMax":4000,"stats":[{"id":"atk","label":"ATK","value":"12"},{"id":"arm","label":"ARM","value":"4"}],"focused":"u1","units":[{"id":"u1","name":"Man-at-Arms","kind":"unit","portrait":0,"icon":"sword","count":1,"health":100,"healthMax":100},{"id":"u2","name":"Man-at-Arms","kind":"unit","portrait":0,"icon":"sword","count":1,"health":90,"healthMax":100},{"id":"u3","name":"Man-at-Arms","kind":"unit","portrait":0,"icon":"sword","count":1,"health":80,"healthMax":100},{"id":"u4","name":"Man-at-Arms","kind":"unit","portrait":0,"icon":"sword","count":1,"health":70,"healthMax":100},{"id":"u5","name":"Man-at-Arms","kind":"unit","portrait":0,"icon":"sword","count":1,"health":60,"healthMax":100},{"id":"u6","name":"Man-at-Arms","kind":"unit","portrait":0,"icon":"sword","count":1,"health":50,"healthMax":100},{"id":"u7","name":"Man-at-Arms","kind":"unit","portrait":0,"icon":"sword","count":1,"health":40,"healthMax":100},{"id":"u8","name":"Man-at-Arms","kind":"unit","portrait":0,"icon":"sword","count":1,"health":30,"healthMax":100},{"id":"u9","name":"Man-at-Arms","kind":"unit","portrait":0,"icon":"sword","count":1,"health":20,"healthMax":100},{"id":"u10","name":"Man-at-Arms","kind":"unit","portrait":0,"icon":"sword","count":1,"health":10,"healthMax":100},{"id":"u11","name":"Man-at-Arms","kind":"unit","portrait":0,"icon":"sword","count":1,"health":100,"healthMax":100},{"id":"u12","name":"Man-at-Arms","kind":"unit","portrait":0,"icon":"sword","count":1,"health":100,"healthMax":100},{"id":"u13","name":"Man-at-Arms","kind":"unit","portrait":0,"icon":"sword","count":1,"health":100,"healthMax":100},{"id":"u14","name":"Man-at-Arms","kind":"unit","portrait":0,"icon":"sword","count":1,"health":100,"healthMax":100},{"id":"u15","name":"Man-at-Arms","kind":"unit","portrait":0,"icon":"sword","count":1,"health":100,"healthMax":100},{"id":"u16","name":"Man-at-Arms","kind":"unit","portrait":0,"icon":"sword","count":1,"health":100,"healthMax":100},{"id":"u17","name":"Man-at-Arms","kind":"unit","portrait":0,"icon":"sword","count":1,"health":100,"healthMax":100},{"id":"u18","name":"Man-at-Arms","kind":"unit","portrait":0,"icon":"sword","count":1,"health":100,"healthMax":100},{"id":"u19","name":"Man-at-Arms","kind":"unit","portrait":0,"icon":"sword","count":1,"health":100,"healthMax":100},{"id":"u20","name":"Man-at-Arms","kind":"unit","portrait":0,"icon":"sword","count":1,"health":100,"healthMax":100},{"id":"u21","name":"Man-at-Arms","kind":"unit","portrait":0,"icon":"sword","count":1,"health":100,"healthMax":100},{"id":"u22","name":"Man-at-Arms","kind":"unit","portrait":0,"icon":"sword","count":1,"health":100,"healthMax":100},{"id":"u23","name":"Man-at-Arms","kind":"unit","portrait":0,"icon":"sword","count":1,"health":100,"healthMax":100},{"id":"u24","name":"Man-at-Arms","kind":"unit","portrait":0,"icon":"sword","count":1,"health":100,"healthMax":100},{"id":"u25","name":"Man-at-Arms","kind":"unit","portrait":0,"icon":"sword","count":1,"health":100,"healthMax":100},{"id":"u26","name":"Man-at-Arms","kind":"unit","portrait":0,"icon":"sword","count":1,"health":100,"healthMax":100},{"id":"u27","name":"Man-at-Arms","kind":"unit","portrait":0,"icon":"sword","count":1,"health":100,"healthMax":100},{"id":"u28","name":"Man-at-Arms","kind":"unit","portrait":0,"icon":"sword","count":1,"health":100,"healthMax":100},{"id":"u29","name":"Man-at-Arms","kind":"unit","portrait":0,"icon":"sword","count":1,"health":100,"healthMax":100},{"id":"u30","name":"Man-at-Arms","kind":"unit","portrait":0,"icon":"sword","count":1,"health":100,"healthMax":100},{"id":"u31","name":"Man-at-Arms","kind":"unit","portrait":0,"icon":"sword","count":1,"health":100,"healthMax":100},{"id":"u32","name":"Man-at-Arms","kind":"unit","portrait":0,"icon":"sword","count":1,"health":100,"healthMax":100},{"id":"u33","name":"Man-at-Arms","kind":"unit","portrait":0,"icon":"sword","count":1,"health":100,"healthMax":100},{"id":"u34","name":"Man-at-Arms","kind":"unit","portrait":0,"icon":"sword","count":1,"health":100,"healthMax":100},{"id":"u35","name":"Man-at-Arms","kind":"unit","portrait":0,"icon":"sword","count":1,"health":100,"healthMax":100},{"id":"u36","name":"Man-at-Arms","kind":"unit","portrait":0,"icon":"sword","count":1,"health":100,"healthMax":100},{"id":"u37","name":"Man-at-Arms","kind":"unit","portrait":0,"icon":"sword","count":1,"health":100,"healthMax":100},{"id":"u38","name":"Man-at-Arms","kind":"unit","portrait":0,"icon":"sword","count":1,"health":100,"healthMax":100},{"id":"u39","name":"Man-at-Arms","kind":"unit","portrait":0,"icon":"sword","count":1,"health":100,"healthMax":100},{"id":"u40","name":"Man-at-Arms","kind":"unit","portrait":0,"icon":"sword","count":1,"health":100,"healthMax":100}]}'>
<meta name="fixture:overflow" content='{"name":"Grand Marshal Aurelia von Hohenstaufen-Wittelsbach the Unyielding","count":1284,"kind":"unit","line":"Hero · Commander of the Northern Expeditionary Host","status":"Channelling the Banner of the Old Kingdom","portrait":0,"icon":"crown","health":2,"healthMax":1800000,"stats":[{"id":"atk","label":"ATTACK","value":"1,240,500"},{"id":"arm","label":"ARMOUR","value":"35"},{"id":"spd","label":"SPEED","value":"3.2"},{"id":"rng","label":"RANGE","value":"14"}],"focused":"u7","units":[{"id":"u1","name":"Spearmen","kind":"unit","portrait":0,"icon":"sword","count":12,"health":900,"healthMax":960},{"id":"u2","name":"Archers","kind":"unit","portrait":0,"icon":"bolt","count":8,"health":500,"healthMax":640},{"id":"u3","name":"Knights","kind":"unit","portrait":0,"icon":"shield","count":6,"health":1100,"healthMax":1500},{"id":"u4","name":"Healers","kind":"unit","portrait":0,"icon":"potion","count":4,"health":220,"healthMax":320},{"id":"u5","name":"Scouts","kind":"unit","portrait":0,"icon":"map","count":3,"health":40,"healthMax":300},{"id":"u6","name":"Rams","kind":"unit","portrait":0,"icon":"bomb","count":2,"health":600,"healthMax":1400},{"id":"u7","name":"Grand Marshal Aurelia","kind":"unit","portrait":0,"icon":"crown","count":1,"health":2,"healthMax":1800},{"id":"u8","name":"Crossbowmen","kind":"unit","portrait":0,"icon":"bolt","count":1284,"health":480,"healthMax":480},{"id":"b1","name":"Watchtower","kind":"building","portrait":0,"icon":"flag","count":2,"health":700,"healthMax":900},{"id":"b2","name":"Barracks","kind":"building","portrait":0,"icon":"home","count":1,"health":1500,"healthMax":1500}]}'>

<!--
  The selection panel at the foot: the portrait in a tile with a lit edge,
  its health as a bar and in figures under it; the name large with how
  many are selected, its class line in small capitals, what it is doing
  and its stats on tags; and every unit selected as a card with its health,
  the focused one lit. Reports choose (a card's key: show that unit or
  subgroup). The game owns every figure and what the panel shows.
-->
<section id="StrategySelection" part="StrategySelection.frame" class="hud bare" aria-label={name != '' ? name : t("strategy.selection.title")} data-kind={kind}>
  {#if name != ''}
    <div class="lead">
      <div part="StrategySelection.portrait" class="portrait"><div class="portrait-face">{#if portrait}<img class="art" src={portrait} alt="">{:else}<og-icon class="mark" icon={icon}></og-icon>{/if}</div></div>
      {#if healthMax > 0}
        <div part="StrategySelection.health" class="health" role="meter" aria-label={t("strategy.selection.health")} aria-valuemin="0" aria-valuemax={healthMax} aria-valuenow={health <= 0 ? 0 : health} data-tone={health / healthMax <= 0.3 ? 'low' : (health / healthMax <= 0.6 ? 'hurt' : 'ok')} style:--fill={health <= 0 ? 0 : health >= healthMax ? 1 : health / healthMax}><div part="StrategySelection.fill" class="fill"></div></div>
        <div part="StrategySelection.vital" class="vital"><span class="vital-now">{formatNumber(health)}</span><span class="vital-max">{t("strategy.selection.healthMax", { max: healthMax })}</span></div>
      {/if}
    </div>
    <div class="main">
      <div class="head">
        <div class="titles">
          <div class="name-line"><span part="StrategySelection.name" class="unit-name">{name}</span>{#if count > 1}<span part="StrategySelection.count" class="count">{t("strategy.selection.count", { count: count })}</span>{/if}</div>
          {#if line != ''}<span part="StrategySelection.line" class="line">{line}</span>{/if}
        </div>
        {#if stats.length || status != ''}<div class="stats" role="list" aria-label={t("strategy.selection.stats")}>
          {#if status != ''}<span part="StrategySelection.status" class="status" role="status">{status}</span>{/if}
          {#each stats as s (s.id)}<span part="StrategySelection.stat" class="stat" role="listitem" aria-label={t("strategy.selection.stat", { label: s.label, value: s.value })}>{t("strategy.selection.stat", { label: s.label, value: s.value })}</span>{/each}
        </div>{/if}
      </div>
      {#if units.length > 1}
        <div part="StrategySelection.roster" class="roster" role="listbox" aria-label={t("strategy.selection.units")}>
          {#each units as u, i (u.id)}
            <button part="StrategySelection.cell" class="card" data-past={i >= 4} role="option" aria-selected={u.id == focused} aria-label={u.name} aria-description={t("strategy.selection.unitValue", { count: u.count, now: u.health, max: u.healthMax })} data-kind={u.kind} onclick={choose}><div class="card-face">
              {#if u.portrait}<img part="StrategySelection.art" class="art" src={u.portrait} alt="">{:else}<div part="StrategySelection.art" class="blank-art"><og-icon class="card-mark" icon={u.icon}></og-icon></div>{/if}
              {#if u.count > 1}<span part="StrategySelection.badge" class="stack">{t("strategy.selection.count", { count: u.count })}</span>{/if}
              <div part="StrategySelection.meter" class="meter" data-tone={u.healthMax > 0 && u.health / u.healthMax <= 0.3 ? 'low' : (u.healthMax > 0 && u.health / u.healthMax <= 0.6 ? 'hurt' : 'ok')} style:--fill={u.healthMax <= 0 || u.health <= 0 ? 0 : u.health >= u.healthMax ? 1 : u.health / u.healthMax}><div class="meter-fill"></div></div>
            </div></button>
          {/each}
          {#if units.length > 4}<div part="StrategySelection.more" class="more" role="status" aria-label={t("strategy.selection.more", { n: units.length - 4 })}><span class="more-count">{t("strategy.selection.moreCount", { n: units.length - 4 })}</span></div>{/if}
        </div>
      {/if}
    </div>
  {:else}
    <p class="empty-text" role="status">{t("strategy.selection.empty")}</p>
  {/if}
</section>

<style>
:host { --portrait: calc(var(--size-xl) * 1.35); --card: calc(var(--size-sm) * 1.25); display: flex; align-items: flex-start; column-gap: calc(14 * var(--font-step-px)); inline-size: stretch; min-inline-size: 0; color: var(--hud-ink); }
/* The portrait, its health bar and figures. The portrait is art: it keeps
   its size at any text scale. */
.lead { display: flex; flex-direction: column; align-items: stretch; row-gap: calc(5 * var(--font-step-px)); flex: none; min-inline-size: var(--portrait); }
.portrait { align-self: center; inline-size: var(--portrait); block-size: var(--portrait); padding: calc(2 * var(--font-step-px)); box-sizing: border-box; background: var(--hud-ready-edge); clip-path: polygon(var(--hud-plate-cut) 0, 100% 0, 100% calc(100% - var(--hud-plate-cut)), calc(100% - var(--hud-plate-cut)) 100%, 0 100%, 0 var(--hud-plate-cut)); }
.portrait-face { display: flex; align-items: center; justify-content: center; inline-size: 100%; block-size: 100%; overflow: hidden; background: var(--hud-plate-fill); clip-path: polygon(var(--hud-plate-cut) 0, 100% 0, 100% calc(100% - var(--hud-plate-cut)), calc(100% - var(--hud-plate-cut)) 100%, 0 100%, 0 var(--hud-plate-cut)); }
.portrait-face .art { inline-size: 100%; block-size: 100%; object-fit: cover; }
.mark { inline-size: 55%; block-size: 55%; color: var(--hud-glyph); filter: var(--hud-glyph-shadow); }
.health { position: relative; block-size: calc(7 * var(--font-step-px)); overflow: hidden; background: var(--bar-well); box-shadow: var(--bar-rim); }
.fill { position: absolute; left: 0; top: 0; bottom: 0; inline-size: calc(var(--fill) * 100%); background: var(--hud-unit-fill); }
[data-tone=hurt] > .fill { background: var(--hud-unit-hurt-fill); }
[data-tone=low] > .fill { background: var(--hud-unit-low-fill); }
.vital { display: flex; justify-content: center; align-items: baseline; column-gap: calc(3 * var(--font-step-px)); }
.vital-now { flex: none; font-size: var(--hud-size-xs); font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); font-variant-numeric: var(--hud-figure-numeric); white-space: nowrap; }
.vital-max { flex: none; font-size: var(--hud-size-xs); font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); font-variant-numeric: var(--hud-figure-numeric); white-space: nowrap; color: var(--hud-ink-dim); }
/* The name, its line, its stats and the cards. */
.main { display: flex; flex-direction: column; row-gap: calc(8 * var(--font-step-px)); flex: 1 1 0; min-inline-size: 0; }
.head { display: flex; flex-wrap: wrap; align-items: flex-start; justify-content: space-between; column-gap: calc(12 * var(--font-step-px)); row-gap: calc(4 * var(--font-step-px)); min-inline-size: 0; }
.titles { display: flex; flex-direction: column; row-gap: calc(1 * var(--font-step-px)); flex: 1 1 auto; min-inline-size: 0; }
.name-line { display: flex; align-items: baseline; column-gap: calc(6 * var(--font-step-px)); min-inline-size: 0; }
.unit-name { flex: 0 1 auto; min-inline-size: 0; font-size: var(--hud-size-unit); line-height: 1.15; font-weight: var(--hud-figure-weight); white-space: nowrap; text-overflow: -og-scroll var(--motion-duration-orbit) var(--motion-duration-turn); overflow: hidden; text-shadow: var(--hud-shadow); }
.count { flex: none; font-size: var(--hud-size-sm); font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); font-variant-numeric: var(--hud-figure-numeric); white-space: nowrap; color: var(--hud-ink-dim); }
.line { min-inline-size: 0; font-size: var(--hud-label-size); font-weight: var(--hud-label-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); color: var(--hud-label-ink); white-space: nowrap; text-overflow: -og-scroll var(--motion-duration-orbit) var(--motion-duration-turn); overflow: hidden; }
.stats { display: flex; flex-wrap: wrap; align-items: center; justify-content: flex-end; column-gap: calc(5 * var(--font-step-px)); row-gap: calc(4 * var(--font-step-px)); flex: 0 1 auto; min-inline-size: 0; max-inline-size: 100%; }
.stat, .status { flex: 0 1 auto; min-inline-size: 0; padding: 0 calc(6 * var(--font-step-px)); font-size: var(--hud-tag-size); line-height: 1.6; font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); font-variant-numeric: var(--hud-figure-numeric); white-space: nowrap; text-overflow: -og-scroll var(--motion-duration-orbit) var(--motion-duration-turn); overflow: hidden; background: var(--hud-tag-fill); box-shadow: var(--hud-tag-rim); clip-path: polygon(var(--hud-tag-cut) 0, 100% 0, 100% calc(100% - var(--hud-tag-cut)), calc(100% - var(--hud-tag-cut)) 100%, 0 100%, 0 var(--hud-tag-cut)); }
.status { color: var(--hud-gold); }
/* The cards: two rows at most, then they scroll. Each is art with a lit
   edge and keeps its size at any text scale. */
.roster { display: flex; flex-wrap: wrap; align-content: flex-start; gap: calc(4 * var(--font-step-px)); min-inline-size: 0; max-block-size: calc(var(--card) * 2 + 4 * var(--font-step-px)); overflow-y: auto; }
.card { flex: none; padding: 0; border: 0; border-radius: 0; inline-size: var(--card); block-size: var(--card); min-inline-size: 0; color: var(--hud-ink); }
.card::part(base) { padding: calc(2 * var(--font-step-px)); border: 0; border-radius: 0; background: var(--hud-edge); box-shadow: none; clip-path: polygon(var(--hud-tag-cut) 0, 100% 0, 100% calc(100% - var(--hud-tag-cut)), calc(100% - var(--hud-tag-cut)) 100%, 0 100%, 0 var(--hud-tag-cut)); }
.card[aria-selected=true]::part(base) { background: var(--hud-ready-edge); }
/* The face is sized from the card: an option button's content box gives
   a stretched or percentage child no width. */
.card-face { position: relative; inline-size: calc(var(--card) - 4 * var(--font-step-px)); block-size: calc(var(--card) - 4 * var(--font-step-px)); overflow: hidden; clip-path: polygon(var(--hud-tag-cut) 0, 100% 0, 100% calc(100% - var(--hud-tag-cut)), calc(100% - var(--hud-tag-cut)) 100%, 0 100%, 0 var(--hud-tag-cut)); }
.card-face .art { position: absolute; left: 0; top: 0; inline-size: 100%; block-size: 100%; object-fit: cover; }
.blank-art { position: absolute; left: 0; top: 0; inline-size: 100%; block-size: 100%; display: flex; align-items: center; justify-content: center; background: var(--hud-plate-fill); }
.card-mark { inline-size: 60%; block-size: 60%; color: var(--hud-glyph); }
.stack { position: absolute; right: 0; top: 0; padding: 0 calc(2 * var(--font-step-px)); font-size: var(--hud-size-label-compact); line-height: 1.2; font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); white-space: nowrap; background: var(--hud-tag-fill); }
/* The count of the cards a phone leaves out, a tile of the cards' size;
   a desktop shows every card. */
.more { display: none; flex: none; align-items: center; justify-content: center; inline-size: var(--card); block-size: var(--card); box-sizing: border-box; font-size: var(--hud-size-xs); font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); font-variant-numeric: var(--hud-figure-numeric); white-space: nowrap; color: var(--hud-ink); background: var(--hud-tag-fill); box-shadow: var(--hud-tag-rim); clip-path: polygon(var(--hud-tag-cut) 0, 100% 0, 100% calc(100% - var(--hud-tag-cut)), calc(100% - var(--hud-tag-cut)) 100%, 0 100%, 0 var(--hud-tag-cut)); }
/* A card's health: a thin bar inset at its foot. */
.meter { position: absolute; left: calc(3 * var(--font-step-px)); right: calc(3 * var(--font-step-px)); bottom: calc(3 * var(--font-step-px)); block-size: calc(3 * var(--font-step-px)); overflow: hidden; background: var(--hud-line-well); }
.meter-fill { position: absolute; left: 0; top: 0; bottom: 0; inline-size: calc(var(--fill) * 100%); background: var(--hud-boon); }
[data-tone=hurt] > .meter-fill { background: var(--hud-gold); }
[data-tone=low] > .meter-fill { background: var(--hud-harm); }
.empty-text { flex: 1 1 auto; margin: 0; padding: calc(16 * var(--font-step-px)); text-align: center; font-size: var(--hud-label-size); font-weight: var(--hud-label-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); color: var(--hud-label-ink); }
/* On a phone the panel is compact: a smaller portrait, the name and its
   line, and one row of whole cards: the first four and a +n tile counting
   the rest; the stats go. */
@media (width < 600px) {
  :host { --portrait: var(--size-lg); --card: var(--size-md); column-gap: calc(8 * var(--font-step-px)); }
  .lead { row-gap: calc(3 * var(--font-step-px)); }
  .health { block-size: calc(5 * var(--font-step-px)); }
  .vital-max { display: none; }
  .main { row-gap: calc(5 * var(--font-step-px)); }
  .unit-name { font-size: var(--hud-size-unit-compact); }
  .count { font-size: var(--hud-size-xs); }
  .line { font-size: var(--hud-size-label-compact); letter-spacing: normal; }
  .stats { display: none; }
  .roster { flex-wrap: nowrap; max-block-size: none; overflow: hidden; }
  .card[data-past=true] { display: none; }
  .more { display: flex; }
}
</style>
