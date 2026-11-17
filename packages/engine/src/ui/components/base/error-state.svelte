<meta name="component" content="ErrorState">
<meta name="strings" content="./states.en.ogt">
<meta name="parts" content="ErrorState.frame ErrorState.mark ErrorState.title ErrorState.body ErrorState.code ErrorState.actions ErrorState.retry ErrorState.dismiss ErrorState.headline ErrorState.caption">
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

<section id="ErrorState" part="ErrorState.frame" data-layout="stack" role="alert" aria-busy={busy} aria-label={title || t("base.error.title")}>
  <og-icon part="ErrorState.mark" class="mark" data-tone="danger" icon="close"></og-icon>
  <h3 part="ErrorState.title ErrorState.headline">{title || t("base.error.title")}</h3>
  {#if body}<p part="ErrorState.body" class="body">{body}</p>{/if}
  {#if code}<p part="ErrorState.code ErrorState.caption" class="code">{code}</p>{/if}
  {#if canRetry || dismissLabel}<div part="ErrorState.actions" class="actions" data-layout="cluster">
    {#if canRetry}<button id="Retry" part="ErrorState.retry" variant="primary" disabled={busy} onclick={retry}>{#if busy}<Spinner size={16} name={t("base.error.busy")} />{/if}<span>{busy ? t("base.error.busy") : retryLabel || t("base.error.retry")}</span></button>{/if}
    {#if dismissLabel}<button id="Dismiss" part="ErrorState.dismiss" disabled={busy} onclick={dismiss}>{dismissLabel}</button>{/if}
  </div>{/if}
</section>

<style>
:host { align-items: center; padding: var(--space-xl); text-align: center; }
.mark { inline-size: var(--size-lg); block-size: var(--size-lg); }
[part~="ErrorState.title"] { margin: 0; --headline-text: var(--text-lg); }
.body, .code { margin: 0; align-self: center; max-inline-size: var(--measure-lg); text-align: center; overflow-wrap: anywhere; }
.actions { justify-content: center; margin-top: var(--space-xs); }
.actions button { min-block-size: var(--touch-target); max-inline-size: 100%; white-space: normal; }
</style>
