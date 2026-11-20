<meta name="component" content="PuzzleBoosters">
<meta name="strings" content="./puzzle.en.ogt">
<meta name="prop:boosters" content="list" data-doc="The boosters in tray order, keyed rows of { id, icon, name, count, available }: count how many the player holds, as the game counts them; available false when the game will not take it now (none left, not allowed on this level, a move in progress): the booster is disabled, sends nothing and says so.">
<meta name="prop:armed" content="string" data-doc="The booster the game has armed (waiting for its target on the board); empty is none. Shown pressed and worded Armed.">
<meta name="action:choose" content="Use">
<meta name="fixture:default" content='{"armed":"","boosters":[{"id":"hammer","icon":"bomb","name":"Hammer","count":3,"available":true},{"id":"shuffle","icon":"bolt","name":"Shuffle","count":1,"available":true},{"id":"moves","icon":"plus","name":"Extra moves","count":5,"available":true},{"id":"burst","icon":"fire","name":"Colour burst","count":0,"available":false}]}'>
<meta name="fixture:selected" content='{"armed":"hammer","boosters":[{"id":"hammer","icon":"bomb","name":"Hammer","count":3,"available":true},{"id":"shuffle","icon":"bolt","name":"Shuffle","count":1,"available":true},{"id":"moves","icon":"plus","name":"Extra moves","count":5,"available":true}]}'>
<meta name="fixture:disabled" content='{"armed":"","boosters":[{"id":"hammer","icon":"bomb","name":"Hammer","count":3,"available":false},{"id":"shuffle","icon":"bolt","name":"Shuffle","count":0,"available":false},{"id":"moves","icon":"plus","name":"Extra moves","count":2,"available":false}]}'>
<meta name="fixture:empty" content='{"armed":"","boosters":[]}'>
<meta name="fixture:overflow" content='{"armed":"b4","boosters":[{"id":"b1","icon":"bomb","name":"Hammer","count":999,"available":true},{"id":"b2","icon":"bolt","name":"Shuffle","count":12,"available":true},{"id":"b3","icon":"plus","name":"Extra moves","count":1250,"available":true},{"id":"b4","icon":"fire","name":"Colour burst of the Whispering Lemonade Falls","count":7,"available":true},{"id":"b5","icon":"star","name":"Lucky star","count":0,"available":false},{"id":"b6","icon":"key","name":"Unlock","count":2,"available":true},{"id":"b7","icon":"gem","name":"Gem rain","count":40,"available":false},{"id":"b8","icon":"drop","name":"Jelly spill","count":3,"available":true}]}'>

