<meta name="component" content="RpgPrompt">
<meta name="strings" content="./rpg.en.ogt">
<meta name="parts" content="RpgPrompt.frame RpgPrompt.key RpgPrompt.action RpgPrompt.reason">
<meta name="prop:key" content="string" data-doc="The input that does it as the game names it (F, Hold E); empty hides the key cap.">
<meta name="prop:text" content="string" data-doc="What it does as the game words it (Finisher: Sunder the guard); empty hides the prompt.">
<meta name="prop:disabled" content="bool" data-doc="It cannot be done now: the prompt dims and the reason shows.">
<meta name="prop:reason" content="string" data-doc="Why it cannot (Guard holds); shown while disabled.">
<meta name="fixture:default" content='{"key":"F","text":"Finisher: Sunder the guard","disabled":false,"reason":""}'>
<meta name="fixture:disabled" content='{"key":"F","text":"Finisher: Sunder the guard","disabled":true,"reason":"Guard holds"}'>
<meta name="fixture:empty" content='{"key":"","text":"","disabled":false,"reason":""}'>
<meta name="fixture:overflow" content='{"key":"Hold Left Trigger + Right Trigger","text":"Finisher: Sunder the guard of the Ashen Colossus, Warden of the Ninefold Gate","disabled":false,"reason":""}'>

<!--
  What the player can do now, over the ability bar: a key cap and the
  action in words (a finisher, an interaction). The game decides when it
  shows and what the input does; read only.
-->
<section id="RpgPrompt" part="RpgPrompt.frame" class="hud bare" data-layout="row" data-gap="tight" role="status" aria-disabled={disabled} class:disabled={disabled}>
  {#if text != ''}
  {#if key != ''}<span part="RpgPrompt.key" class="key">{key}</span>{/if}
  <span part="RpgPrompt.action" class="action">{text}</span>
  {#if disabled && reason != ''}<span part="RpgPrompt.reason" class="reason">{reason}</span>{/if}
  {/if}
</section>

<style>
:host { align-items: center; justify-content: center; max-inline-size: stretch; color: var(--hud-ink); text-shadow: var(--hud-shadow); -webkit-text-stroke-width: var(--hud-stroke-width); -webkit-text-stroke-color: var(--hud-stroke-color); paint-order: stroke fill; }
.key { flex: 0 1 auto; min-inline-size: calc(24 * var(--font-step-px)); max-inline-size: 40%; padding: 0 var(--space-xs); font-size: var(--hud-tag-size); line-height: 1.6; font-weight: var(--hud-figure-weight); text-align: center; color: var(--hud-key-ink); background: var(--hud-key-fill); border-radius: var(--hud-key-radius); box-shadow: var(--hud-key-shadow); -webkit-text-stroke-width: 0; text-shadow: none; white-space: nowrap; text-overflow: -og-scroll var(--motion-duration-orbit) var(--motion-duration-turn); overflow: hidden; }
.action { min-inline-size: 0; font-size: var(--hud-size-title); font-weight: var(--hud-label-weight); white-space: nowrap; text-overflow: -og-scroll var(--motion-duration-orbit) var(--motion-duration-turn); overflow: hidden; }
.reason { flex: none; font-size: var(--hud-label-size); font-weight: var(--hud-label-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); color: var(--hud-harm); white-space: nowrap; }
.disabled .action, .disabled .key { opacity: 0.55; }
@media (pointer: coarse) { .key { display: none; } }
@media (width < 600px), (height < 500px) { .action { font-size: var(--hud-size-xs); } }
</style>
