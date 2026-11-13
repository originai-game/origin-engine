<meta name="component" content="IdleGenerators">
<meta name="strings" content="./idle.en.ogt">
<meta name="parts" content="IdleGenerators.frame IdleGenerators.band IdleGenerators.title IdleGenerators.row IdleGenerators.name IdleGenerators.figure IdleGenerators.rate IdleGenerators.meter IdleGenerators.badge">
<meta name="prop:title" content="string" data-doc="The list's heading; empty takes the group's (Generators).">
<meta name="prop:generators" content="list" data-doc="The producers in the game's order, keyed rows of { id, icon, name, owned, rate, cycle, cycleText, level, boost, auto, locked }: owned how many the player has (a number, compacted; 0 hides it); rate what it yields as the game words it (+1.2K/s; on a locked row what unlocks it); cycle the running cycle 0 to 1 (negative hides the bar); cycleText the time left as the game words it (empty shows the percent); level its level (0 hides it); boost a multiplier as the game words it (×2; empty hides it); auto it runs by itself (named Auto); locked not yet available (named Locked, disabled).">
<meta name="prop:selected" content="string" data-doc="The generator the player last chose, selected; empty is none.">
<meta name="fixture:default" content='{"title":"","generators":[{"id":"farm","icon":"leaf","name":"Farm","owned":25,"rate":"+48/s","cycle":0.62,"cycleText":"0.4s","level":3,"boost":"","auto":true,"locked":false},{"id":"mine","icon":"gem","name":"Gem Mine","owned":4,"rate":"+1.2K/s","cycle":0.18,"cycleText":"8.2s","level":1,"boost":"×2","auto":false,"locked":false},{"id":"forge","icon":"fire","name":"Forge","owned":0,"rate":"+25K/s","cycle":-1,"cycleText":"","level":0,"boost":"","auto":false,"locked":true}],"selected":"mine"}'>
<meta name="fixture:empty" content='{"title":"","generators":[],"selected":""}'>
<meta name="fixture:disabled" content='{"title":"","generators":[{"id":"forge","icon":"fire","name":"Forge","owned":0,"rate":"Reach 1M gold","cycle":-1,"cycleText":"","level":0,"boost":"","auto":false,"locked":true},{"id":"tower","icon":"crown","name":"Mage Tower","owned":0,"rate":"Reach 1B gold","cycle":-1,"cycleText":"","level":0,"boost":"","auto":false,"locked":true}],"selected":""}'>
<meta name="fixture:selected" content='{"title":"","generators":[{"id":"farm","icon":"leaf","name":"Farm","owned":25,"rate":"+48/s","cycle":1,"cycleText":"","level":3,"boost":"","auto":false,"locked":false},{"id":"mine","icon":"gem","name":"Gem Mine","owned":4,"rate":"+1.2K/s","cycle":0,"cycleText":"","level":1,"boost":"","auto":false,"locked":false}],"selected":"farm"}'>
<meta name="fixture:overflow" content='{"title":"Generators of the Sprawling Clockwork Empire","generators":[{"id":"g1","icon":"leaf","name":"Overgrown Terraced Hydroponic Garden Complex","owned":1.25e15,"rate":"+4.56 quintillion per second","cycle":0.5,"cycleText":"1h 22m 10s","level":999,"boost":"×1,024","auto":true,"locked":false},{"id":"g2","icon":"gem","name":"Mine","owned":12,"rate":"+1.2K/s","cycle":0.1,"cycleText":"3s","level":2,"boost":"","auto":false,"locked":false},{"id":"g3","icon":"fire","name":"Forge","owned":3,"rate":"+30K/s","cycle":0.9,"cycleText":"0.1s","level":1,"boost":"×3","auto":true,"locked":false},{"id":"g4","icon":"bolt","name":"Dynamo","owned":1,"rate":"+400K/s","cycle":0.33,"cycleText":"20s","level":1,"boost":"","auto":false,"locked":false},{"id":"g5","icon":"crown","name":"Mage Tower","owned":0,"rate":"+9M/s","cycle":-1,"cycleText":"","level":0,"boost":"","auto":false,"locked":true},{"id":"g6","icon":"star","name":"Star Engine","owned":0,"rate":"+1B/s","cycle":-1,"cycleText":"","level":0,"boost":"","auto":false,"locked":true},{"id":"g7","icon":"skull","name":"Necropolis","owned":0,"rate":"+80B/s","cycle":-1,"cycleText":"","level":0,"boost":"","auto":false,"locked":true}],"selected":"g1"}'>

