<meta name="component" content="IdleGenerators">
<meta name="strings" content="./idle.en.ogt">
<meta name="prop:title" content="string" data-doc="The list's heading; empty takes the group's (Generators).">
<meta name="prop:generators" content="list" data-doc="The producers in the game's order, keyed rows of { id, icon, name, owned, rate, cycle, cycleText, level, boost, auto, locked }: owned how many the player has (a number, compacted; 0 hides it); rate what it yields as the game words it (+1.2K/s; on a locked row what unlocks it); cycle the running cycle 0 to 1 (negative hides the bar); cycleText the time left as the game words it (empty shows the percent); level its level (0 hides it); boost a multiplier as the game words it (×2; empty hides it); auto it runs by itself (named Auto); locked not yet available (named Locked, disabled).">
<meta name="prop:selected" content="string" data-doc="The generator the player last chose, selected; empty is none.">
<meta name="fixture:default" content='{"title":"","generators":[{"id":"farm","icon":"wheat","name":"Farm","owned":25,"rate":"+48/s","cycle":0.62,"cycleText":"0.4s","level":3,"boost":"","auto":true,"locked":false},{"id":"mine","icon":"gem","name":"Gem Mine","owned":4,"rate":"+1.2K/s","cycle":0.18,"cycleText":"8.2s","level":1,"boost":"×2","auto":false,"locked":false},{"id":"forge","icon":"fire","name":"Forge","owned":0,"rate":"10K gold","cycle":-1,"cycleText":"","level":0,"boost":"","auto":false,"locked":true}],"selected":"mine"}'>
<meta name="fixture:empty" content='{"title":"","generators":[],"selected":""}'>
<meta name="fixture:disabled" content='{"title":"","generators":[{"id":"forge","icon":"fire","name":"Forge","owned":0,"rate":"Reach 1M gold","cycle":-1,"cycleText":"","level":0,"boost":"","auto":false,"locked":true},{"id":"tower","icon":"crown","name":"Mage Tower","owned":0,"rate":"Reach 1B gold","cycle":-1,"cycleText":"","level":0,"boost":"","auto":false,"locked":true}],"selected":""}'>
<meta name="fixture:selected" content='{"title":"","generators":[{"id":"farm","icon":"wheat","name":"Farm","owned":25,"rate":"+48/s","cycle":1,"cycleText":"","level":3,"boost":"","auto":false,"locked":false},{"id":"mine","icon":"gem","name":"Gem Mine","owned":4,"rate":"+1.2K/s","cycle":0,"cycleText":"","level":1,"boost":"","auto":false,"locked":false}],"selected":"farm"}'>
<meta name="fixture:overflow" content='{"title":"Generators of the Sprawling Clockwork Empire","generators":[{"id":"g1","icon":"leaf","name":"Overgrown Terraced Hydroponic Garden Complex","owned":1.25e15,"rate":"+4.56 quintillion per second","cycle":0.5,"cycleText":"1h 22m 10s","level":999,"boost":"×1,024","auto":true,"locked":false},{"id":"g2","icon":"gem","name":"Mine","owned":12,"rate":"+1.2K/s","cycle":0.1,"cycleText":"3s","level":2,"boost":"","auto":false,"locked":false},{"id":"g3","icon":"fire","name":"Forge","owned":3,"rate":"+30K/s","cycle":0.9,"cycleText":"0.1s","level":1,"boost":"×3","auto":true,"locked":false},{"id":"g4","icon":"bolt","name":"Dynamo","owned":1,"rate":"+400K/s","cycle":0.33,"cycleText":"20s","level":1,"boost":"","auto":false,"locked":false},{"id":"g5","icon":"crown","name":"Mage Tower","owned":0,"rate":"+9M/s","cycle":-1,"cycleText":"","level":0,"boost":"","auto":false,"locked":true},{"id":"g6","icon":"star","name":"Star Engine","owned":0,"rate":"+1B/s","cycle":-1,"cycleText":"","level":0,"boost":"","auto":false,"locked":true},{"id":"g7","icon":"skull","name":"Necropolis","owned":0,"rate":"+80B/s","cycle":-1,"cycleText":"","level":0,"boost":"","auto":false,"locked":true}],"selected":"g1"}'>