<!--
  The booster tray: each booster a round socket with its art, how many the
  player holds on a tag at its foot and its name under it. The one the
  game has armed is lifted, ringed and tagged Armed; one the game will not
  take is disabled, its art dimmed, tagged in words (Unavailable), and one
  the player has none of shows a gold + where its count was. Reports
  choose (a booster's key; tap, click, Enter or pad A); the game decides
  whether it applies, spends it and arms or fires it. Many boosters scroll
  across the tray; a narrow tray wraps them onto more rows.
-->
<nav id="PuzzleBoosters" part="frame" variant="hud" layout="stack" fill aria-label={t("puzzle.boosters.name")}>
  {#if boosters.length}
    <div class="tray-scroll" fill>
      <div class="tray" layout="row" gap="tight" role="toolbar" aria-label={t("puzzle.boosters.name")}>
        {#each boosters as b (b.id)}
          <button class="booster" part="booster" aria-pressed={b.id == armed} disabled={!b.available} aria-label={!b.available ? t("puzzle.boosters.unavailable", { name: b.name }) : b.id == armed ? t("puzzle.boosters.armedName", { name: b.name, count: b.count }) : t("puzzle.boosters.count", { name: b.name, count: b.count })} onclick={choose} class:armed={b.available && b.id == armed} class:not-available={!(b.available)} class:out={b.count == 0}><div class="face" layout="stack" gap="hair">
            {#if !b.available || b.id == armed}<span class="state" part="state" class:armed={b.available && b.id == armed}>{b.available ? t("puzzle.boosters.armed") : t("puzzle.boosters.locked")}</span>{:else}<div class="state-space"></div>{/if}
            <div class="well">
              <div part="socket" class="socket cover" layout="stack" fill><og-icon class="art" icon={b.icon}></og-icon></div>
              <span class="count" part="count" class:out={b.count == 0}>{b.count == 0 ? '+' : t("puzzle.boosters.times", { count: formatNumber(b.count) })}</span>
            </div>
            <span part="name" class="name">{b.name}</span>
          </div></button>
        {/each}
      </div>
    </div>
  {:else}
    <p part="empty" class="empty" role="status">{t("puzzle.boosters.empty")}</p>
  {/if}
</nav>

<style>
.cover { justify-content: center; align-items: center; }
/* Drawn over the scene: the style's tag halo holds its text there. */
#PuzzleBoosters { text-shadow: var(--tag-halo); --scene-halo: var(--tag-halo); }
#PuzzleBoosters { padding: var(--space-xs); }
.tray-scroll { overflow-x: auto; overflow-y: hidden; }
.tray { width: max-content; min-width: 100%; justify-content: center; padding: var(--space-xs) var(--space-xxs) var(--space-xxs); }
.booster { flex: none; width: max-content; min-width: calc(var(--size-xl) * 1.5); max-width: calc(var(--size-xl) * 2.5); min-height: var(--size-touch); padding: var(--space-xxs) var(--space-xs); border: none; border-radius: 0; background: var(--inv-bare-fill); box-shadow: none; }
.face { align-items: center; min-width: 0; }
.well { position: relative; width: var(--size-xl); height: var(--size-xl); }
/* The socket: a round well; armed, lifted and ringed in the accent. */
.socket { position: absolute; inset: 0; border-radius: var(--play-socket-radius); background: var(--play-socket-fill); box-shadow: var(--play-socket-rim); }
.armed .socket { translate: 0 calc(var(--play-lift) * -1); box-shadow: var(--play-armed-rim); }
.art { width: 70%; height: 70%; filter: var(--inv-slot-art-shadow); }
.not-available .art { filter: var(--play-out-art); }
/* The count on a dark tag at the socket's foot; none left, a gold +. */
.count { position: absolute; inset-inline-end: calc(var(--space-xs) * -1); bottom: calc(var(--space-xxs) * -1); min-width: var(--size-xs); padding: 0 var(--space-xxs); text-align: center; white-space: nowrap; font-size: var(--hud-tag-size); font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); color: var(--ink-bright); background: var(--hud-tag-fill); box-shadow: var(--hud-tag-rim); border-radius: var(--hud-tag-cut) 0; corner-shape: var(--cut-shape); clip-path: border-box; }
.count.out { color: var(--play-out-tag-ink); background: var(--play-out-tag-fill); }
/* Armed or Unavailable in words on a tag over the socket, in the
   booster's own column, so it is never cut by the tray or a neighbour. */
.state, .state-space { flex: none; block-size: calc(var(--hud-tag-size) * 1.45); margin-block-end: var(--space-xxs); }
.state { padding: 0 var(--space-xxs); white-space: nowrap; line-height: 1.45; font-size: var(--hud-tag-size); font-weight: var(--hud-label-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); color: var(--hud-label-ink); background: var(--hud-tag-fill); box-shadow: var(--hud-tag-rim); }
.armed .state { color: var(--play-armed-tag-ink); background: var(--play-armed-ink); }
.name { max-inline-size: calc(var(--size-xl) * 2); text-align: center; line-clamp: 2; overflow: hidden; font-size: var(--hud-tag-size); font-weight: var(--inv-name-weight); color: var(--ink-bright); }
.armed .name { color: var(--play-armed-ink); }
.not-available .name { color: var(--hud-label-ink); }
.empty { margin: 0; flex: 1; padding: var(--space-lg); text-align: center; }
@container (width < 600px) {
  .tray { width: stretch; min-width: 0; flex-wrap: wrap; justify-content: center; row-gap: var(--space-xs); --layout-gap-tight: var(--layout-gap-hair); }
  .booster { min-width: calc(var(--size-lg) * 1.9); max-width: none; }
  .name { max-inline-size: calc(var(--size-lg) * 1.9); }
  .well { width: var(--size-lg); height: var(--size-lg); }
}
</style>
