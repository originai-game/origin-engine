<meta name="component" content="PassTrack">
<meta name="strings" content="./meta.en.ogt">
<meta name="parts" content="PassTrack.frame PassTrack.band PassTrack.title PassTrack.season PassTrack.refresh PassTrack.progress PassTrack.rewards PassTrack.reward PassTrack.tier PassTrack.lane PassTrack.well PassTrack.icon PassTrack.name PassTrack.amount PassTrack.detail PassTrack.state PassTrack.claim PassTrack.upgrade PassTrack.caption PassTrack.card PassTrack.headline PassTrack.figure">
<meta name="prop:title" content="string" data-default="" data-doc="Heading; empty uses Season pass.">
<meta name="prop:season" content="string" data-default="" data-doc="The game's season name.">
<meta name="prop:refresh" content="string" data-default="" data-doc="The game's already formatted time remaining; no clock runs here.">
<meta name="prop:progress" content="number" data-default="0" data-doc="Overall progress, 0 to 1, supplied by the game.">
<meta name="prop:progressText" content="string" data-default="" data-doc="Progress words, such as Level 8 · 240 / 500 XP; empty uses a percent.">
<meta name="prop:premiumUnlocked" content="bool" data-default="false" data-doc="The game owns the premium entitlement.">
<meta name="prop:upgradeLabel" content="string" data-default="" data-doc="Optional upgrade action while premium is not unlocked; no purchase logic.">
<meta name="prop:rewards" content="list" data-default="[]" data-doc="Keyed rewards in the game's order: { id, tier, premium, name, icon, amount, detail, state }. state locked/available/claimed. Each reward is one tile, and lane/tier are always written in words.">
<meta name="prop:loading" content="bool" data-default="false" data-doc="Loading the track; no reward action is available.">
<meta name="prop:error" content="string" data-default="" data-doc="A failure explanation with retry; empty is no error.">
<meta name="prop:disabled" content="bool" data-default="false" data-doc="Prevent claims and upgrades while the game processes an operation.">
<meta name="action:claim" content="Claim the available reward by stable key">
<meta name="action:upgrade" content="Inspect the premium upgrade">
<meta name="action:retry" content="Retry loading the track">
<meta name="fixture:default" content='{"title":"","season":"Season of Embers","refresh":"Ends in 12 days","progress":0.4,"progressText":"Level 4 · 240 / 500 XP","premiumUnlocked":false,"upgradeLabel":"View premium pass","rewards":[{"id":"gold","tier":"1","premium":false,"name":"Gold","icon":"coin","amount":"500","detail":"","state":"claimed"},{"id":"keys","tier":"4","premium":false,"name":"Chest keys","icon":"key","amount":"3","detail":"","state":"available"},{"id":"blade","tier":"4","premium":true,"name":"Ember Blade","icon":"sword","amount":"1","detail":"Legendary weapon skin","state":"available"},{"id":"gems","tier":"5","premium":false,"name":"Gems","icon":"gem","amount":"100","detail":"","state":"locked"}]}'>
<meta name="fixture:empty" content='{"rewards":[]}'>
<meta name="fixture:loading" content='{"loading":true}'>
<meta name="fixture:error" content='{"error":"The season service is unavailable. Your progress is saved."}'>
<meta name="fixture:premium" content='{"premiumUnlocked":true,"progress":1,"progressText":"Season complete","rewards":[{"id":"skin","tier":"50","premium":true,"name":"Watcher of the Ninefold Gate","icon":"crown","amount":"1","detail":"An exclusive outfit for completing this season’s premium track.","state":"available"}]}'>

<section id="PassTrack" part="PassTrack.frame" data-layout="stack" data-gap="loose" aria-label={title || t("meta.pass.title")} aria-busy={loading}>
  <header part="PassTrack.band" class="band" data-layout="cluster" data-gap="loose"><div class="heading" data-layout="stack" data-gap="hair" data-fill><h2 part="PassTrack.title">{title || t("meta.pass.title")}</h2>{#if season}<span part="PassTrack.season PassTrack.caption" class="muted">{season}</span>{/if}</div>{#if refresh}<span part="PassTrack.refresh PassTrack.caption" class="muted">{refresh}</span>{/if}{#if !premiumUnlocked && upgradeLabel}<button id="Upgrade" part="PassTrack.upgrade" disabled={disabled || loading || !!error} onclick={upgrade}>{upgradeLabel}</button>{/if}</header>
  {#if loading}<div class="wait" data-layout="row" role="status"><Spinner name={t("meta.loading")} /><span>{t("meta.loading")}</span></div>
  {:else if error}<ErrorState body={error} />
  {:else if !rewards.length}<EmptyState title={t("meta.pass.empty")} icon="bag" />
  {:else}
    <div part="PassTrack.progress"><ProgressBar value={progress} label={t("meta.pass.progress")} valueText={progressText} showValue={true} /></div>
    <ol part="PassTrack.rewards" class="track" data-layout="row" aria-label={t("meta.pass.rewards")}>
      {#each rewards as reward (reward.id)}<li part="PassTrack.reward PassTrack.card" class="reward" data-layout="stack" data-gap="tight" data-state={reward.state} data-premium={reward.premium}>
        <div class="tier" data-layout="cluster" data-gap="tight"><div part="PassTrack.tier PassTrack.caption">{t("meta.pass.tier", { tier: reward.tier })}</div><div part="PassTrack.lane PassTrack.caption" class="muted">{reward.premium ? t("meta.pass.premium") : t("meta.pass.free")}</div></div>
        <div part="PassTrack.well" class="well" data-layout="row" data-gap="none"><og-icon part="PassTrack.icon" class="icon" icon={reward.icon}></og-icon></div>
        <h3 part="PassTrack.name PassTrack.headline">{reward.name}</h3>{#if reward.amount}<span part="PassTrack.amount PassTrack.figure" class="amount">{reward.amount}</span>{/if}{#if reward.detail}<p part="PassTrack.detail PassTrack.caption" class="muted">{reward.detail}</p>{/if}
        <span part="PassTrack.state PassTrack.caption" class="state" data-tone={reward.state == 'claimed' ? 'success' : ''} data-state={reward.state}>{reward.state == 'claimed' ? t("meta.claimed") : reward.premium && !premiumUnlocked ? t("meta.pass.requiresPremium") : reward.state == 'available' ? t("meta.available") : t("meta.locked")}</span>
        <button part="PassTrack.claim" disabled={disabled || reward.state != 'available' || (reward.premium && !premiumUnlocked)} onclick={claim}>{t("meta.claim")}</button>
      </li>{/each}
    </ol>
  {/if}
</section>

<style>
:host { inline-size: stretch; }
.heading { flex-basis: var(--measure-xs); }
h2, h3, p { margin: 0; overflow-wrap: anywhere; }
.muted { overflow-wrap: anywhere; }
.track { align-items: stretch; overflow-x: auto; scroll-snap-type: x proximity; margin: 0; padding: var(--space-xs); list-style: none; }
.reward { flex: 0 0 var(--measure-xs); max-inline-size: 100%; scroll-snap-align: start; }
.tier { justify-content: space-between; }
.well { justify-content: center; block-size: var(--size-xl); }
.icon { inline-size: var(--size-lg); block-size: var(--size-lg); }
.state { margin-top: auto; }
.wait { padding: var(--space-lg); }
button { min-block-size: var(--touch-target); white-space: normal; }
</style>
