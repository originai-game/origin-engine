<meta name="component" content="StrategyWave">
<meta name="strings" content="./strategy.en.ogt">
<meta name="parts" content="StrategyWave.frame StrategyWave.readout StrategyWave.title StrategyWave.badge StrategyWave.row StrategyWave.name StrategyWave.figure">
<meta name="prop:wave" content="number" data-doc="The wave on now or last fought, from 1; 0 before the first.">
<meta name="prop:waves" content="number" data-doc="Waves in the level; 0 is an endless run (no total shown).">
<meta name="prop:phase" content="string" data-doc="countdown (the next wave is coming), active (a wave is on) or done (every wave cleared), as the game says: data-phase and a word.">
<meta name="prop:next" content="number" data-default="-1" data-doc="Seconds to the next wave (HudTimer); negative hides the clock.">
<meta name="prop:nextTotal" content="number" data-doc="The whole wait, for the timer's ring; 0 draws none.">
<meta name="prop:finalWave" content="bool" data-doc="The coming or current wave is the last: the Final wave badge.">
<meta name="prop:enemies" content="list" data-doc="The next wave's enemies by kind, keyed rows of { id, name, icon, count, boss, scouted }: count shown as ×n; boss marks it with a Boss badge (data-boss); scouted false hides name and count behind Not yet scouted.">
<meta name="prop:canCallEarly" content="bool" data-doc="The game allows calling the next wave now; false hides Call early.">
<meta name="prop:reward" content="string" data-doc="What calling early pays, as the game words it (+40 gold), shown on the button; empty none.">
<meta name="prop:warnAt" content="number" data-default="5" data-doc="Seconds left when the countdown turns warn (HudTimer's warnAt); 5 when left out.">
<meta name="fixture:default" content='{"wave":6,"waves":20,"phase":"countdown","next":14.2,"nextTotal":30,"finalWave":false,"canCallEarly":true,"reward":"+35 gold","enemies":[{"id":"goblin","name":"Goblin","icon":"skull","count":18,"boss":false,"scouted":true},{"id":"wolf","name":"Warg rider","icon":"bolt","count":6,"boss":false,"scouted":true},{"id":"shaman","name":"Shaman","icon":"fire","count":2,"boss":false,"scouted":true}]}'>
<meta name="fixture:active" content='{"wave":7,"waves":20,"phase":"active","next":-1,"nextTotal":0,"finalWave":false,"canCallEarly":false,"reward":"","enemies":[{"id":"goblin","name":"Goblin","icon":"skull","count":24,"boss":false,"scouted":true},{"id":"ogre","name":"Ogre","icon":"shield","count":3,"boss":false,"scouted":true}]}'>
<meta name="fixture:final" content='{"wave":20,"waves":20,"phase":"countdown","next":6.5,"nextTotal":45,"finalWave":true,"canCallEarly":true,"reward":"+120 gold","enemies":[{"id":"warlord","name":"Iron Warlord","icon":"crown","count":1,"boss":true,"scouted":true},{"id":"guard","name":"Honour guard","icon":"shield","count":8,"boss":false,"scouted":true},{"id":"x","name":"","icon":"skull","count":0,"boss":false,"scouted":false}]}'>
<meta name="fixture:done" content='{"wave":20,"waves":20,"phase":"done","next":-1,"nextTotal":0,"finalWave":false,"canCallEarly":false,"reward":"","enemies":[]}'>
<meta name="fixture:empty" content='{"wave":0,"waves":0,"phase":"countdown","next":30,"nextTotal":30,"finalWave":false,"canCallEarly":true,"reward":"","enemies":[]}'>
<meta name="fixture:overflow" content='{"wave":1284,"waves":0,"phase":"countdown","next":3725,"nextTotal":3725,"finalWave":false,"canCallEarly":true,"reward":"+12,450,000 gold and a Legendary Chest","enemies":[{"id":"e1","name":"Abyssal Behemoth of the Thousand-Year Eclipse","icon":"skull","count":1,"boss":true,"scouted":true},{"id":"e2","name":"Goblin","icon":"skull","count":12480,"boss":false,"scouted":true},{"id":"e3","name":"Warg rider","icon":"bolt","count":3200,"boss":false,"scouted":true},{"id":"e4","name":"Shaman","icon":"fire","count":640,"boss":false,"scouted":true},{"id":"e5","name":"Ogre","icon":"shield","count":210,"boss":false,"scouted":true},{"id":"e6","name":"Wyvern","icon":"drop","count":96,"boss":false,"scouted":true},{"id":"e7","name":"","icon":"skull","count":0,"boss":false,"scouted":false}]}'>

<!--
  The wave readout: wave n of N (or n in an endless run), the phase as a
  word, the next wave's countdown (HudTimer), the enemies coming by kind
  with their counts, and Call early with what it pays. Reports callEarly;
  the game owns timing, the wave's makeup and the reward.
