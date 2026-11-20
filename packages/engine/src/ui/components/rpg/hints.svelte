<meta name="component" content="RpgHints">
<meta name="strings" content="./rpg.en.ogt">
<meta name="prop:hints" content="list" data-doc="The inputs worth a reminder, keyed rows of { id, key, label }: key the input as the game names it (Tab, Space) on a key cap, label what it does (Target). Under a touch screen the hints are hidden: the controls on screen say it.">
<meta name="fixture:default" content='{"hints":[{"id":"target","key":"Tab","label":"Target"},{"id":"dodge","key":"Space","label":"Dodge"}]}'>
<meta name="fixture:empty" content='{"hints":[]}'>
<meta name="fixture:overflow" content='{"hints":[{"id":"a","key":"Left Shoulder + Right Shoulder","label":"Switch the locked target to the next enemy in sight"},{"id":"b","key":"B","label":"Dodge"}]}'>

<!--
  Input hints at the screen's foot: a row of HudHint blocks, each a key
  cap and what it does in small tracked capitals. The row only spaces
  them; a game lays HudHint out itself anywhere else. Read only; hidden
  under a touch screen.
-->
<ul id="RpgHints" part="frame" variant="hud" layout="wrap" aria-label={t("rpg.hints.name")}>
  {#each hints as h (h.id)}
  <li part="hint" class="hint" layout="row"><HudHint key={h.key} label={h.label} /></li>
  {/each}
</ul>

<style>
#RpgHints { margin: 0; padding: 0; list-style: none; justify-content: flex-end; column-gap: calc(16 * var(--font-step-px)); max-inline-size: stretch; color: var(--hud-ink); }
.hint { min-inline-size: 0; max-inline-size: 100%; }
@media (pointer: coarse) { #RpgHints { display: none; } }
</style>
