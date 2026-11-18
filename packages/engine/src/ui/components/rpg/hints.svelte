<meta name="component" content="RpgHints">
<meta name="strings" content="./rpg.en.ogt">
<meta name="parts" content="RpgHints.frame RpgHints.hint RpgHints.key RpgHints.action">
<meta name="prop:hints" content="list" data-doc="The inputs worth a reminder, keyed rows of { id, key, label }: key the input as the game names it (Tab, Space) on a key cap, label what it does (Target). Under a touch screen the hints are hidden: the controls on screen say it.">
<meta name="fixture:default" content='{"hints":[{"id":"target","key":"Tab","label":"Target"},{"id":"dodge","key":"Space","label":"Dodge"}]}'>
<meta name="fixture:empty" content='{"hints":[]}'>
<meta name="fixture:overflow" content='{"hints":[{"id":"a","key":"Left Shoulder + Right Shoulder","label":"Switch the locked target to the next enemy in sight"},{"id":"b","key":"B","label":"Dodge"}]}'>

<!--
  Input hints at the screen's foot: a key cap and what it does, small
  tracked capitals. Read only; hidden under a touch screen.
-->
<ul id="RpgHints" part="RpgHints.frame" class="hud bare" data-layout="cluster" aria-label={t("rpg.hints.name")}>
  {#each hints as h (h.id)}
  <li part="RpgHints.hint" class="hint" data-layout="row" data-gap="tight">
    <span part="RpgHints.key" class="key">{h.key}</span>
    <span part="RpgHints.action" class="action">{h.label}</span>
  </li>
  {/each}
</ul>

<style>
:host { margin: 0; padding: 0; list-style: none; justify-content: flex-end; column-gap: calc(16 * var(--font-step-px)); max-inline-size: stretch; color: var(--hud-ink); }
.hint { align-items: center; min-inline-size: 0; max-inline-size: 100%; }
.key { flex: 0 1 auto; min-inline-size: calc(24 * var(--font-step-px)); max-inline-size: calc(120 * var(--font-step-px)); padding: 0 var(--space-xs); font-size: var(--hud-tag-size); line-height: 1.6; font-weight: var(--hud-figure-weight); text-align: center; color: var(--hud-key-ink); background: var(--hud-key-fill); border-radius: var(--hud-key-radius); box-shadow: var(--hud-key-shadow); white-space: nowrap; text-overflow: -og-scroll var(--motion-duration-orbit) var(--motion-duration-turn); overflow: hidden; }
.action { min-inline-size: 0; font-size: var(--hud-label-size); font-weight: var(--hud-label-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); color: var(--hud-label-ink); white-space: nowrap; text-overflow: -og-scroll var(--motion-duration-orbit) var(--motion-duration-turn); overflow: hidden; text-shadow: var(--hud-shadow); -webkit-text-stroke-width: var(--hud-stroke-width); -webkit-text-stroke-color: var(--hud-stroke-color); paint-order: stroke fill; }
@media (pointer: coarse) { :host { display: none; } }
</style>
