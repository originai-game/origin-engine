<meta name="component" content="RpgQuestTracker">
<meta name="strings" content="./rpg.en.ogt">
<meta name="parts" content="RpgQuestTracker.frame RpgQuestTracker.title RpgQuestTracker.row">
<meta name="prop:name" content="string" data-doc="The tracked quest's name, on the plate; empty hides the tracker.">
<meta name="prop:steps" content="list" data-doc="Its current objectives, keyed rows of { id, text, done }: done ticks and quiets one.">
<meta name="prop:distance" content="string" data-doc="How far its marker is, as the game words it (312 m); empty hides it.">
<meta name="fixture:default" content='{"name":"The Fallen Shrine","steps":[{"id":"a","text":"Reach the Ashen Spire","done":true},{"id":"b","text":"Investigate the Bloodstone Circle","done":false}],"distance":"312 m"}'>
<meta name="fixture:none" content='{"name":"","steps":[],"distance":""}'>

<!--
  The tracked quest on the HUD: its name on a plate, its objectives with
  their ticks, and the distance to its marker. Read only; the journal owns
  choosing what to track.
-->
<section id="RpgQuestTracker" part="RpgQuestTracker.frame" class="hud" data-empty={!name} role="status" aria-label={t("rpg.quests.tracked")}>
  {#if name}
  <h3 part="RpgQuestTracker.title">{name}</h3>
  <ul class="tracker-steps">
    {#each steps as s (s.id)}
      <li part="RpgQuestTracker.row" data-done={s.done}><og-icon class="tracker-mark" icon={s.done ? 'check' : 'flag'}></og-icon><span class="tracker-text">{s.text}</span></li>
    {/each}
  </ul>
  {#if distance}<div class="tracker-distance"><og-icon class="tracker-arrow" icon="map"></og-icon><span class="tracker-far">{distance}</span></div>{/if}
  {/if}
</section>

<style>
/* Drawn over the scene: the style's tag halo holds its text there. */
:host { text-shadow: var(--tag-halo); }
:host { display: flex; flex-direction: column; inline-size: stretch; gap: var(--space-sm); padding: var(--space-md); }
[part="RpgQuestTracker.frame"][data-empty=true] { display: none; }
[part="RpgQuestTracker.title"] { margin: 0; text-align: start; font-size: var(--font-step-lg); }
.tracker-steps { display: flex; flex-direction: column; margin: 0; padding: 0; list-style: none; gap: var(--space-sm); }
[part="RpgQuestTracker.row"] { display: flex; align-items: center; gap: var(--space-sm); }
[part="RpgQuestTracker.row"][data-done=true] { color: var(--scene-ink); text-shadow: var(--tag-halo); }
.tracker-mark { width: var(--size-xs); height: var(--size-xs); flex: none; color: var(--primary); }
[part="RpgQuestTracker.row"][data-done=true] .tracker-mark { color: var(--success); }
.tracker-text { flex: 1; min-width: 0; }
.tracker-distance { display: flex; align-items: center; gap: var(--space-xs); padding-block-start: var(--space-xs); border-block-start: var(--stroke-hairline) solid var(--line); color: var(--ink-muted); font: var(--text-caption); }
.tracker-arrow { width: var(--space-md); height: var(--space-md); }
</style>
