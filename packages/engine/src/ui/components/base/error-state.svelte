<meta name="component" content="ErrorState">
<meta name="strings" content="./states.en.ogt">
<meta name="parts" content="ErrorState.frame ErrorState.disc ErrorState.dot ErrorState.mark ErrorState.headline ErrorState.body ErrorState.code ErrorState.badge ErrorState.actions ErrorState.retry ErrorState.dismiss">
<meta name="prop:title" content="string" data-default="" data-doc="Heading; empty uses Something went wrong.">
<meta name="prop:body" content="string" data-default="" data-doc="The game's explanation and recovery advice.">
<meta name="prop:code" content="string" data-default="" data-doc="Optional support reference; never a raw exception or credential.">
<meta name="prop:retryLabel" content="string" data-default="" data-doc="Retry wording; empty uses Try again.">
<meta name="prop:canRetry" content="bool" data-default="true" data-doc="Offer the retry action.">
<meta name="prop:busy" content="bool" data-default="false" data-doc="Retry is in progress; both actions are disabled and a wait is announced.">
<meta name="prop:dismissLabel" content="string" data-default="" data-doc="Optional secondary action; empty hides it.">
<meta name="action:retry" content="Retry the failed operation">
<meta name="action:dismiss" content="Leave the error state">
<meta name="fixture:default" content='{"title":"Could not load your builds","body":"Your saved builds are safe. Check your connection and try again.","code":"NET-204","canRetry":true,"busy":false,"dismissLabel":"Go back"}'>
<meta name="fixture:busy" content='{"body":"Connecting to the server…","busy":true,"dismissLabel":"Go back"}'>
<meta name="fixture:terminal" content='{"title":"This replay is no longer available","body":"It was recorded on an older game version.","canRetry":false,"dismissLabel":"Return to replays"}'>
<meta name="fixture:overflow" content='{"title":"We could not retrieve the shared expedition report","body":"The host may have left the session. Your completed objectives are saved, and you can return to the lobby without losing rewards.","code":"EXPEDITION-REPORT-UNAVAILABLE","dismissLabel":"Return to the expedition lobby"}'>

<!-- EmptyState's recess with the failure on a disc tinted and ringed in
     danger (quieter than the action it leads to): what went
     wrong, the support reference set apart as a code, then retry (the
     primary action) and the way out. -->
<section id="ErrorState" part="ErrorState.frame" data-layout="stack" data-motion="enter" role="alert" aria-busy={busy} aria-label={title || t("base.error.title")}>
  <div part="ErrorState.disc ErrorState.dot" class="disc" data-layout="cover" data-motion="pop"><og-icon part="ErrorState.mark" class="mark" data-tone="danger" icon="close"></og-icon></div>
  <div class="words" data-layout="stack" data-gap="tight">
    <h3 part="ErrorState.headline" class="title">{title || t("base.error.title")}</h3>
    {#if body}<p part="ErrorState.body" class="body">{body}</p>{/if}
    {#if code}<p part="ErrorState.code ErrorState.badge" class="code">{code}</p>{/if}
  </div>
  {#if canRetry || dismissLabel}<div part="ErrorState.actions" class="actions" data-layout="cluster">
    {#if canRetry}<button id="Retry" part="ErrorState.retry" variant="primary" disabled={busy} onclick={retry}>{#if busy}<Spinner size={16} name={t("base.error.busy")} />{/if}<span class="retry">{busy ? t("base.error.busy") : retryLabel || t("base.error.retry")}</span></button>{/if}
    {#if dismissLabel}<button id="Dismiss" part="ErrorState.dismiss" disabled={busy} onclick={dismiss}>{dismissLabel}</button>{/if}
  </div>{/if}
</section>

<style>
:host { align-items: center; padding: var(--space-xl); text-align: center; --layout-gap: var(--layout-gap-wide); --frame-fill: var(--well-fill); --frame-shadow: var(--well-shadow); --frame-rim-width: 0px; --frame-radius: var(--card-radius); }
.disc { flex: none; inline-size: var(--size-xl); block-size: var(--size-xl); --dot-fill: var(--dot-alert-fill); --dot-rim: var(--dot-alert-rim); --dot-rim-width: var(--stroke-thin); }
.mark { inline-size: var(--size-md); block-size: var(--size-md); }
.words { align-items: center; max-inline-size: var(--measure-lg); }
.title { margin: 0; --headline-text: var(--text-lg); }
.body { margin: 0; text-align: center; overflow-wrap: anywhere; }
.code { align-self: center; inline-size: fit-content; margin: var(--space-xs) 0 0; max-inline-size: 100%; overflow-wrap: anywhere; --badge-radius: var(--radius-control); }
.actions { justify-content: center; }
.actions button { min-block-size: var(--touch-target); max-inline-size: 100%; white-space: normal; }
</style>
