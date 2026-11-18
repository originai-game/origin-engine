<meta name="component" content="StrategyWave">
<meta name="strings" content="./strategy.en.ogt">
<meta name="parts" content="StrategyWave.frame StrategyWave.plate StrategyWave.icon StrategyWave.label StrategyWave.time StrategyWave.bar StrategyWave.fill StrategyWave.state StrategyWave.badge StrategyWave.row StrategyWave.mark StrategyWave.name StrategyWave.count StrategyWave.call StrategyWave.reward">
<meta name="prop:wave" content="number" data-doc="The wave on now or coming, from 1.">
<meta name="prop:waves" content="number" data-doc="Waves in the level; 0 is an endless run (no total shown).">
<meta name="prop:name" content="string" data-doc="The wave's name as the game words it (Siege), after its number; empty none.">
<meta name="prop:icon" content="number" data-doc="The wave's picture (og.load.texture) in a cut tile before it; 0 hides it.">
<meta name="prop:phase" content="string" data-doc="countdown (the wave is coming), active (a wave is on) or done (every wave cleared), as the game says: data-phase; active and done stand as words in place of the countdown.">
<meta name="prop:next" content="number" data-default="-1" data-doc="Seconds to the wave while it is coming, shown as a countdown (0:42); negative hides it.">
<meta name="prop:nextTotal" content="number" data-doc="The whole wait: the bar beside the countdown shows the share still to run; 0 draws no bar.">
<meta name="prop:warnAt" content="number" data-default="5" data-doc="Seconds left when the countdown is near (data-near on the frame); 5 when left out.">
<meta name="prop:finalWave" content="bool" data-doc="The coming or current wave is the last: the Final wave badge (data-final).">
<meta name="prop:enemies" content="list" data-doc="The wave's enemies by kind, under the plate, keyed rows of { id, name, icon, count, boss, scouted }: icon an icon of the atlas; count shown as ×n; boss marks it with a Boss badge (data-boss); scouted false hides name and count behind Not yet scouted. Empty shows none.">
<meta name="prop:canCallEarly" content="bool" data-doc="The game allows calling the wave now; false hides Call early.">
<meta name="prop:reward" content="string" data-doc="What calling early pays, as the game words it (+40 gold), on the button in gold; empty none.">
<meta name="fixture:default" content='{"wave":7,"waves":0,"name":"Siege","icon":0,"phase":"countdown","next":42,"nextTotal":140,"warnAt":5,"finalWave":false,"canCallEarly":false,"reward":"","enemies":[]}'>
<meta name="fixture:incoming" content='{"wave":6,"waves":20,"name":"","icon":0,"phase":"countdown","next":14.2,"nextTotal":30,"warnAt":5,"finalWave":false,"canCallEarly":true,"reward":"+35 gold","enemies":[{"id":"goblin","name":"Goblin","icon":"skull","count":18,"boss":false,"scouted":true},{"id":"wolf","name":"Warg rider","icon":"bolt","count":6,"boss":false,"scouted":true},{"id":"shaman","name":"Shaman","icon":"fire","count":2,"boss":false,"scouted":true}]}'>
<meta name="fixture:active" content='{"wave":7,"waves":20,"name":"","icon":0,"phase":"active","next":-1,"nextTotal":0,"warnAt":5,"finalWave":false,"canCallEarly":false,"reward":"","enemies":[{"id":"goblin","name":"Goblin","icon":"skull","count":24,"boss":false,"scouted":true},{"id":"ogre","name":"Ogre","icon":"shield","count":3,"boss":false,"scouted":true}]}'>
<meta name="fixture:final" content='{"wave":20,"waves":20,"name":"Warlord","icon":0,"phase":"countdown","next":4.5,"nextTotal":45,"warnAt":5,"finalWave":true,"canCallEarly":true,"reward":"+120 gold","enemies":[{"id":"warlord","name":"Iron Warlord","icon":"crown","count":1,"boss":true,"scouted":true},{"id":"guard","name":"Honour guard","icon":"shield","count":8,"boss":false,"scouted":true},{"id":"x","name":"","icon":"skull","count":0,"boss":false,"scouted":false}]}'>
<meta name="fixture:done" content='{"wave":20,"waves":20,"name":"","icon":0,"phase":"done","next":-1,"nextTotal":0,"warnAt":5,"finalWave":false,"canCallEarly":false,"reward":"","enemies":[]}'>
<meta name="fixture:empty" content='{"wave":1,"waves":0,"name":"","icon":0,"phase":"countdown","next":30,"nextTotal":30,"warnAt":5,"finalWave":false,"canCallEarly":true,"reward":"","enemies":[]}'>
<meta name="fixture:overflow" content='{"wave":1284,"waves":0,"name":"The Abyssal Tide of the Thousand-Year Eclipse","icon":0,"phase":"countdown","next":3725,"nextTotal":3725,"warnAt":5,"finalWave":false,"canCallEarly":true,"reward":"+12,450,000 gold and a Legendary Chest","enemies":[{"id":"e1","name":"Abyssal Behemoth of the Thousand-Year Eclipse","icon":"skull","count":1,"boss":true,"scouted":true},{"id":"e2","name":"Goblin","icon":"skull","count":12480,"boss":false,"scouted":true},{"id":"e3","name":"Warg rider","icon":"bolt","count":3200,"boss":false,"scouted":true},{"id":"e4","name":"Shaman","icon":"fire","count":640,"boss":false,"scouted":true},{"id":"e5","name":"Ogre","icon":"shield","count":210,"boss":false,"scouted":true},{"id":"e6","name":"Wyvern","icon":"drop","count":96,"boss":false,"scouted":true},{"id":"e7","name":"","icon":"skull","count":0,"boss":false,"scouted":false}]}'>