<!--
  The generator list of an idle game: one cut row per producer with its art
  in a socket and its level on a hexagon at the socket's foot, its name and
  how many are owned, its yield in green, its running cycle as a leaning
  bar with the time left, and its Auto and boost tags. The game runs the
  cycles and owns every number; the list reports choose (a row's key) and
  the chosen row is washed in the accent. A locked row is disabled and
  greyed, and says Locked with what unlocks it. It fills its height and
  scrolls, its foot fading while there is more.
-->
<section class="frame" id="IdleGenerators" part="frame" layout="stack" fill gap="loose" aria-label={title != '' ? title : t("idle.generators.name")}>
  <div class="lit"></div>
  <header class="band" part="band" layout="stack" gap="none">
    <span part="kicker" class="kicker">{t("idle.generators.kicker")}</span>
    <h2 class="title" part="title">{title != '' ? title : t("idle.generators.name")}</h2>
  </header>
  {#if generators.length}
    <div class="rows-shell" layout="stack" gap="none" fill>
    <div class="rows-scroll" layout="scroll" gap="none">
      <div class="rows" layout="stack" gap="tight" role="listbox" aria-label={title != '' ? title : t("idle.generators.name")}>
        {#each generators as g (g.id)}
          <button class="row" part="row" role="option" aria-selected={g.id == selected} aria-label={g.locked ? t("idle.generators.lockedName", { name: g.name, what: g.rate }) : g.level > 0 ? t("idle.generators.levelName", { name: g.name, level: formatNumber(g.level) }) : g.name} aria-disabled={g.locked} disabled={g.locked} data-auto={g.auto} data-boosted={!!g.boost} onclick={choose} state:selected={g.id == selected} state:locked={g.locked}><div class="face" layout="row" gap="loose">
            <div class="art">
              <div part="socket" class="socket cover" layout="stack" fill><og-icon class="icon" icon={g.icon}></og-icon></div>
              {#if !g.locked && g.level > 0}<div part="level" class="level" layout="center"><span>{formatNumber(g.level)}</span></div>{/if}
            </div>
            <div class="body" layout="stack" gap="hair" fill>
              <div class="line" layout="row" gap="tight">
                <span part="name" class="name">{g.name}</span>
                {#if !g.locked && g.owned > 0}<span part="owned" class="owned">{t("idle.generators.owned", { owned: formatCompact(g.owned) })}</span>{/if}
                <span class="spacer" fill></span>
                {#if !g.locked}<span part="rate" class="rate">{g.rate}</span>{/if}
              </div>
              <div class="line" layout="row" gap="tight">
                {#if !g.locked && g.cycle >= 0}
                  <div part="track" class="track" fill role="progressbar" aria-label={t("idle.generators.cycle")} aria-valuemin="0" aria-valuemax="100" aria-valuenow={g.cycle * 100} aria-valuetext={g.cycleText ? g.cycleText : formatPercent(g.cycle)}><div part="fill" class="fill" style:--value={g.cycle > 1 ? 1 : g.cycle}></div></div>
                  <span part="time" class="time">{g.cycleText ? g.cycleText : formatPercent(g.cycle)}</span>
                {:else}<span class="spacer" fill></span>{/if}
                {#if !g.locked && g.boost}<span part="tag" class="tag" class:tag-boost={true}>{g.boost}</span>{/if}
                {#if !g.locked && g.auto}<span part="tag" class="tag" class:tag-auto={true}>{t("idle.generators.auto")}</span>{/if}
                {#if g.locked}<div part="tag" class="tag" layout="row" gap="hair" class:tag-locked={true}><span>{t("idle.generators.locked")}</span><span class="dot"></span><span class="unlock">{g.rate}</span></div>{/if}
              </div>
            </div>
          </div></button>
        {/each}
      </div>
    </div>
    <div class="scroll-fade" part="fade" anchor="bottom-edge"></div>
    </div>
  {:else}
    <p class="empty-text" part="kicker" role="status">{t("idle.generators.empty")}</p>
  {/if}
</section>

<style>
.cover { justify-content: center; align-items: center; }
#IdleGenerators { inline-size: stretch; block-size: stretch; min-block-size: 0; }
.frame { position: relative; border-radius: var(--inv-frame-cut) 0; corner-shape: var(--cut-shape); clip-path: border-box; }
.lit { position: absolute; inset-inline: var(--inv-frame-cut) 30%; top: 0; block-size: var(--stroke-hairline); background: var(--inv-lit-edge); pointer-events: none; }
.kicker { white-space: nowrap; text-overflow: ellipsis; overflow: hidden; font-size: var(--inv-kicker-size); font-weight: var(--hud-label-weight); letter-spacing: var(--inv-kicker-tracking); text-transform: var(--inv-kicker-transform); color: var(--hud-label-ink); }
.band { width: stretch; flex: none; align-items: flex-start; }
.title { margin: 0; max-inline-size: 100%; white-space: nowrap; text-overflow: ellipsis; overflow: hidden; }
.rows-shell { position: relative; min-block-size: 0; }
.rows-scroll { overflow-y: auto; flex: 1; min-height: 0; }
.rows { padding: var(--space-xxs); padding-block-end: var(--space-lg); }
/* A row: cut at two corners, the chosen one washed in the accent. */
.row { width: stretch; min-height: var(--size-touch); padding: var(--space-xs) var(--space-sm); min-width: 0; border: none; background: var(--inv-row-fill); box-shadow: var(--inv-row-rim); color: var(--ink-bright); border-radius: var(--inv-tag-cut) 0; corner-shape: var(--cut-shape); clip-path: border-box; }
.row:state(selected) { background: var(--inv-row-on-fill); box-shadow: var(--inv-row-on-rim); }
.row:state(locked) { color: var(--ink-bright); }
:state(locked) .art, :state(locked) .name { opacity: var(--disabled-content-opacity); }
:state(locked) .icon { filter: var(--play-locked-art); }
.face { width: stretch; align-items: center; }
.art { position: relative; flex: none; inline-size: var(--size-lg); block-size: var(--size-lg); }
.socket { position: absolute; inset: 0; background: var(--inv-socket-fill); box-shadow: var(--inv-socket-rim); border-radius: var(--inv-tag-cut) 0; corner-shape: var(--cut-shape); clip-path: border-box; }
.icon { width: 76%; height: 76%; filter: var(--inv-slot-art-shadow); }
/* The level on a hexagon at the socket's foot. */
.level { position: absolute; inset-inline-start: calc(var(--space-xxs) * -1); bottom: calc(var(--space-xs) * -1); inline-size: calc(var(--size-xs) * 1.2); block-size: var(--size-xs); padding: 0; text-align: center; white-space: nowrap; font-size: var(--hud-tag-size); font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); font-variant-numeric: var(--hud-figure-numeric); color: var(--play-level-ink); background: var(--play-level-fill); clip-path: polygon(25% 0, 75% 0, 100% 50%, 75% 100%, 25% 100%, 0 50%); }
.body { min-width: 0; }
.line { width: stretch; min-width: 0; align-items: center; }
.name { flex: 0 1 auto; min-width: 0; text-align: start; white-space: nowrap; text-overflow: ellipsis; overflow: hidden; font-size: var(--play-figure-size); font-weight: var(--inv-name-weight); color: var(--ink-bright); }
.owned { flex: none; white-space: nowrap; font-size: var(--hud-tag-size); font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); font-variant-numeric: var(--hud-figure-numeric); color: var(--hud-label-ink); }
.spacer { min-width: 0; }
.rate { flex: none; max-width: 50%; white-space: nowrap; text-overflow: ellipsis; overflow: hidden; font-size: var(--play-figure-size); font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); font-variant-numeric: var(--hud-figure-numeric); color: var(--play-gain-ink); }
/* The cycle: a thin bar leaning in its well, the time left beside it. */
.track { position: relative; min-width: var(--size-lg); block-size: calc(8 * var(--font-step-px)); overflow: hidden; transform: skewX(var(--bar-skew)); background: var(--bar-well); box-shadow: var(--bar-rim); }
.fill { inline-size: calc(var(--value) * 100%); block-size: 100%; background: var(--play-cycle-fill); }
.time { flex: none; white-space: nowrap; font-size: var(--hud-tag-size); font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); font-variant-numeric: var(--hud-figure-numeric); color: var(--hud-label-ink); }
.tag { flex: none; padding: 0 var(--space-xs); white-space: nowrap; font-size: var(--hud-tag-size); font-weight: var(--hud-label-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); color: var(--hud-label-ink); background: var(--hud-tag-fill); border-radius: var(--hud-tag-cut) 0; corner-shape: var(--cut-shape); clip-path: border-box; }
.tag.tag-auto { color: var(--play-auto-ink); background: var(--play-auto-fill); }
.tag.tag-boost { color: var(--play-boost-ink); background: var(--play-boost-fill); }
.tag.tag-locked { min-width: 0; max-width: stretch; align-items: center; }
.dot { flex: none; inline-size: calc(3 * var(--font-step-px)); block-size: calc(3 * var(--font-step-px)); background: currentColor; }
.unlock { min-width: 0; white-space: nowrap; text-overflow: ellipsis; overflow: hidden; }
.empty-text { margin: 0; flex: 1; padding: var(--space-lg); text-align: center; }
/* A narrow panel (a phone, or a side column on a wide screen) tightens the rows. */
@container (width < 560px) {
  .row { padding: var(--space-xs); }
  .face { --layout-gap-loose: var(--layout-gap-tight); }
  .art { inline-size: var(--size-md); block-size: var(--size-md); }
}
</style>