-->
<section id="StrategyWave" part="StrategyWave.frame" class="hud" aria-label={t("strategy.wave.name")} data-phase={phase} data-final={finalWave}>
  <div class="head">
    <div part="StrategyWave.readout StrategyWave.figure" class="count" role="group" aria-label={waves > 0 ? t("strategy.wave.ofName", { wave: wave, waves: waves }) : t("strategy.wave.nName", { wave: wave })}>
      <span part="StrategyWave.title" class="caption">{t("strategy.wave.wave")}</span>
      <div class="figure"><span class="now">{formatNumber(wave)}</span>{#if waves > 0}<span class="of">{t("strategy.wave.of", { total: waves })}</span>{/if}</div>
    </div>
    <div class="status">
      <span part="StrategyWave.badge" class="phase" data-phase={phase} role="status">{phase == 'done' ? t("strategy.wave.done") : (phase == 'active' ? t("strategy.wave.active") : t("strategy.wave.countdown"))}</span>
      {#if finalWave}<span part="StrategyWave.badge" class="final">{t("strategy.wave.final")}</span>{/if}
    </div>
    {#if next >= 0 && phase != 'done'}<div class="clock"><HudTimer seconds={next < 0 ? 0 : next} total={nextTotal} countdown={true} warnAt={warnAt} paused={false} label={t("strategy.wave.next")} /></div>{/if}
  </div>
  {#if enemies.length && phase != 'done'}
  <div class="incoming">
    <span part="StrategyWave.title" class="caption">{t("strategy.wave.incoming")}</span>
    <div class="enemies" role="list" aria-label={t("strategy.wave.incoming")}>
      {#each enemies as e (e.id)}
        <div part="StrategyWave.row" role="listitem" aria-label={e.scouted ? t("strategy.wave.enemyName", { count: e.count, name: e.name }) : t("strategy.wave.unknown")} data-boss={e.boss} data-scouted={e.scouted}>
          <og-icon class="mark" icon={e.icon}></og-icon>
          <span part="StrategyWave.name" class="enemy-name">{e.scouted ? e.name : t("strategy.wave.unknown")}</span>
          {#if e.boss}<span part="StrategyWave.badge" class="boss">{t("strategy.wave.boss")}</span>{/if}
          {#if e.scouted}<span class="enemy-count">{t("strategy.wave.enemyCount", { count: e.count })}</span>{/if}
        </div>
      {/each}
    </div>
  </div>
  {/if}
  {#if canCallEarly && phase != 'done'}<button id="CallEarly" variant="primary" onclick={callEarly}><span>{t("strategy.wave.callEarly")}</span>{#if reward != ''}<span class="reward">{reward}</span>{/if}</button>{/if}
</section>

<style>
/* Drawn over the scene: the style's tag halo holds its text there. */
:host { text-shadow: var(--tag-halo); }
:host { display: flex; flex-direction: column; inline-size: fit-content; max-inline-size: stretch; min-inline-size: 0; gap: var(--space-md); padding: var(--space-md); }
.head { display: flex; align-items: flex-end; gap: var(--space-md); flex-wrap: wrap; }
[part="StrategyWave.readout StrategyWave.figure"] { display: flex; flex-direction: column; align-items: flex-start; gap: 0; flex: none; }
.caption { font: var(--text-sm); color: var(--ink-muted); white-space: nowrap; }
.now { font: var(--text-xl2); font-weight: var(--font-weight-semibold); }
.of { color: var(--ink-muted); }
.figure { display: flex; align-items: baseline; gap: var(--space-xxs); white-space: nowrap; }
.now, .of, .enemy-count { font-variant-numeric: tabular-nums; white-space: nowrap; }
.status { display: flex; flex-direction: column; align-items: flex-start; gap: var(--space-xxs); min-width: 0; }
.phase, .final, .boss { white-space: nowrap; padding: 0 var(--space-xs); }
.clock { flex: none; }
.incoming { display: flex; flex-direction: column; gap: var(--space-xs); }
.enemies { display: flex; flex-direction: column; gap: var(--space-xs); }
[part="StrategyWave.row"] { display: flex; width: stretch; align-items: center; gap: var(--space-xs); min-width: 0; }
.mark { flex: none; width: var(--size-sm); height: var(--size-sm); }
.enemy-name { flex: 1; min-width: 0; white-space: nowrap; text-overflow: ellipsis; overflow: hidden; }
.enemy-count { flex: none; min-width: var(--size-lg); text-align: end; }
#CallEarly { align-self: stretch; flex-wrap: wrap; min-height: var(--size-touch); gap: var(--space-xxs) var(--space-sm); max-width: stretch; }
.reward { font: var(--text-sm); overflow-wrap: anywhere; }
@media (width < 600px) {
  :host { inline-size: stretch; padding: var(--space-sm); gap: var(--space-sm); }
  .head { gap: var(--space-sm); }
  #CallEarly { align-self: stretch; }
}
</style>
