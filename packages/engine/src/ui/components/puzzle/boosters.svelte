<meta name="component" content="PuzzleBoosters">
<meta name="strings" content="./puzzle.en.ogt">
<meta name="parts" content="PuzzleBoosters.frame PuzzleBoosters.booster PuzzleBoosters.count PuzzleBoosters.name PuzzleBoosters.state PuzzleBoosters.empty">
<meta name="prop:boosters" content="list" data-doc="The boosters in tray order, keyed rows of { id, icon, name, count, available }: count how many the player holds, as the game counts them; available false when the game will not take it now (none left, not allowed on this level, a move in progress): the booster is disabled, sends nothing and says so.">
<meta name="prop:armed" content="string" data-doc="The booster the game has armed (waiting for its target on the board); empty is none. Shown pressed and worded Armed.">
<meta name="action:choose" content="Use">
<meta name="fixture:default" content='{"armed":"","boosters":[{"id":"hammer","icon":"bomb","name":"Hammer","count":3,"available":true},{"id":"shuffle","icon":"bolt","name":"Shuffle","count":1,"available":true},{"id":"moves","icon":"plus","name":"Extra moves","count":5,"available":true},{"id":"burst","icon":"fire","name":"Colour burst","count":0,"available":false}]}'>
<meta name="fixture:selected" content='{"armed":"hammer","boosters":[{"id":"hammer","icon":"bomb","name":"Hammer","count":3,"available":true},{"id":"shuffle","icon":"bolt","name":"Shuffle","count":1,"available":true},{"id":"moves","icon":"plus","name":"Extra moves","count":5,"available":true}]}'>
<meta name="fixture:disabled" content='{"armed":"","boosters":[{"id":"hammer","icon":"bomb","name":"Hammer","count":3,"available":false},{"id":"shuffle","icon":"bolt","name":"Shuffle","count":0,"available":false},{"id":"moves","icon":"plus","name":"Extra moves","count":2,"available":false}]}'>
<meta name="fixture:empty" content='{"armed":"","boosters":[]}'>
<meta name="fixture:overflow" content='{"armed":"b4","boosters":[{"id":"b1","icon":"bomb","name":"Hammer","count":999,"available":true},{"id":"b2","icon":"bolt","name":"Shuffle","count":12,"available":true},{"id":"b3","icon":"plus","name":"Extra moves","count":1250,"available":true},{"id":"b4","icon":"fire","name":"Colour burst of the Whispering Lemonade Falls","count":7,"available":true},{"id":"b5","icon":"star","name":"Lucky star","count":0,"available":false},{"id":"b6","icon":"key","name":"Unlock","count":2,"available":true},{"id":"b7","icon":"gem","name":"Gem rain","count":40,"available":false},{"id":"b8","icon":"drop","name":"Jelly spill","count":3,"available":true}]}'>

<!--
  The booster tray: each booster a button with its icon, how many the
  player holds and its name. A booster the game will not take is disabled
  and marked with a lock and its word; the one the game has armed is
  pressed and worded Armed. Reports choose (a booster's key; tap, click,
  Enter or pad A); the game decides whether it applies, spends it and arms
  or fires it. Many boosters scroll across the tray.
-->
<nav id="PuzzleBoosters" part="PuzzleBoosters.frame" class="hud" aria-label={t("puzzle.boosters.name")}>
  {#if boosters.length}
    <scroll class="tray-scroll">
      <div class="tray" role="toolbar" aria-label={t("puzzle.boosters.name")}>
        {#each boosters as b (b.id)}
          <button part="PuzzleBoosters.booster" data-available={b.available} data-armed={b.id == armed} aria-pressed={b.id == armed} disabled={!b.available} aria-label={!b.available ? t("puzzle.boosters.unavailable", { name: b.name }) : b.id == armed ? t("puzzle.boosters.armedName", { name: b.name, count: b.count }) : t("puzzle.boosters.count", { name: b.name, count: b.count })} onclick={choose}><div class="face">
            <div class="well">
              <og-icon class="art" icon={b.icon}></og-icon>
              {#if !b.available}<og-icon class="lock" icon="lock"></og-icon>{/if}
              <span part="PuzzleBoosters.count" data-available={b.available}>{formatNumber(b.count)}</span>
            </div>
            <span part="PuzzleBoosters.name" class="name">{b.name}</span>
            <span part="PuzzleBoosters.state" data-armed={b.available && b.id == armed} hidden={b.available && b.id != armed}>{b.available ? t("puzzle.boosters.armed") : t("puzzle.boosters.locked")}</span>
          </div></button>
        {/each}
      </div>
    </scroll>
  {:else}
    <p part="PuzzleBoosters.empty" role="status">{t("puzzle.boosters.empty")}</p>
  {/if}
</nav>

<style>
[part="PuzzleBoosters.frame"] [hidden] { display: none; }
[part="PuzzleBoosters.frame"] { display: flex; width: stretch; min-width: 0; padding: var(--space-xs); container-type: inline-size; }
.tray-scroll { width: stretch; min-width: 0; overflow-x: auto; overflow-y: hidden; }
.tray { display: flex; width: max-content; min-width: 100%; justify-content: center; gap: var(--space-sm); padding: var(--space-xxs); }
[part="PuzzleBoosters.booster"] { flex: none; width: max-content; min-width: calc(var(--size-xl) * 1.5); max-width: calc(var(--size-xl) * 2.5); min-height: var(--size-touch); padding: var(--space-xs); }
[part="PuzzleBoosters.booster"] .face { display: flex; flex-direction: column; width: stretch; align-items: center; gap: var(--space-xxs); }
.well { position: relative; display: flex; width: var(--size-xl); height: var(--size-xl); align-items: center; justify-content: center; }
.art { width: 72%; height: 72%; }
.lock { position: absolute; inset-inline-start: 0; top: 0; width: var(--size-xs); height: var(--size-xs); }
[part="PuzzleBoosters.count"] { position: absolute; inset-inline-end: calc(var(--space-xs) * -1); bottom: calc(var(--space-xxs) * -1); min-width: var(--size-xs); padding: 0 var(--space-xxs); text-align: center; white-space: nowrap; }
[part="PuzzleBoosters.name"] { width: stretch; text-align: center; text-wrap: balance; }
[part="PuzzleBoosters.state"] { max-width: stretch; padding: 0 var(--space-xxs); white-space: nowrap; text-overflow: ellipsis; overflow: hidden; }
[part="PuzzleBoosters.empty"] { margin: 0; width: stretch; padding: var(--space-sm); text-align: center; }
@container (width < 600px) {
  .tray { justify-content: flex-start; gap: var(--space-xs); }
  [part="PuzzleBoosters.booster"] { min-width: calc(var(--size-lg) * 1.75); max-width: calc(var(--size-lg) * 3); }
  .well { width: var(--size-lg); height: var(--size-lg); }
}
</style>