<!--
  The generator list of an idle game: one row per producer with its icon,
  name, how many are owned, its yield, its level and boost, Auto when it
  runs by itself, and the running cycle as a ProgressBar. The game runs the
  cycles and owns every number; the list reports choose (a row's key). A
  locked row is disabled and says Locked. It fills its height and scrolls.
-->
<section id="IdleGenerators" part="IdleGenerators.frame" aria-label={title != '' ? title : t("idle.generators.name")}>
  <header part="IdleGenerators.band"><h2 part="IdleGenerators.title">{title != '' ? title : t("idle.generators.name")}</h2></header>
  {#if generators.length}
    <scroll class="rows-scroll">
      <div class="rows" role="listbox" aria-label={title != '' ? title : t("idle.generators.name")}>
        {#each generators as g (g.id)}
          <button part="IdleGenerators.row" role="option" aria-selected={g.id == selected} aria-label={g.name} aria-disabled={g.locked} disabled={g.locked} data-locked={g.locked} data-auto={g.auto} data-boosted={g.boost != ''} onclick={choose}><div class="face">
            <div class="art"><og-icon class="icon" icon={g.locked ? 'lock' : g.icon}></og-icon></div>
            <div class="body">
              <div class="line">
                <span part="IdleGenerators.name" class="name">{g.name}</span>
                <span part="IdleGenerators.figure" hidden={g.locked || !(g.owned > 0)}>{t("idle.generators.owned", { owned: formatCompact(g.owned) })}</span>
              </div>
              <div class="line">
                <span part="IdleGenerators.rate" class="rate">{g.rate}</span>
                <div class="badges">
                  <span part="IdleGenerators.badge" data-tag="level" hidden={g.locked || !(g.level > 0)}>{t("idle.generators.level", { level: formatNumber(g.level) })}</span>
                  <span part="IdleGenerators.badge" data-tag="boost" hidden={g.locked || g.boost == ''}>{g.boost}</span>
                  <span part="IdleGenerators.badge" data-tag="auto" hidden={g.locked || !g.auto}>{t("idle.generators.auto")}</span>
                  <span part="IdleGenerators.badge" data-tag="locked" hidden={!g.locked}>{t("idle.generators.locked")}</span>
                </div>
              </div>
              <div part="IdleGenerators.meter" role="group" aria-label={t("idle.generators.cycle")} hidden={g.locked || g.cycle < 0}><ProgressBar value={g.cycle} buffer={0} label="" valueText={g.cycleText} showValue={true} indeterminate={false} tone="neutral" /></div>
            </div>
          </div></button>
        {/each}
      </div>
    </scroll>
  {:else}
    <p class="empty-text" role="status">{t("idle.generators.empty")}</p>
  {/if}
</section>

<style>
[part="IdleGenerators.frame"][hidden], [part="IdleGenerators.frame"] [hidden] { display: none; }
[part="IdleGenerators.frame"] { display: flex; flex-direction: column; width: stretch; height: stretch; gap: var(--space-sm); min-height: 0; container-type: inline-size; }
[part="IdleGenerators.band"] { display: flex; width: stretch; align-items: center; flex: none; }
[part="IdleGenerators.title"] { margin: 0; white-space: nowrap; text-overflow: ellipsis; overflow: hidden; }
.rows-scroll { width: stretch; flex: 1; min-height: 0; }
.rows { display: flex; flex-direction: column; width: stretch; gap: var(--space-xs); padding: var(--space-xxs); }
[part="IdleGenerators.row"] { width: stretch; min-height: var(--size-touch); padding: var(--space-xs) var(--space-sm); }
.face { display: flex; width: stretch; align-items: center; gap: var(--space-sm); }
.art { display: flex; flex: none; width: var(--size-lg); height: var(--size-lg); align-items: center; justify-content: center; }
.icon { width: var(--size-md); height: var(--size-md); }
.body { display: flex; flex-direction: column; flex: 1; min-width: 0; gap: var(--space-xxs); }
.line { display: flex; width: stretch; align-items: center; gap: var(--space-sm); min-width: 0; }
[part="IdleGenerators.name"] { flex: 1; min-width: 0; text-align: start; white-space: nowrap; text-overflow: ellipsis; overflow: hidden; }
[part="IdleGenerators.figure"] { flex: none; white-space: nowrap; font-variant-numeric: tabular-nums; }
[part="IdleGenerators.rate"] { flex: 1; min-width: 0; text-align: start; white-space: nowrap; text-overflow: ellipsis; overflow: hidden; font-variant-numeric: tabular-nums; }
.badges { display: flex; flex: none; gap: var(--space-xxs); }
[part="IdleGenerators.badge"] { white-space: nowrap; padding: 0 var(--space-xs); }
[part="IdleGenerators.meter"] { width: stretch; }
.empty-text { margin: 0; flex: 1; text-align: center; }
/* A narrow panel (a phone, or a side column on a wide screen) tightens the
   rows and wraps each line's figures under its name. */
@container (width < 560px) {
  [part="IdleGenerators.row"] { padding: var(--space-xxs) var(--space-xs); }
  .art { width: var(--size-md); height: var(--size-md); }
  .icon { width: var(--size-sm); height: var(--size-sm); }
  .line { flex-wrap: wrap; gap: 0 var(--space-xs); }
}
</style>
