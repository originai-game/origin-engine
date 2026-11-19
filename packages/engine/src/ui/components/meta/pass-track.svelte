<meta name="component" content="PassTrack">
<meta name="strings" content="./meta.en.ogt">
<meta name="parts" content="PassTrack.frame PassTrack.band PassTrack.kicker PassTrack.title PassTrack.upgrade PassTrack.level PassTrack.progress PassTrack.track PassTrack.fill PassTrack.rewards PassTrack.lane PassTrack.reward PassTrack.tier PassTrack.art PassTrack.name PassTrack.amount PassTrack.detail PassTrack.state PassTrack.claim PassTrack.caption PassTrack.figure">
<meta name="prop:title" content="string" data-default="" data-doc="Heading; empty uses Season pass.">
<meta name="prop:season" content="string" data-default="" data-doc="The game's season name, a kicker over the title.">
<meta name="prop:refresh" content="string" data-default="" data-doc="The game's already formatted time remaining, after the season in the kicker; no clock runs here.">
<meta name="prop:level" content="string" data-default="" data-doc="The player's pass level as the game words it, on a hexagon beside the progress; empty hides it.">
<meta name="prop:progress" content="number" data-default="0" data-doc="Progress to the next tier, 0 to 1, supplied by the game.">
<meta name="prop:progressText" content="string" data-default="" data-doc="Progress words, such as 240 / 500 XP; empty uses a percent.">
<meta name="prop:premiumUnlocked" content="bool" data-default="false" data-doc="The game owns the premium entitlement. Without it every premium reward reads Locked.">
<meta name="prop:upgradeLabel" content="string" data-default="" data-doc="Optional upgrade action while premium is not unlocked (Get premium); no purchase logic.">
<meta name="prop:rewards" content="list" data-default="[]" data-doc="Keyed rewards in the game's order: { id, tier, premium, name, icon, amount, detail, state }. state locked/available/claimed. Premium rewards fill the premium lane, the rest the free lane; each reward is one tile, and its tier and state are always written in words.">
<meta name="prop:loading" content="bool" data-default="false" data-doc="Loading the track; no reward action is available.">
<meta name="prop:error" content="string" data-default="" data-doc="A failure explanation with retry; empty is no error.">
<meta name="prop:disabled" content="bool" data-default="false" data-doc="Prevent claims and upgrades while the game processes an operation.">
<meta name="action:claim" content="Claim the available reward by stable key">
<meta name="action:upgrade" content="Inspect the premium upgrade">
<meta name="action:retry" content="Retry loading the track">
<meta name="fixture:default" content='{"title":"","season":"Season of Embers","refresh":"Ends in 12 days","level":"3","progress":0.48,"progressText":"240 / 500 XP","premiumUnlocked":false,"upgradeLabel":"Get premium","rewards":[{"id":"shard","tier":"1","premium":true,"name":"Ember shard","icon":"fire","amount":"×5","detail":"","state":"available"},{"id":"cloak","tier":"2","premium":true,"name":"Ashen Cloak","icon":"shield","amount":"Look","detail":"","state":"available"},{"id":"flame","tier":"3","premium":true,"name":"Phoenix flame","icon":"fire","amount":"×2","detail":"","state":"available"},{"id":"blade","tier":"4","premium":true,"name":"Ember Blade","icon":"sword","amount":"Skin","detail":"Legendary weapon skin","state":"locked"},{"id":"crown","tier":"5","premium":true,"name":"Sun Crown","icon":"crown","amount":"×1","detail":"","state":"locked"},{"id":"stones","tier":"6","premium":true,"name":"Moonstones","icon":"gem","amount":"×20","detail":"","state":"locked"},{"id":"gold","tier":"1","premium":false,"name":"Gold","icon":"coin","amount":"500","detail":"","state":"claimed"},{"id":"keys","tier":"2","premium":false,"name":"Chest keys","icon":"key","amount":"×3","detail":"","state":"claimed"},{"id":"potions","tier":"3","premium":false,"name":"Potions","icon":"potion","amount":"×5","detail":"","state":"available"},{"id":"leaf","tier":"4","premium":false,"name":"Silverleaf","icon":"leaf","amount":"×10","detail":"","state":"locked"},{"id":"map","tier":"5","premium":false,"name":"Scroll","icon":"map","amount":"×1","detail":"","state":"locked"},{"id":"gems","tier":"6","premium":false,"name":"Gems","icon":"gem","amount":"100","detail":"","state":"locked"}]}'>
<meta name="fixture:empty" content='{"rewards":[]}'>
<meta name="fixture:loading" content='{"loading":true}'>
<meta name="fixture:error" content='{"error":"The season service is unavailable. Your progress is saved."}'>
<meta name="fixture:premium" content='{"season":"Season of Embers","level":"50","premiumUnlocked":true,"progress":1,"progressText":"Season complete","rewards":[{"id":"skin","tier":"50","premium":true,"name":"Watcher of the Ninefold Gate","icon":"crown","amount":"×1","detail":"An exclusive outfit for completing this season’s premium track.","state":"available"},{"id":"chest","tier":"50","premium":false,"name":"Gold","icon":"coin","amount":"5,000","detail":"","state":"claimed"}]}'>

