<meta name="component" content="HudHint">
<meta name="prop:key" content="string" data-doc="The input as the game names it (Tab, Space), on a key cap; empty is none.">
<meta name="prop:label" content="string" data-doc="What it does (Target), in small tracked capitals.">
<meta name="fixture:default" content='{"key":"Tab","label":"Target"}'>
<meta name="fixture:empty" content='{"key":"","label":""}'>
<meta name="fixture:overflow" content='{"key":"Left Shoulder + Right Shoulder","label":"Switch the locked target to the next enemy in sight"}'>

<!--
  One input hint of a HUD, a block a hint row is built from: a key
  cap and what it does in small tracked capitals. Read only; hidden under
  a touch screen, where the controls on screen say it.
-->
<div id="HudHint" part="hint" layout="row" gap="tight">
  {#if key}<span part="key" class="key">{key}</span>{/if}
  <span part="action" class="action">{label}</span>
</div>

<style>
#HudHint { inline-size: fit-content; align-items: center; min-inline-size: 0; max-inline-size: 100%; color: var(--hud-ink); }
.key { flex: 0 1 auto; min-inline-size: calc(24 * var(--font-step-px)); max-inline-size: calc(120 * var(--font-step-px)); padding: 0 var(--space-xs); font-size: var(--hud-tag-size); line-height: 1.6; font-weight: var(--hud-figure-weight); text-align: center; color: var(--hud-key-ink); background: var(--hud-key-fill); border-radius: var(--hud-key-radius); box-shadow: var(--hud-key-shadow); white-space: nowrap; text-overflow: -og-scroll var(--motion-duration-orbit) var(--motion-duration-turn); overflow: hidden; }
.action { min-inline-size: 0; font-size: var(--hud-label-size); font-weight: var(--hud-label-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); color: var(--hud-label-ink); white-space: nowrap; text-overflow: -og-scroll var(--motion-duration-orbit) var(--motion-duration-turn); overflow: hidden; text-shadow: var(--hud-shadow); -webkit-text-stroke-width: var(--hud-stroke-width); -webkit-text-stroke-color: var(--hud-stroke-color); paint-order: stroke fill; }
#HudHint:touch { display: none; }
</style>
