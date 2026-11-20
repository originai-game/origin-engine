<meta name="component" content="QtePrompt">
<meta name="strings" content="./qte-prompt.en.ogt">
<meta name="prop:label" content="string" data-default="" data-doc="The action to attempt; empty uses Respond.">
<meta name="prop:keyLabel" content="string" data-default="" data-doc="The displayed input hint, not an input binding; empty hides it.">
<meta name="prop:action" content="string" data-default="" data-doc="An og-input action bound by the game. Pointer, focused keyboard/pad and this action activate the same control.">
<meta name="prop:mode" content="string" data-default="press" data-doc="press, hold or mash. Hold emits once after holdMs; mash emits one respond per activation. The game counts responses and judges success.">
<meta name="prop:holdMs" content="number" data-default="600" data-doc="The minimum hold duration in milliseconds, at least 1. Releasing early cancels without an intent.">
<meta name="prop:remaining" content="number" data-default="1" data-doc="Time left as a fraction, 0 to 1. The game advances it; no clock runs here and zero does not choose a result.">
<meta name="prop:progress" content="number" data-default="0" data-doc="Game-supplied completion fraction, 0 to 1; displayed for mash prompts.">
<meta name="prop:state" content="string" data-default="active" data-doc="active, waiting, success, failure or expired. Only active accepts input; the game owns resolution and expiry.">
<meta name="prop:disabled" content="bool" data-default="false" data-doc="Temporarily unavailable even while active.">
<meta name="action:respond" content="Respond">
<meta name="fixture:default" content='{"label":"Evade the falling gate","keyLabel":"Space","action":"","mode":"press","holdMs":600,"remaining":0.72,"progress":0,"state":"active","disabled":false}'>
<meta name="fixture:hold" content='{"label":"Brace the gate","keyLabel":"E","mode":"hold","holdMs":600,"remaining":0.6}'>
<meta name="fixture:mash" content='{"label":"Break free","keyLabel":"Space","mode":"mash","progress":0.4,"remaining":0.35}'>
<meta name="fixture:success" content='{"state":"success","remaining":0.2}'>
<meta name="fixture:failure" content='{"state":"failure","remaining":0.1}'>
<meta name="fixture:expired" content='{"state":"expired","remaining":0}'>
<meta name="fixture:waiting" content='{"state":"waiting"}'>
<meta name="fixture:disabled" content='{"disabled":true}'>
<meta name="fixture:overflow" content='{"label":"Brace the damaged northern observatory mechanism","keyLabel":"Right trigger","mode":"hold"}'>

<!-- respond carries the enclosing keyed prompt's id, or the stable mount
     instance for a single prompt. Game logic decides success and expiry. -->
<section id="QtePrompt" part="frame" variant="hud" layout="stack" data-state={state} aria-label={label || t("hud.qte-prompt.respond")} aria-busy={state == 'waiting'}>
  <div class="label" part="headline">{label || t("hud.qte-prompt.respond")}</div>
  {#if mode == 'hold'}
    <button class="trigger" id="Hold" part="trigger" data-hold={holdMs > 0 ? holdMs : 1} data-action={action} disabled={disabled || state != 'active'} onhold={respond} aria-label={label || t("hud.qte-prompt.respond")}>
      <div class="trigger-face" layout="wrap">
      {#if keyLabel}<div class="key" part="figure">{keyLabel}</div>{/if}<div class="instruction" part="instruction">{t("hud.qte-prompt.hold")}</div>
      </div>
    </button>
  {:else}
    <button class="trigger" id="Press" part="trigger" data-action={action} disabled={disabled || state != 'active'} onclick={respond} aria-label={label || t("hud.qte-prompt.respond")}>
      <div class="trigger-face" layout="wrap">
      {#if keyLabel}<div class="key" part="figure">{keyLabel}</div>{/if}<div class="instruction" part="instruction">{mode == 'mash' ? t("hud.qte-prompt.mash") : t("hud.qte-prompt.press")}</div>
      </div>
    </button>
  {/if}
  <div class="state" part="caption" data-tone={state == 'success' ? 'success' : state == 'failure' || state == 'expired' ? 'danger' : null} role="status">{state == 'success' ? t("hud.qte-prompt.success") : state == 'failure' ? t("hud.qte-prompt.failure") : state == 'expired' ? t("hud.qte-prompt.expired") : state == 'waiting' ? t("hud.qte-prompt.waiting") : disabled ? t("hud.qte-prompt.disabled") : t("hud.qte-prompt.active")}</div>
  <div class="timeLabel" part="caption">{t("hud.qte-prompt.time")}</div>
  <progress class="timer" id="Timer" part="timer" value={remaining < 0 ? 0 : remaining > 1 ? 1 : remaining} aria-label={t("hud.qte-prompt.time")}></progress>
  {#if mode == 'mash'}<div class="progressLabel" part="caption">{t("hud.qte-prompt.progress")}</div><progress class="progress" id="Progress" part="progress" value={progress < 0 ? 0 : progress > 1 ? 1 : progress} aria-label={t("hud.qte-prompt.progress")}></progress>{/if}
</section>

<style>
#QtePrompt { max-inline-size: 100%; }
.label { overflow-wrap: anywhere; }
.trigger { justify-content: center; min-block-size: var(--touch-target); padding: var(--space-sm) var(--space-md); }
.trigger-face { justify-content: center; }
.instruction { overflow-wrap: anywhere; }
.timer, .progress { inline-size: 100%; }
</style>