<!--
  A season pass: a kicker with the season and the time left over the
  title, a gold Get premium while premium is not owned, the level on a
  hexagon beside a leaning bar to the next tier, and two lanes of tier
  tiles, premium (in gold) over free. Each lane scrolls sideways on its
  own; on a narrow screen its name sits above it. A tile says its tier,
  shows the reward and how much, and ends in Claim when it is ready and
  owned, Claimed once taken, or Locked (a premium tile always reads
  Locked without premium). The game owns progress, entitlement and every
  reward's state; the track reports claim (a reward's key), upgrade and
  retry.
-->
<section id="PassTrack" part="PassTrack.frame" data-layout="shell" data-gap="loose" aria-label={title || t("meta.pass.title")} aria-busy={loading}>
  <div class="lit"></div>
  <header part="PassTrack.band" class="band" data-layout="cluster" data-gap="loose">
    <div class="heading" data-layout="stack" data-gap="hair" data-fill>
      {#if season || refresh}<span part="PassTrack.kicker" class="kicker">{season && refresh ? t("meta.pass.kicker", { season, refresh }) : season || refresh}</span>{/if}
      <h2 part="PassTrack.title">{title || t("meta.pass.title")}</h2>
    </div>
    {#if !premiumUnlocked && upgradeLabel}<button id="Upgrade" part="PassTrack.upgrade" class="upgrade" disabled={disabled || loading || !!error} onclick={upgrade}>{upgradeLabel}</button>{/if}
  </header>
  {#if loading}<div class="wait" data-layout="row" role="status"><Spinner name={t("meta.loading")} /><span>{t("meta.loading")}</span></div>
  {:else if error}<ErrorState body={error} />
  {:else if !rewards.length}<EmptyState title={t("meta.pass.empty")} icon="bag" />
  {:else}
    <div class="body" data-layout="scroll" data-gap="loose" data-fill>
    <div part="PassTrack.progress" class="progress" data-layout="row" data-gap="loose">
      {#if level}<div part="PassTrack.level" class="level" data-layout="cover" aria-label={t("meta.pass.level", { level })}><span>{level}</span></div>{/if}
      <div class="meter" data-layout="stack" data-gap="tight" data-fill>
        <div class="meter-line" data-layout="row" data-gap="tight"><span part="PassTrack.caption" class="key" data-fill>{t("meta.pass.progress")}</span><span part="PassTrack.figure" class="figure">{progressText || formatPercent(progress)}</span></div>
        <div part="PassTrack.track" class="bar" role="meter" aria-label={t("meta.pass.progress")} aria-valuetext={progressText || formatPercent(progress)} style:--fill={progress > 1 ? 1 : progress < 0 ? 0 : progress}><div part="PassTrack.fill" class="bar-fill"></div></div>
      </div>
    </div>
    <div part="PassTrack.rewards" class="lanes" role="group" aria-label={t("meta.pass.rewards")}>
      <div class="lane-h" data-lane="premium"><span>{t("meta.pass.premium")}</span>{#if !premiumUnlocked}<span part="PassTrack.caption" class="sub">{t("meta.locked")}</span>{/if}</div>
      <ol part="PassTrack.lane" class="lane" data-layout="row" data-gap="tight" data-lane="premium" aria-label={t("meta.pass.premiumRewards")}>
        {#each rewards as r (r.id)}{#if r.premium}<li part="PassTrack.reward" class="tile" data-layout="stack" data-gap="tight" data-premium={true} data-state={r.state} data-ready={r.state == 'available' && premiumUnlocked} aria-label={t("meta.pass.reward", { tier: r.tier, name: r.name, state: r.state == 'claimed' ? t("meta.claimed") : r.state == 'available' && premiumUnlocked ? t("meta.available") : t("meta.locked") })}>
          <span part="PassTrack.tier" class="tier">{t("meta.pass.tier", { tier: r.tier })}</span>
          <og-icon part="PassTrack.art" class="art" icon={r.icon}></og-icon>
          <span part="PassTrack.name" class="name">{r.name}</span>
          {#if r.amount}<span part="PassTrack.amount" class="amount">{r.amount}</span>{/if}
          {#if r.detail}<p part="PassTrack.detail" class="detail">{r.detail}</p>{/if}
          {#if r.state == 'claimed'}<div part="PassTrack.state" class="state" data-layout="row" data-gap="hair" data-kind="claimed"><og-icon class="mark" icon="check"></og-icon>{t("meta.claimed")}</div>
          {:else if r.state == 'available' && premiumUnlocked}<button part="PassTrack.claim" class="claim" variant="primary" disabled={disabled} onclick={claim}>{t("meta.claim")}</button>
          {:else}<div part="PassTrack.state" class="state" data-layout="row" data-gap="hair" data-kind="locked">{t("meta.locked")}</div>{/if}
        </li>{/if}{/each}
      </ol>
      <div class="lane-h" data-lane="free"><span>{t("meta.pass.free")}</span></div>
      <ol part="PassTrack.lane" class="lane" data-layout="row" data-gap="tight" data-lane="free" aria-label={t("meta.pass.freeRewards")}>
        {#each rewards as r (r.id)}{#if !r.premium}<li part="PassTrack.reward" class="tile" data-layout="stack" data-gap="tight" data-premium={false} data-state={r.state} data-ready={r.state == 'available'} aria-label={t("meta.pass.reward", { tier: r.tier, name: r.name, state: r.state == 'claimed' ? t("meta.claimed") : r.state == 'available' ? t("meta.available") : t("meta.locked") })}>
          <span part="PassTrack.tier" class="tier">{t("meta.pass.tier", { tier: r.tier })}</span>
          <og-icon part="PassTrack.art" class="art" icon={r.icon}></og-icon>
          <span part="PassTrack.name" class="name">{r.name}</span>
          {#if r.amount}<span part="PassTrack.amount" class="amount">{r.amount}</span>{/if}
          {#if r.detail}<p part="PassTrack.detail" class="detail">{r.detail}</p>{/if}
          {#if r.state == 'claimed'}<div part="PassTrack.state" class="state" data-layout="row" data-gap="hair" data-kind="claimed"><og-icon class="mark" icon="check"></og-icon>{t("meta.claimed")}</div>
          {:else if r.state == 'available'}<button part="PassTrack.claim" class="claim" variant="primary" disabled={disabled} onclick={claim}>{t("meta.claim")}</button>
          {:else}<div part="PassTrack.state" class="state" data-layout="row" data-gap="hair" data-kind="locked">{t("meta.locked")}</div>{/if}
        </li>{/if}{/each}
      </ol>
    </div>
    </div>
  {/if}
</section>

<style>
:host { inline-size: stretch; }
/* The frame: two corners cut, a lit top edge. */
[part="PassTrack.frame"] { position: relative; min-block-size: 0; clip-path: polygon(var(--inv-frame-cut) 0, 100% 0, 100% calc(100% - var(--inv-frame-cut)), calc(100% - var(--inv-frame-cut)) 100%, 0 100%, 0 var(--inv-frame-cut)); }
.lit { position: absolute; inset-inline: var(--inv-frame-cut) 30%; top: 0; block-size: var(--stroke-hairline); background: var(--inv-lit-edge); pointer-events: none; }
h2 { margin: 0; overflow-wrap: anywhere; }
.band { width: stretch; flex: none; align-items: flex-end; }
.heading { min-width: 0; flex-basis: calc(160 * var(--font-step-px)); }
.kicker { font-size: var(--inv-kicker-size); font-weight: var(--inv-name-weight); letter-spacing: var(--inv-kicker-tracking); text-transform: var(--inv-kicker-transform); color: var(--inv-kicker-ink); overflow-wrap: anywhere; }
/* Get premium: a gold face. */
.upgrade { flex: none; max-width: stretch; min-block-size: var(--size-touch); white-space: normal; --button-fill: var(--shop-premium-fill); --button-hover-fill: var(--shop-premium-hover-fill); --button-pressed-fill: var(--shop-premium-fill); --button-ink: var(--shop-premium-text); --button-shadow: var(--shop-premium-glow); }
/* The level on a hexagon, the bar to the next tier leaning beside it. */
.body { min-block-size: 0; width: stretch; }
.progress { flex: none; width: stretch; align-items: center; }
.level { flex: none; inline-size: var(--shop-level-size); block-size: var(--shop-level-size); font-size: var(--shop-level-figure-size); font-weight: var(--inv-figure-weight); font-stretch: var(--inv-figure-stretch); color: var(--inv-accent-ink); background: var(--shop-level-fill); clip-path: polygon(50% 0, 100% 25%, 100% 75%, 50% 100%, 0 75%, 0 25%); }
.meter { min-width: 0; }
.meter-line { width: stretch; align-items: baseline; flex-wrap: wrap; }
.key { min-width: 0; font-size: var(--inv-kicker-size); font-weight: var(--inv-name-weight); letter-spacing: var(--inv-kicker-tracking); text-transform: var(--inv-kicker-transform); color: var(--inv-kicker-ink); }
.figure { flex: none; white-space: nowrap; font-weight: var(--inv-figure-weight); font-stretch: var(--inv-figure-stretch); font-variant-numeric: var(--hud-figure-numeric); }
.bar { position: relative; width: stretch; margin-inline: var(--space-xs); block-size: calc(10 * var(--font-step-px)); transform: skewX(var(--bar-skew)); background: var(--bar-well); box-shadow: var(--bar-rim); }
.bar-fill { position: absolute; inset-inline-start: 0; top: 0; bottom: 0; inline-size: calc(var(--fill, 0) * 100%); background: var(--inv-load-fill); }
/* Two lanes, each its name then its tiles scrolling sideways; narrow,
   the name sits over its lane. */
.lanes { display: grid; grid-template-columns: var(--shop-lane-width) minmax(0, 1fr); column-gap: var(--space-sm); row-gap: var(--space-sm); width: stretch; min-block-size: 0; }
.lane-h { display: flex; flex-direction: column; justify-content: center; gap: var(--space-xxs); padding-inline-start: var(--space-sm); font-weight: var(--inv-figure-weight); color: var(--ink-label); border-inline-start: var(--shop-free-rule); }
.lane-h[data-lane=premium] { color: var(--shop-premium-ink); border-inline-start: var(--shop-premium-rule); }
.sub { color: var(--inv-kicker-ink); }
.lane { min-width: 0; margin: 0; padding: var(--space-xxs); list-style: none; overflow-x: auto; overflow-y: hidden; scrollbar-width: none; scroll-snap-type: x proximity; mask-image: var(--shop-lane-fade); padding-inline-end: var(--size-lg); }
/* A tile: cut at two corners; premium washed and rimmed in gold; ready
   ringed in the family's light. */
.tile { flex: 1 0 var(--shop-tier-width); min-inline-size: var(--shop-tier-width); align-items: center; padding: var(--space-xs); scroll-snap-align: start; text-align: center; background: var(--shop-tile-fill); box-shadow: var(--shop-tile-rim); clip-path: polygon(var(--inv-slot-cut) 0, 100% 0, 100% calc(100% - var(--inv-slot-cut)), calc(100% - var(--inv-slot-cut)) 100%, 0 100%, 0 var(--inv-slot-cut)); }
.tile[data-premium=true] { background: var(--shop-premium-tile); box-shadow: var(--shop-premium-rim); }
.tile[data-ready=true] { box-shadow: var(--shop-ready-rim); }
.tier { align-self: flex-start; font-size: var(--inv-tag-size); font-weight: var(--inv-figure-weight); letter-spacing: var(--inv-kicker-tracking); text-transform: var(--inv-kicker-transform); color: var(--inv-kicker-ink); white-space: nowrap; }
.art { flex: none; inline-size: var(--shop-tier-art-size); block-size: var(--shop-tier-art-size); filter: var(--inv-slot-art-shadow); }
.tile[data-state=claimed] .art { opacity: var(--shop-tier-claimed-opacity); }
.tile[data-ready=false]:not([data-state=claimed]) .art { filter: var(--shop-art-refused); }
.name { max-width: stretch; font-weight: var(--inv-name-weight); color: var(--ink-bright); overflow-wrap: anywhere; }
.detail { margin: 0; max-width: stretch; font-size: var(--inv-tag-size); color: var(--ink-label); overflow-wrap: anywhere; }
.amount { font-weight: var(--inv-figure-weight); font-stretch: var(--inv-figure-stretch); color: var(--ink-bright); white-space: nowrap; }
.state { align-items: center; margin-block-start: auto; padding: var(--space-xxs) var(--space-xs); font-size: var(--inv-tag-size); font-weight: var(--inv-figure-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--inv-kicker-transform); white-space: nowrap; color: var(--shop-sold-ink); background: var(--shop-sold-fill); clip-path: polygon(var(--inv-tag-cut) 0, 100% 0, 100% calc(100% - var(--inv-tag-cut)), calc(100% - var(--inv-tag-cut)) 100%, 0 100%, 0 var(--inv-tag-cut)); }
.state[data-kind=claimed] { color: var(--shop-owned-ink); background: var(--shop-owned-fill); }
.mark { flex: none; inline-size: var(--inv-tag-size); block-size: var(--inv-tag-size); }
.claim { margin-block-start: auto; width: stretch; min-block-size: var(--size-touch); white-space: normal; }
.wait { padding: var(--space-lg); }
@container (width < 600px) {
  .heading { flex-basis: 100%; }
  .upgrade { width: stretch; }
  .lanes { grid-template-columns: minmax(0, 1fr); }
  .lane-h { flex-direction: row; justify-content: flex-start; align-items: baseline; gap: var(--space-xs); padding-block: var(--space-xxs); }
}
</style>
