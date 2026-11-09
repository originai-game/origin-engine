<meta name="component" content="RpgTracker">
<meta name="strings" content="./rpg.en.ogt">
<meta name="parts" content="Tracker.frame Tracker.objective">
<meta name="prop:quest" content="string" data-doc="The tracked quest's name; empty hides the tracker.">
<meta name="prop:distance" content="string" data-doc="How far its next place is, as the game words it (240 m); empty hides it.">
<meta name="prop:objectives" content="list" data-doc="Its open objectives in the game's order: each { id, text, progress, done, optional }; progress is the game's count as words (2 / 3). A done objective stays until the game removes it.">
<meta name="prop:updated" content="string" data-doc="The id of the objective the game just changed: a new id plays its mark again.">
<meta name="fixture:default" content='{"quest":"The Drowned Ferryman","distance":"240 m","objectives":[{"id":"o2","text":"Light the marsh beacons","progress":"2 / 3","done":false,"optional":false},{"id":"o3","text":"Find the ferryman’s ledger","progress":"","done":false,"optional":true}],"updated":"o2"}'>
<meta name="fixture:empty" content='{"quest":"","distance":"","objectives":[],"updated":""}'>

<!--
  The rpg kit's quest tracker (README.md), the open-world HUD's objective
  list: the tracked quest, its distance and its open objectives, read at a
  glance over the world. It sends nothing: tracking changes in the
  journal. The game decides what is open and when a done objective goes.
-->
<aside id="RpgTracker" part="Tracker.frame" hidden={!quest} aria-label={t("rpg.tracker.name", { quest })} aria-live="polite">
  <div id="Head">
    <icon class="pin" icon="star"></icon>
    <span id="Quest">{quest}</span>
    <span id="Distance" class="number" hidden={!distance}>{distance}</span>
  </div>
  <ul id="Objectives">
    {#each objectives as o (o.id)}
      <li id="Objective" part="Tracker.objective" class="objective" class:done={o.done} class:optional={o.optional} class:updated={o.id == updated} aria-checked={o.done}>
        {#key updated}
          <div class="bullet" in:scale={{ start: 0.4, duration: 320, easing: backOut }}><icon class="state" icon="check"></icon></div>
        {/key}
        <span class="text">{o.text}</span>
        <span class="progress number" hidden={!o.progress}>{o.progress}</span>
      </li>
    {/each}
  </ul>
</aside>

<style>
#RpgTracker { width: stretch; max-width: calc(15 * var(--space-lg)); gap: var(--space-xs); padding: var(--space-sm) var(--space-md); }
#Head { display: flex; width: stretch; gap: var(--space-xs); align-items: center; }
.pin { width: var(--space-md); height: var(--space-md); color: var(--highlight); flex-shrink: 0; }
#Quest { width: stretch; font: var(--text-body); font-family: fantasy; color: var(--highlight); text-shadow: 0 1px 2px var(--shadow); }
.number { font-family: monospace; font-variant-numeric: tabular-nums; white-space: nowrap; }
#Distance { font: var(--text-caption); font-family: monospace; color: var(--ink-muted); }
#Objectives { margin: 0; padding: 0; list-style: none; width: stretch; gap: var(--space-xxs); }
.objective { display: flex; width: stretch; gap: var(--space-xs); align-items: start; }
.bullet { width: var(--space-md); height: var(--space-md); margin-top: var(--space-xxs); flex-shrink: 0; align-items: center; justify-content: center; border: var(--stroke-thin) solid var(--ink-muted); border-radius: var(--radius-xs); }
.state { width: var(--space-sm); height: var(--space-sm); color: var(--success); opacity: 0; }
.objective.done .state { opacity: 1; }
.objective.updated .bullet { border-color: var(--highlight); }
.text { width: stretch; font: var(--text-sm); color: var(--ink); text-shadow: 0 1px 2px var(--shadow); }
.objective.optional .text { font-style: italic; color: var(--ink-muted); }
.objective.done .text { color: var(--ink-muted); text-decoration: line-through; }
.progress { font: var(--text-sm); font-family: monospace; color: var(--ink); text-shadow: 0 1px 2px var(--shadow); }
@media (prefers-reduced-motion: reduce) {
  .bullet { transition: none; }
}
</style>
