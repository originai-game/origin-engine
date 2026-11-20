<meta name="component" content="StrategyGroups">
<meta name="strings" content="./strategy.en.ogt">
<meta name="parts" content="StrategyGroups.frame StrategyGroups.group StrategyGroups.key StrategyGroups.picture StrategyGroups.count">
<meta name="prop:groups" content="list" data-doc="The control groups and quick picks (the idle villagers, the army) in the game's order, keyed rows of { id, label, key, icon, count, active, alert }: label for a reader (Group 1, Idle villagers); key the group's key as the game names it (1), on a key cap; icon a picture (og.load.texture) before the count, 0 none; count what it holds; active the group selected now, lit (aria-pressed); alert one that wants the player (idle villagers waiting), its count and edge in gold (data-alert).">
<meta name="fixture:default" content='{"groups":[{"id":"g1","label":"Group 1","key":"1","icon":0,"count":12,"active":true,"alert":false},{"id":"g2","label":"Group 2","key":"2","icon":0,"count":6,"active":false,"alert":false},{"id":"g3","label":"Group 3","key":"3","icon":0,"count":4,"active":false,"alert":false}]}'>
<meta name="fixture:idle" content='{"groups":[{"id":"idle","label":"Idle villagers","key":".","icon":0,"count":3,"active":false,"alert":true},{"id":"army","label":"Whole army","key":"Q","icon":0,"count":38,"active":false,"alert":false},{"id":"g1","label":"Group 1","key":"1","icon":0,"count":12,"active":false,"alert":false}]}'>
<meta name="fixture:empty" content='{"groups":[]}'>
<meta name="fixture:overflow" content='{"groups":[{"id":"g1","label":"Group 1","key":"1","icon":0,"count":1240,"active":true,"alert":false},{"id":"g2","label":"Group 2","key":"2","icon":0,"count":6,"active":false,"alert":false},{"id":"g3","label":"Group 3","key":"3","icon":0,"count":4,"active":false,"alert":false},{"id":"g4","label":"Group 4","key":"4","icon":0,"count":98,"active":false,"alert":false},{"id":"g5","label":"Group 5","key":"5","icon":0,"count":15,"active":false,"alert":false},{"id":"g6","label":"Group 6","key":"6","icon":0,"count":2,"active":false,"alert":false},{"id":"g7","label":"Group 7","key":"7","icon":0,"count":7,"active":false,"alert":false},{"id":"g8","label":"Group 8","key":"8","icon":0,"count":11,"active":false,"alert":false},{"id":"g9","label":"Group 9","key":"9","icon":0,"count":3,"active":false,"alert":false},{"id":"g0","label":"Group 10","key":"Left Shoulder + 0","icon":0,"count":24,"active":false,"alert":true}]}'>

<!--
  The control groups over the selection: one small plate each with its key
  cap and the units it holds, the group selected now lit; a quick pick (the
  idle villagers) may carry a picture, its count in gold while it wants the
  player. Reports pick (the group's key); the game owns the groups.
-->
<div id="StrategyGroups" part="StrategyGroups.frame" class="hud bare" role="toolbar" aria-label={t("strategy.groups.name")}>
  {#each groups as g (g.id)}
    <button part="StrategyGroups.group" class="group" aria-label={t("strategy.groups.value", { label: g.label, count: g.count })} aria-pressed={g.active} data-alert={g.alert} onclick={pick}><div class="face">
      {#if g.key}<span part="StrategyGroups.key" class="key">{g.key}</span>{/if}
      {#if g.icon}<img part="StrategyGroups.picture" class="picture" src={g.icon} alt="">{/if}
      <span part="StrategyGroups.count" class="count">{formatNumber(g.count)}</span>
    </div></button>
  {/each}
</div>

<style>
:host { display: flex; flex-wrap: wrap; align-items: center; column-gap: calc(6 * var(--font-step-px)); row-gap: calc(4 * var(--font-step-px)); inline-size: fit-content; max-inline-size: stretch; min-inline-size: 0; color: var(--hud-ink); }
/* A small plate per group: smoked glass, lit in the accent when it is
   the group selected, its edge in gold while it wants the player. */
.group { flex: 0 1 auto; min-inline-size: 0; max-inline-size: 100%; color: var(--hud-ink); }
.group::part(base) { padding: calc(3 * var(--font-step-px)) calc(8 * var(--font-step-px)); border: 0; background: var(--hud-plate-fill); box-shadow: var(--hud-plate-rim); border-radius: var(--hud-tag-cut) 0; corner-shape: var(--cut-shape); clip-path: border-box; }
.group[aria-pressed=true]::part(base) { background: var(--hud-plate-fill); box-shadow: var(--hud-plate-lit-rim); }
.group[data-alert=true]::part(base) { box-shadow: var(--hud-step-rim); }
.face { display: flex; align-items: center; column-gap: calc(6 * var(--font-step-px)); min-inline-size: 0; }
.key { flex: 0 1 auto; min-inline-size: calc(18 * var(--font-step-px)); max-inline-size: calc(var(--size-xl) * 1.2); padding: 0 calc(4 * var(--font-step-px)); font-size: var(--hud-tag-size); line-height: 1.5; font-weight: var(--hud-figure-weight); text-align: center; color: var(--hud-key-ink); background: var(--hud-key-fill); border-radius: var(--hud-key-radius); box-shadow: var(--hud-key-shadow); white-space: nowrap; text-overflow: -og-scroll var(--motion-duration-orbit) var(--motion-duration-turn); overflow: hidden; }
.group[aria-pressed=true] .key { background: var(--hud-key-lit-fill); }
.picture { flex: none; inline-size: var(--size-sm); block-size: var(--size-sm); object-fit: cover; border-radius: var(--hud-tag-cut) 0; corner-shape: var(--cut-shape); clip-path: border-box; }
.count { flex: none; font-size: var(--hud-size-sm); font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); font-variant-numeric: var(--hud-figure-numeric); white-space: nowrap; }
[data-alert=true] .count { color: var(--hud-gold); }
/* The key cap stays under a touch screen too: it names the group. */
@media (width < 600px), (height < 500px) {
  :host { column-gap: calc(4 * var(--font-step-px)); }
  .group::part(base) { padding: calc(2 * var(--font-step-px)) calc(7 * var(--font-step-px)); }
  .count { font-size: var(--hud-size-xs); }
}
</style>