<!--
  The wave readout at the top right: a plate whose edge is lit in danger
  while a wave comes or fights, its picture, Wave n (of N) and its name in
  red capitals, the countdown with a bar of the wait still to run, or
  Under attack or All waves cleared; the wave's enemies by kind under it,
  and Call early with what it pays. Reports callEarly; the game owns the
  timing, the wave's makeup and the reward.
-->
<section id="StrategyWave" part="StrategyWave.frame" class="hud bare" aria-label={t("strategy.wave.name")} data-phase={phase} data-final={finalWave} data-near={phase == 'countdown' && next >= 0 && next <= warnAt}>
  <div id="Plate" part="StrategyWave.plate">
    {#if icon}<img part="StrategyWave.icon" class="icon" src={icon} alt="">{/if}
    <div class="main">
      <div class="head">
        <span part="StrategyWave.label" class="label">{waves > 0 ? (name != '' ? t("strategy.wave.titleOfNamed", { wave: wave, waves: waves, name: name }) : t("strategy.wave.titleOf", { wave: wave, waves: waves })) : (name != '' ? t("strategy.wave.titleNamed", { wave: wave, name: name }) : t("strategy.wave.title", { wave: wave }))}</span>
        {#if finalWave}<span part="StrategyWave.badge" class="final">{t("strategy.wave.final")}</span>{/if}
      </div>
      {#if phase == 'countdown' && next >= 0}
        <div class="clock" role="timer" aria-label={t("strategy.wave.next")} aria-valuetext={formatCountdown(next)}>
          <span part="StrategyWave.time" class="time">{formatCountdown(next)}</span>
          {#if nextTotal > 0}<div part="StrategyWave.bar" class="bar" style:--left={next >= nextTotal ? 1 : next / nextTotal}><div part="StrategyWave.fill" class="fill"></div></div>{/if}
        </div>
      {:else}
        <span part="StrategyWave.state" class="state" role="status">{phase == 'done' ? t("strategy.wave.done") : (phase == 'active' ? t("strategy.wave.active") : t("strategy.wave.countdown"))}</span>
      {/if}
    </div>
  </div>
  {#if enemies.length && phase != 'done'}
  <div class="enemies" role="list" aria-label={t("strategy.wave.incoming")}>
    {#each enemies as e (e.id)}
      <div part="StrategyWave.row" class="enemy" role="listitem" aria-label={e.scouted ? t("strategy.wave.enemyName", { count: e.count, name: e.name }) : t("strategy.wave.unknown")} data-boss={e.boss} data-scouted={e.scouted}>
        <span class="edge"></span>
        <og-icon part="StrategyWave.mark" class="mark" icon={e.icon}></og-icon>
        <span part="StrategyWave.name" class="enemy-name">{e.scouted ? e.name : t("strategy.wave.unknown")}</span>
        {#if e.boss}<span part="StrategyWave.badge" class="boss">{t("strategy.wave.boss")}</span>{/if}
        {#if e.scouted}<span part="StrategyWave.count" class="enemy-count">{t("strategy.wave.enemyCount", { count: e.count })}</span>{/if}
      </div>
    {/each}
  </div>
  {/if}
  {#if canCallEarly && phase != 'done'}<button id="CallEarly" part="StrategyWave.call" onclick={callEarly}><div class="call-face"><span class="call-label">{t("strategy.wave.callEarly")}</span>{#if reward != ''}<span part="StrategyWave.reward" class="reward">{reward}</span>{/if}</div></button>{/if}
</section>

<style>
:host { display: flex; flex-direction: column; align-items: stretch; row-gap: calc(5 * var(--font-step-px)); inline-size: fit-content; max-inline-size: stretch; min-inline-size: 0; color: var(--hud-ink); }
/* The plate: smoked glass, its edge lit in danger while a wave comes or
   fights; quiet once every wave is cleared. */
#Plate { display: flex; align-items: center; column-gap: calc(10 * var(--font-step-px)); min-inline-size: 0; padding: calc(6 * var(--font-step-px)) calc(14 * var(--font-step-px)); background: var(--hud-plate-fill); box-shadow: var(--hud-plate-alert-rim); clip-path: polygon(var(--hud-plate-cut) 0, 100% 0, 100% calc(100% - var(--hud-plate-cut)), calc(100% - var(--hud-plate-cut)) 100%, 0 100%, 0 var(--hud-plate-cut)); }
[data-phase=done] #Plate { box-shadow: var(--hud-plate-rim); }
/* The picture is art: it keeps its size at any text scale. */
.icon { flex: none; inline-size: calc(var(--size-md) * 0.75); block-size: calc(var(--size-md) * 0.75); object-fit: cover; clip-path: polygon(var(--hud-tag-cut) 0, 100% 0, 100% calc(100% - var(--hud-tag-cut)), calc(100% - var(--hud-tag-cut)) 100%, 0 100%, 0 var(--hud-tag-cut)); }
.main { display: flex; flex-direction: column; row-gap: calc(1 * var(--font-step-px)); flex: 1 1 auto; min-inline-size: 0; }
.head { display: flex; align-items: center; column-gap: calc(6 * var(--font-step-px)); min-inline-size: 0; }
/* The wave's title is short: it fits, or wraps onto a second line; it is
   never cut. */
.label { flex: 0 1 auto; min-inline-size: 0; font-size: var(--hud-label-size); font-weight: var(--hud-label-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); color: var(--hud-harm); }
[data-phase=done] .label { color: var(--hud-label-ink); }
.final { flex: none; padding: 0 calc(5 * var(--font-step-px)); font-size: var(--hud-tag-size); line-height: 1.4; font-weight: var(--hud-figure-weight); white-space: nowrap; color: var(--hud-gold); background: var(--hud-tag-fill); box-shadow: var(--hud-tag-rim); clip-path: polygon(var(--hud-tag-cut) 0, 100% 0, 100% calc(100% - var(--hud-tag-cut)), calc(100% - var(--hud-tag-cut)) 100%, 0 100%, 0 var(--hud-tag-cut)); }
.clock { display: flex; align-items: center; column-gap: calc(8 * var(--font-step-px)); min-inline-size: 0; }
.time { flex: none; font-size: var(--hud-size-readout); line-height: 1.1; font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); font-variant-numeric: var(--hud-figure-numeric); white-space: nowrap; text-shadow: var(--hud-shadow); }
[data-near=true] .time { color: var(--hud-harm); }
/* The wait still to run: a thin leaning well with a red fill. */
.bar { position: relative; flex: 1 1 auto; min-inline-size: calc(var(--size-xl) * 1.2); block-size: calc(6 * var(--font-step-px)); overflow: hidden; transform: skewX(var(--bar-skew)); background: var(--bar-well); box-shadow: var(--bar-rim); }
.fill { position: absolute; left: 0; top: 0; bottom: 0; inline-size: calc(var(--left) * 100%); background: var(--hud-low-fill); }
.state { min-inline-size: 0; font-size: var(--hud-size-sm); line-height: 1.2; font-weight: var(--hud-figure-weight); white-space: nowrap; text-overflow: -og-scroll var(--motion-duration-orbit) var(--motion-duration-turn); overflow: hidden; text-shadow: var(--hud-shadow); }
[data-phase=done] .state { color: var(--hud-boon); }
/* The enemies coming, one feed row per kind. */
.enemies { display: flex; flex-direction: column; row-gap: calc(3 * var(--font-step-px)); min-inline-size: 0; }
.enemy { display: flex; align-items: center; column-gap: calc(6 * var(--font-step-px)); min-inline-size: 0; padding: calc(2 * var(--font-step-px)) calc(10 * var(--font-step-px)) calc(2 * var(--font-step-px)) 0; background: var(--hud-feed-fill-start); }
.edge { flex: none; align-self: stretch; inline-size: var(--hud-feed-edge-width); margin-inline-end: calc(4 * var(--font-step-px)); background: var(--hud-harm); }
[data-scouted=false] > .edge { background: var(--hud-feed-edge); }
.mark { flex: none; inline-size: calc(16 * var(--font-step-px)); block-size: calc(16 * var(--font-step-px)); color: var(--hud-ink); }
.enemy-name { flex: 1 1 auto; min-inline-size: 0; font-size: var(--hud-size-feed); font-weight: var(--hud-label-weight); white-space: nowrap; text-overflow: -og-scroll var(--motion-duration-orbit) var(--motion-duration-turn); overflow: hidden; }
.enemy[data-scouted=false] .enemy-name { color: var(--hud-ink-dim); }
.boss { flex: none; padding: 0 calc(5 * var(--font-step-px)); font-size: var(--hud-tag-size); line-height: 1.4; font-weight: var(--hud-figure-weight); white-space: nowrap; color: var(--hud-harm); background: var(--hud-tag-fill); box-shadow: var(--hud-tag-rim); clip-path: polygon(var(--hud-tag-cut) 0, 100% 0, 100% calc(100% - var(--hud-tag-cut)), calc(100% - var(--hud-tag-cut)) 100%, 0 100%, 0 var(--hud-tag-cut)); }
.enemy-count { flex: none; font-size: var(--hud-size-feed); font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); font-variant-numeric: var(--hud-figure-numeric); white-space: nowrap; }
/* Call early: a lit plate, what it pays in gold; a long reward scrolls in
   its line, inside the button's padding. */
#CallEarly { min-block-size: var(--size-touch); min-inline-size: 0; max-inline-size: stretch; }
#CallEarly::part(base) { padding: calc(4 * var(--font-step-px)) calc(12 * var(--font-step-px)); border: 0; border-radius: 0; background: var(--hud-plate-fill); box-shadow: var(--hud-plate-lit-rim); clip-path: polygon(var(--hud-plate-cut) 0, 100% 0, 100% calc(100% - var(--hud-plate-cut)), calc(100% - var(--hud-plate-cut)) 100%, 0 100%, 0 var(--hud-plate-cut)); }
.call-face { display: flex; flex-wrap: wrap; align-items: baseline; justify-content: center; column-gap: calc(8 * var(--font-step-px)); max-inline-size: stretch; min-inline-size: 0; }
.call-label { flex: none; font-size: var(--hud-label-size); font-weight: var(--hud-figure-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); white-space: nowrap; color: var(--hud-ink); }
.reward { flex: 0 1 auto; min-inline-size: 0; max-inline-size: 100%; font-size: var(--hud-size-xs); font-weight: var(--hud-figure-weight); color: var(--hud-gold); white-space: nowrap; text-overflow: -og-scroll var(--motion-duration-orbit) var(--motion-duration-turn); overflow: hidden; }
@media (width < 600px) {
  :host { inline-size: stretch; }
  #Plate { column-gap: calc(8 * var(--font-step-px)); padding: calc(4 * var(--font-step-px)) calc(10 * var(--font-step-px)); }
  .icon { inline-size: var(--size-sm); block-size: var(--size-sm); }
  .label { font-size: var(--hud-size-label-compact); letter-spacing: normal; }
  .time { font-size: var(--hud-size-readout-compact); }
  .bar { min-inline-size: var(--size-lg); }
}
</style>
