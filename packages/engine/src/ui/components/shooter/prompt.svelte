<meta name="component" content="ShooterPrompt">
<meta name="strings" content="./shooter.en.ogt">
<meta name="prop:key" content="string" data-doc="The input that does it as the game names it (E, X, Hold F); empty hides the key cap.">
<meta name="prop:text" content="string" data-doc="What it does as the game words it (Open supply bin); empty hides the prompt.">
<meta name="prop:disabled" content="bool" data-doc="It cannot be done now: the prompt dims and the reason shows.">
<meta name="prop:reason" content="string" data-doc="Why it cannot (Locked); shown while disabled.">
<meta name="fixture:default" content='{"key":"E","text":"Open supply bin","disabled":false,"reason":""}'>
<meta name="fixture:disabled" content='{"key":"E","text":"Open armory door","disabled":true,"reason":"Needs a keycard"}'>
<meta name="fixture:empty" content='{"key":"","text":"","disabled":false,"reason":""}'>
<meta name="fixture:overflow" content='{"key":"Hold Left Trigger","text":"Pick up the extended heavy magazine for the Northern Relay marksman rifle","disabled":false,"reason":""}'>

<!--
  What the player can do with what is in front of them, under the aim: the
  key cap and the action in words. The game decides when it shows and
  what the input does; read only.
-->
<section id="ShooterPrompt" part="frame" variant="hud" layout="row" gap="tight" role="status" aria-disabled={disabled} class:disabled={disabled}>
  {#if text != ''}
  {#if key != ''}<span part="key" class="key">{key}</span>{/if}
  <span part="action" class="action">{text}</span>
  {#if disabled && reason != ''}<span part="reason" class="reason">{reason}</span>{/if}
  {/if}
</section>

<style>
#ShooterPrompt { align-items: center; justify-content: center; max-inline-size: stretch; color: var(--hud-ink); text-shadow: var(--hud-shadow); -webkit-text-stroke-width: var(--hud-stroke-width); -webkit-text-stroke-color: var(--hud-stroke-color); paint-order: stroke fill; }
.key { flex: 0 1 auto; min-inline-size: calc(22 * var(--font-step-px)); max-inline-size: 40%; padding: 0 var(--space-xs); font-size: var(--hud-tag-size); line-height: 1.5; font-weight: var(--hud-figure-weight); text-align: center; color: var(--hud-key-ink); background: var(--hud-key-fill); border-radius: var(--hud-key-radius); box-shadow: var(--hud-key-shadow); -webkit-text-stroke-width: 0; text-shadow: none; white-space: nowrap; text-overflow: -og-scroll var(--motion-duration-orbit) var(--motion-duration-turn); overflow: hidden; }
.action { min-inline-size: 0; font-size: var(--hud-size-sm); font-weight: var(--hud-label-weight); white-space: nowrap; text-overflow: -og-scroll var(--motion-duration-orbit) var(--motion-duration-turn); overflow: hidden; }
.reason { flex: none; font-size: var(--hud-label-size); font-weight: var(--hud-label-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); color: var(--hud-foe); white-space: nowrap; }
.disabled .action, .disabled .key { opacity: 0.55; }
.action:size(compact) { font-size: var(--hud-size-xs); }
</style>
